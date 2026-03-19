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
