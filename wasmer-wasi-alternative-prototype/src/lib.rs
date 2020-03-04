//! This is highly experimental and entirely untested. Do not try to use it.

#![forbid(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

pub mod wasi_snapshot_preview1;
