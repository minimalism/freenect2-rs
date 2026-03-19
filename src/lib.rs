//! Rust bindings for [libfreenect2](https://github.com/OpenKinect/libfreenect2).
//!
//! Use [`Kinect`] for a safe entry point. The [`raw`] module is an unsafe escape hatch and is
//! **semver-exempt** (layout and symbols may change without a major version bump).

mod kinect;

pub use kinect::{FrameFormat, Frames, Kinect, KinectError};

pub mod raw;

#[test]
fn test_create_destroy() {
    unsafe {
        let p = raw::fn2_create();
        assert!(!p.is_null());
        raw::fn2_destroy(p);
    }
}

/// End-to-end capture path (requires a connected Kinect v2 and working libfreenect2 stack).
#[test]
#[ignore]
fn test_capture_frame() {
    let mut kinect = Kinect::open_first().expect("open first device");
    let frames = kinect
        .wait_for_frame(30_000)
        .expect("waitForNewFrame timed out or incomplete frame set");

    assert!(!frames.color_bytes().is_empty());
    assert!(!frames.depth_bytes().is_empty());
    assert!(frames.color_width() > 0 && frames.color_height() > 0);
    assert!(frames.depth_width() > 0 && frames.depth_height() > 0);
    assert!(
        matches!(frames.color_format(), FrameFormat::Bgrx | FrameFormat::Rgbx),
        "expected BGRX or RGBX color from libfreenect2, got {:?}",
        frames.color_format()
    );
    assert_eq!(frames.depth_format(), FrameFormat::F32Millimeters);
}
