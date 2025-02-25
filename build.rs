//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::{env, fs::File, io::Write, path::PathBuf};

extern crate bindgen;
use bindgen::CargoCallbacks;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    #[allow(clippy::if_same_then_else)]
    let buf: &[u8] = if env::var_os("CARGO_FEATURE_MICROBIT_V2").is_some() {
        include_bytes!("microbit_v2_memory.x")
    } else if env::var_os("CARGO_FEATURE_NRF52840DK").is_some() {
        include_bytes!("nrf52840dk_memory.x")
    } else if env::var_os("CARGO_FEATURE_NRF52DK").is_some() {
        include_bytes!("nrf52dk_memory.x")
    } else {
        panic!("Unknown target - cannot determine memory.x - aborting")
    };
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(buf)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");





    cc::Build::new()
    .file("hello/hello.c")
    .file("hello/myled.c")
    .compile("hello");

    println!("hello target crated");


}
