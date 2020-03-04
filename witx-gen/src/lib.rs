//! This is highly experimental and entirely untested. Do not try to use it.

#![forbid(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]
#![warn(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    missing_docs,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

mod ptr;

use std::{cell::Cell, error::Error, fmt, mem};

pub use self::ptr::*;
pub use witx_gen_macros::witx_gen;

/// Trait to convert WASI values between the Rust and native WASM representation.
pub trait WasiValue: Sized {
    /// The native WASM type.
    type NativeType: Copy + fmt::Debug;

    /// Converts a native WASM value to its Rust representation.
    fn from_native(native: Self::NativeType) -> Result<Self, WasiValueError<Self>>;
    /// Converts a Rust value to its native WASM representation.
    fn to_native(self) -> Self::NativeType;
}

#[doc(hidden)]
pub trait WasiLeafValue: WasiValue {
    fn get_error_message(value: Self::NativeType) -> String;
}

impl<T: WasiValue> WasiLeafValue for T
where
    T::NativeType: fmt::Display,
{
    fn get_error_message(native: Self::NativeType) -> String {
        format!(
            "Could not convert the native value {} to {}.",
            native,
            std::any::type_name::<T>()
        )
    }
}

/// Error type for representing a conversion error from the native WASM value
/// to the Rust value.
pub struct WasiValueError<T: WasiValue>(T::NativeType, String);

impl<T: WasiLeafValue> WasiValueError<T> {
    #[doc(hidden)]
    pub fn from_native(native: T::NativeType) -> Self {
        WasiValueError(native, T::get_error_message(native))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! try_from_native {
    ($n:expr, $i:expr) => {
        match $crate::WasiValue::from_native($i) {
            Ok(value) => value,
            Err(error) => return Err($crate::WasiValueError::from_inner($n, error)),
        }
    };
}

impl<T: WasiValue> WasiValueError<T> {
    #[doc(hidden)]
    pub fn from_inner<U: WasiValue>(native: T::NativeType, inner: WasiValueError<U>) -> Self {
        WasiValueError(native, inner.1)
    }

    #[doc(hidden)]
    pub fn from_unknown(native: T::NativeType) -> Self {
        WasiValueError(native, "Cannot convert unknown union value.".into())
    }

    /// Returns the value that failed to convert.
    pub fn as_native(self) -> T::NativeType {
        self.0
    }
}

impl<T: WasiValue> Clone for WasiValueError<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

impl<T: WasiValue> fmt::Debug for WasiValueError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("WasiValueError").field(&self.0).finish()
    }
}

impl<T: WasiValue> fmt::Display for WasiValueError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "Could not convert the native value {} to {}.",
            self.1,
            std::any::type_name::<T>()
        )
    }
}

impl<T: WasiValue> Error for WasiValueError<T> {}

/// Reexported items that are used inside the `witx_gen` macro.
pub mod reexports {
    #[doc(no_inline)]
    pub use bitflags::bitflags;

    #[doc(no_inline)]
    pub use wasmer_runtime_core::{func, import::ImportObject, imports, memory::Memory, vm::Ctx};
}

/// A value that can be stored into WASM memory.
pub trait WasmValue: fmt::Debug + Copy {
    /// Size of a value
    const SIZE: u32;
    /// Offset between two elements of this type inside an array.
    const ARRAY_OFFSET: u32;

    /// Reads the value from memory at the given offset.
    fn read(mem: &[Cell<u8>]) -> Self;

    /// Writes the value to memory at the given offset.
    fn write(self, mem: &[Cell<u8>]);
}

macro_rules! primitive_wasmvalue_impl {
    ($t:ty) => {
        impl WasmValue for $t {
            const SIZE: u32 = mem::size_of::<$t>() as u32;
            const ARRAY_OFFSET: u32 = mem::size_of::<$t>() as u32;

            fn read(mem: &[Cell<u8>]) -> Self {
                const SIZE: usize = mem::size_of::<$t>();

                assert_eq!(mem.len(), SIZE);
                let mut bytes = [0u8; SIZE];
                for i in 0..SIZE {
                    bytes[i] = mem[i].get();
                }
                Self::from_le_bytes(bytes)
            }

            fn write(self, mem: &[Cell<u8>]) {
                const SIZE: usize = mem::size_of::<$t>();

                assert_eq!(mem.len(), SIZE);
                let bytes = self.to_le_bytes();
                for i in 0..SIZE {
                    mem[i].set(bytes[i])
                }
            }
        }
    };
}

primitive_wasmvalue_impl!(u8);
primitive_wasmvalue_impl!(i8);
primitive_wasmvalue_impl!(u16);
primitive_wasmvalue_impl!(i16);
primitive_wasmvalue_impl!(u32);
primitive_wasmvalue_impl!(i32);
primitive_wasmvalue_impl!(u64);
primitive_wasmvalue_impl!(i64);
primitive_wasmvalue_impl!(f32);
primitive_wasmvalue_impl!(f64);
