//! Rust bindings for [libfreenect2](https://github.com/OpenKinect/libfreenect2).

pub mod raw;

#[test]
fn test_create_destroy() {
    unsafe {
        let p = raw::fn2_create();
        assert!(!p.is_null());
        raw::fn2_destroy(p);
    }
}

#[cfg(test)]
struct KinectCaptureSession {
    f2: raw::Freenect2Handle,
    dev: raw::DeviceHandle,
    listener: raw::ListenerHandle,
}

#[cfg(test)]
impl KinectCaptureSession {
    /// Connects to the first enumerated device and starts streaming (same steps as `test_capture_frame`).
    unsafe fn connect_default() -> Self {
        unsafe {
            let f2 = raw::fn2_create();
            assert!(!f2.is_null());

            let n = raw::fn2_enumerate_devices(f2);
            assert!(n > 0, "no Kinect enumerated");

            let dev = raw::fn2_open_device(f2, 0);
            assert!(!dev.is_null(), "openDevice failed");

            let listener = raw::fn2_create_sync_listener();
            assert!(!listener.is_null());

            raw::fn2_set_listeners(dev, listener);
            raw::fn2_start_device(dev);

            Self {
                f2,
                dev,
                listener,
            }
        }
    }
}

#[cfg(test)]
impl Drop for KinectCaptureSession {
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

/// End-to-end capture path (requires a connected Kinect v2 and working libfreenect2 stack).
#[test]
#[ignore]
fn test_capture_frame() {
    unsafe {
        let session = KinectCaptureSession::connect_default();

        let mut fd: raw::FrameData = std::mem::zeroed();
        let ok = raw::fn2_wait_for_frame(session.listener, &mut fd, 30_000);
        assert_eq!(ok, 1, "waitForNewFrame timed out or incomplete frame set");

        assert!(!fd.color_data.is_null());
        assert!(!fd.depth_data.is_null());
        assert!(fd.color_width > 0 && fd.color_height > 0);
        assert!(fd.depth_width > 0 && fd.depth_height > 0);

        raw::fn2_release_frame(session.listener, &mut fd);
    }
}
