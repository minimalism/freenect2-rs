# Rust bindings for libfreenect2

## Structure

```
freenect2-rs/
├── Cargo.toml
├── build.rs                  # orchestrates cc + bindgen
├── csrc/
│   ├── freenect2_shim.h
│   └── freenect2_shim.cpp    # C-callable wrappers around C++ API
├── src/
│   ├── lib.rs                # safe Rust wrappers
│   └── raw.rs                # re-exports bindgen output
└── vendor/include/libfreenect2   # libfreenect2 headers                
    ├── color_settings.hpp    
    ├── config.h.in
    ├── frame_listener_impl.h
    ├── frame_listener_impl.hpp    
    ├── led_settings.h
    ├── libfreenect2.hpp    
    ├── logger.h
    ├── packet_pipeline.h    
    ├── registration.h      
    ├── config.h            # libfreenect2 build config
    └── export.h            # libfreenect2 build config
```


## Build

### Prerequisites
* `libclang-dev` - for bindgen: `sudo apt install libclang-dev`
* `libusb-1.0-0-dev` - for libfreenect2 at runtime
* `libfreenect2.so` - See https://github.com/OpenKinect/libfreenect2

### Variables
* `FREENECT2_LIB_DIR` — tells `build.rs` where to find `libfreenect2.so` at compile time
* `LD_LIBRARY_PATH` — tells the runtime linker where to find it when running tests/binaries
Both should point to the same directory.

### Example
```
FREENECT2_LIB_DIR=$HOME/Dev/libfreenect2/build/lib \
LD_LIBRARY_PATH=$FREENECT2_LIB_DIR \
cargo test
```

## Troubleshooting

`[Error] [Freenect2Impl] failed to open Kinect v2: @2:3 LIBUSB_ERROR_ACCESS Access denied (insufficient permissions)`

You need a udev rule that grants your user permission to access the Kinect's USB device.
libfreenect2 actually ships these rules. Run:

```
sudo cp $HOME/Dev/libfreenect2/platform/linux/udev/90-kinect2.rules /etc/udev/rules.d/  # Or wherever you have libfreenect2 
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then unplug and replug the Kinect. udev rules only apply at connection time.
Verify that this works with:
```
lsusb | grep -i microsoft   # find the Kinect
# should show something like: Bus 002 Device 003: ID 045e:02c4 Microsoft Corp.
```

