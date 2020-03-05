#![forbid(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]
#![warn(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

use std::env;
use wasihost::{string_representation::Utf8, wasi_snapshot_preview1::WasiHost};

fn main() {
    let wasm_filename = env::args().skip(1).next().unwrap();

    let arguments = env::args().skip(1);
    let environment = env::vars().map(|(key, value)| format!("{}={}", key, value));

    let wasi_host = WasiHost::<Utf8>::new(arguments, environment);

    let code = wasi_host
        .run_file(&wasm_filename)
        .expect("Unable to run WASM binary");

    eprintln!("WASI program exited with exit code {}.", code);
}
