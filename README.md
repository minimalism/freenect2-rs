# Rust bindings for libfreenect2

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

### Variables

* Path to `libfreenect2.so` is needed.

