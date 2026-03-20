# freenect2-rs

Rust bindings for [libfreenect2](https://github.com/OpenKinect/libfreenect2) (Kinect for Windows v2 / Kinect v2). The crate links against a **system-installed** `libfreenect2` and a small C++ shim that exposes a C ABI.

## Requirements

- **Hardware:** Kinect v2 and a working USB 3.0 setup (see [upstream docs](https://github.com/OpenKinect/libfreenect2)).
- **Library:** Install libfreenect2 with CMake (`cmake --install` or `make install`) so `libfreenect2.so` is on the system library path. Run **`sudo ldconfig`** after install if needed, then confirm with e.g. **`ldconfig -p | grep freenect2`**.
- **Headers:** This repo vendors headers under `vendor/include/libfreenect2` for building the shim; they should match the installed library version when possible.
- **Build:** C++ toolchain, `libclang` for [bindgen](https://rust-lang.github.io/rust-bindgen/) (e.g. Debian/Ubuntu: `libclang-dev`).
- **Runtime:** libusb and other dependencies your libfreenect2 build expects (e.g. `libusb-1.0-0`).

## Usage

Add the dependency from [crates.io](https://crates.io/crates/freenect2-rs):

```toml
[dependencies]
freenect2-rs = "0.1"
```

Minimal capture loop:

```rust
use freenect2_rs::{FrameFormat, Kinect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut kinect = Kinect::open_first()?;
    let frames = kinect.wait_for_frame(10_000)?;

    assert!(matches!(
        frames.color_format(),
        FrameFormat::Bgrx | FrameFormat::Rgbx
    ));
    assert_eq!(frames.depth_format(), FrameFormat::F32Millimeters);

    let _color = frames.color_bytes();
    let _depth = frames.depth_bytes();
    Ok(())
}
```

- **`Kinect::open_first()`** — Creates a libfreenect2 context, opens device index `0`, attaches a sync color+depth listener, and starts streaming.
- **`Kinect::wait_for_frame(timeout_ms)`** — Returns **`Frames<'_>`**, which **borrows the `Kinect` mutably** until it is dropped. You cannot call `wait_for_frame` again while a `Frames` value exists; dropping `Frames` releases buffers back to libfreenect2.
- **`FrameFormat`** — Describes color (typically BGRX/RGBX) and depth (**`F32Millimeters`**: `f32` per pixel, millimeters).

For low-level access, see the **`raw`** module (bindgen output). It is **`unsafe`** and treated as **semver-exempt** relative to the safe API.

## Layout

```
freenect2-rs/
├── Cargo.toml
├── build.rs                  # cc (shim) + bindgen + link libfreenect2
├── csrc/
│   ├── freenect2_shim.h
│   └── freenect2_shim.cpp    # C ABI around libfreenect2 C++ API
├── src/
│   ├── lib.rs                # crate root, re-exports
│   ├── kinect.rs             # safe `Kinect`, `Frames`, `FrameFormat`, …
│   └── raw.rs                # `include!` of generated `bindings.rs`
└── vendor/include/libfreenect2   # headers for building the shim
```

## Build

With libfreenect2 installed and visible to the **linker** and **dynamic loader** (standard paths + `ldconfig`), no extra environment variables are required:

```bash
cargo build
cargo test
```

If the linker cannot find `-lfreenect2`, check that the install prefix’s `lib` directory is configured (e.g. `/usr/local/lib` in `/etc/ld.so.conf` or `ld.so.conf.d/`, then `sudo ldconfig`).

### Tests

- **`test_create_destroy`** — Linkage smoke test (no device).
- **`test_capture_frame`** — Full open + one frame; **`#[ignore]`** by default (needs hardware):
  ```bash
  cargo test test_capture_frame -- --ignored
  ```

## Troubleshooting

### `LIBUSB_ERROR_ACCESS` / permission denied

You need a udev rule so your user can open the Kinect. libfreenect2 ships one:

```bash
sudo cp /path/to/libfreenect2/platform/linux/udev/90-kinect2.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug and replug the device after changing rules.

Check enumeration:

```bash
lsusb | grep -i microsoft
# e.g. Bus 002 Device 003: ID 045e:02c4 Microsoft Corp.
```

### bindgen: `Unable to find libclang`

Install LLVM/Clang dev packages and, if needed, set **`LIBCLANG_PATH`** to the directory containing `libclang.so` (see [bindgen docs](https://rust-lang.github.io/rust-bindgen/requirements.html)).

### Link or load errors for `libfreenect2`

- **Link time:** Ensure the library is installed under a directory the linker searches, or extend `LIBRARY_PATH` / toolchain defaults for your platform.
- **Run time:** Ensure the dynamic loader can find the `.so` (same as above; **`ldconfig -p | grep freenect2`** is a good check on Linux).

## Upstream licensing

[libfreenect2](https://github.com/OpenKinect/libfreenect2) is licensed under Apache-2.0 / GPL-2.0 (dual). Linking this crate loads that library; comply with its terms for distribution.
