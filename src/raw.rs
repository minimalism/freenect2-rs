//! Unsafe bindgen output for the C shim (`csrc/freenect2_shim.h`).
//!
//! Every function is `unsafe` to call. Misuse is undefined behavior.
//!
//! **Semver exempt:** this surface is not covered by the same stability guarantees as
//! [`crate::Kinect`] and friends; it may change when the shim or bindgen options change.

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
