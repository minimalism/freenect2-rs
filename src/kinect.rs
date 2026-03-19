//! Safe wrapper around the libfreenect2 C shim for a single streaming device.

use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::raw;

/// Errors from [`Kinect::open_first`] and [`Kinect::wait_for_frame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinectError {
    /// `fn2_create` returned null.
    NullContext,
    /// `enumerateDevices` reported zero Kinects.
    NoDevice,
    /// `openDevice` failed.
    OpenFailed,
    /// `fn2_create_sync_listener` returned null.
    NullListener,
    /// `waitForNewFrame` timed out.
    WaitTimeout,
    /// A frame set was missing color or depth.
    IncompleteFrame,
}

impl fmt::Display for KinectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KinectError::NullContext => write!(f, "failed to create libfreenect2 context"),
            KinectError::NoDevice => write!(f, "no Kinect device enumerated"),
            KinectError::OpenFailed => write!(f, "openDevice failed"),
            KinectError::NullListener => write!(f, "failed to create sync frame listener"),
            KinectError::WaitTimeout => write!(f, "timed out waiting for frames"),
            KinectError::IncompleteFrame => write!(f, "frame set missing color or depth"),
        }
    }
}

impl std::error::Error for KinectError {}

/// Pixel layout from libfreenect2 [`Frame::Format`](https://openkinect.github.io/libfreenect2/classlibfreenect2_1_1Frame.html) (discriminants match the C++ enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FrameFormat {
    Invalid,
    Raw,
    /// 32-bit float samples (e.g. IR). Not used for the depth channel in [`Frames::depth_format`].
    Float,
    Bgrx,
    Rgbx,
    Gray,
    /// Depth stream: one `f32` per pixel, millimeters (invalid / missing values per libfreenect2 docs).
    F32Millimeters,
}

impl FrameFormat {
    /// Maps the integer stored in [`raw::FrameData`] (`Frame::Format` in C++).
    pub fn from_libfreenect2(raw: i32) -> Self {
        match raw {
            0 => Self::Invalid,
            1 => Self::Raw,
            2 => Self::Float,
            4 => Self::Bgrx,
            5 => Self::Rgbx,
            6 => Self::Gray,
            _ => Self::Invalid,
        }
    }
}

/// One synchronized color + depth capture. Dropping it returns buffers to libfreenect2.
///
/// While a `Frames` value exists, the parent [`Kinect`] is mutably borrowed and you cannot call
/// [`Kinect::wait_for_frame`] again.
pub struct Frames<'a> {
    listener: raw::ListenerHandle,
    fd: raw::FrameData,
    _kinect: PhantomData<&'a mut Kinect>,
}

impl<'a> Frames<'a> {
    fn new(listener: raw::ListenerHandle, fd: raw::FrameData) -> Self {
        Self {
            listener,
            fd,
            _kinect: PhantomData,
        }
    }

    fn byte_len(width: i32, height: i32, bpp: i32) -> Option<usize> {
        if width <= 0 || height <= 0 || bpp <= 0 {
            return None;
        }
        let w = width as usize;
        let h = height as usize;
        let b = bpp as usize;
        w.checked_mul(h)?.checked_mul(b)
    }

    /// Raw BGRX/RGBX color bytes (layout depends on device / pipeline).
    pub fn color_bytes(&self) -> &[u8] {
        self.bytes_from_field(
            self.fd.color_data,
            self.fd.color_width,
            self.fd.color_height,
            self.fd.color_bpp,
        )
    }

    /// Raw depth buffer: `4 * width * height` bytes (`f32` mm per pixel in libfreenect2).
    pub fn depth_bytes(&self) -> &[u8] {
        self.bytes_from_field(
            self.fd.depth_data,
            self.fd.depth_width,
            self.fd.depth_height,
            self.fd.depth_bpp,
        )
    }

    pub fn color_width(&self) -> i32 {
        self.fd.color_width
    }

    pub fn color_height(&self) -> i32 {
        self.fd.color_height
    }

    pub fn color_bytes_per_pixel(&self) -> i32 {
        self.fd.color_bpp
    }

    pub fn depth_width(&self) -> i32 {
        self.fd.depth_width
    }

    pub fn depth_height(&self) -> i32 {
        self.fd.depth_height
    }

    pub fn depth_bytes_per_pixel(&self) -> i32 {
        self.fd.depth_bpp
    }

    /// Color pixel layout (typically [`Bgrx`](FrameFormat::Bgrx) or [`Rgbx`](FrameFormat::Rgbx) depending on pipeline).
    pub fn color_format(&self) -> FrameFormat {
        FrameFormat::from_libfreenect2(self.fd.color_format)
    }

    /// Depth sample layout for Kinect v2: [`F32Millimeters`](FrameFormat::F32Millimeters) when the device reports `Frame::Float`.
    pub fn depth_format(&self) -> FrameFormat {
        match self.fd.depth_format {
            2 => FrameFormat::F32Millimeters,
            raw => FrameFormat::from_libfreenect2(raw),
        }
    }

    fn bytes_from_field(&self, data: *mut c_void, width: i32, height: i32, bpp: i32) -> &[u8] {
        let Some(len) = Self::byte_len(width, height, bpp) else {
            return &[];
        };
        if data.is_null() {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) }
    }
}

impl Drop for Frames<'_> {
    fn drop(&mut self) {
        unsafe {
            raw::fn2_release_frame(self.listener, &mut self.fd);
        }
    }
}

/// Connected Kinect v2 device with color + depth streaming enabled.
pub struct Kinect {
    f2: raw::Freenect2Handle,
    dev: raw::DeviceHandle,
    listener: raw::ListenerHandle,
}

impl Kinect {
    /// Create a context, open device `0`, attach a sync listener, and call `start()`.
    pub fn open_first() -> Result<Self, KinectError> {
        unsafe {
            let f2 = raw::fn2_create();
            if f2.is_null() {
                return Err(KinectError::NullContext);
            }

            if raw::fn2_enumerate_devices(f2) <= 0 {
                raw::fn2_destroy(f2);
                return Err(KinectError::NoDevice);
            }

            let dev = raw::fn2_open_device(f2, 0);
            if dev.is_null() {
                raw::fn2_destroy(f2);
                return Err(KinectError::OpenFailed);
            }

            let listener = raw::fn2_create_sync_listener();
            if listener.is_null() {
                raw::fn2_close_device(dev);
                raw::fn2_destroy(f2);
                return Err(KinectError::NullListener);
            }

            raw::fn2_set_listeners(dev, listener);
            raw::fn2_start_device(dev);

            Ok(Self { f2, dev, listener })
        }
    }

    /// Block until color and depth are available or `timeout_ms` elapses.
    ///
    /// The C shim returns `0` both on timeout and on an incomplete frame set, so failures are reported
    /// as [`KinectError::WaitTimeout`] only. The extra field checks below are defensive (they should
    /// be unreachable when the shim returns success).
    pub fn wait_for_frame(&mut self, timeout_ms: i32) -> Result<Frames<'_>, KinectError> {
        let mut fd: raw::FrameData = unsafe { std::mem::zeroed() };
        let ok = unsafe { raw::fn2_wait_for_frame(self.listener, &mut fd, timeout_ms) };
        if ok == 0 {
            return Err(KinectError::WaitTimeout);
        }
        if fd.color_data.is_null()
            || fd.depth_data.is_null()
            || fd.color_width <= 0
            || fd.color_height <= 0
            || fd.depth_width <= 0
            || fd.depth_height <= 0
        {
            unsafe {
                raw::fn2_release_frame(self.listener, &mut fd);
            }
            return Err(KinectError::IncompleteFrame);
        }
        Ok(Frames::new(self.listener, fd))
    }
}

impl Drop for Kinect {
    fn drop(&mut self) {
        unsafe {
            let listener = std::mem::replace(&mut self.listener, std::ptr::null_mut());
            let dev = std::mem::replace(&mut self.dev, std::ptr::null_mut());
            let f2 = std::mem::replace(&mut self.f2, std::ptr::null_mut());

            if !listener.is_null() {
                let mut fd: raw::FrameData = std::mem::zeroed();
                raw::fn2_release_frame(listener, &mut fd);
            }
            if !dev.is_null() {
                raw::fn2_stop_device(dev);
                raw::fn2_close_device(dev);
            }
            if !listener.is_null() {
                raw::fn2_destroy_listener(listener);
            }
            if !f2.is_null() {
                raw::fn2_destroy(f2);
            }
        }
    }
}
