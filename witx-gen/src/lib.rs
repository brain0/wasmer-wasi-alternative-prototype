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

use std::{error::Error, fmt, marker::PhantomData, mem};
use wasmer_runtime_core::memory::Memory;

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

/// Pointer to a WASM value.
#[derive(Debug, Copy, Clone)]
pub struct WasmValuePtr<T: WasmValue> {
    offset: u32,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T: WasmValue> WasmValuePtr<T> {
    /// Reads the value from WASM memory.
    pub fn read(self, memory: &Memory) -> T {
        WasmValue::read(memory, self.offset)
    }

    /// Writes the value to WASM memory.
    pub fn write(self, memory: &Memory, value: T) {
        value.write(memory, self.offset);
    }
}

/// Pointer to a WASM slice.
#[derive(Debug, Copy, Clone)]
pub struct WasmSlicePtr<T: WasmValue> {
    offset: u32,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T: WasmValue> WasmSlicePtr<T> {
    /// Gets the value at the specified index.
    pub fn get(self, index: u32) -> WasmValuePtr<T> {
        WasmValuePtr {
            offset: self.offset + index * T::ARRAY_OFFSET,
            _phantom: PhantomData,
        }
    }

    /// Adds an offset to this pointer.
    pub fn add(self, offset: u32) -> WasmSlicePtr<T> {
        WasmSlicePtr {
            offset: self.offset + offset * T::ARRAY_OFFSET,
            _phantom: PhantomData,
        }
    }
}

unsafe impl<T: WasmValue> wasmer_runtime_core::types::WasmExternType for WasmValuePtr<T> {
    type Native = i32;

    fn from_native(native: Self::Native) -> Self {
        Self {
            offset: native as u32,
            _phantom: PhantomData,
        }
    }

    fn to_native(self) -> Self::Native {
        self.offset as i32
    }
}

unsafe impl<T: WasmValue> wasmer_runtime_core::types::WasmExternType for WasmSlicePtr<T> {
    type Native = i32;

    fn from_native(native: Self::Native) -> Self {
        Self {
            offset: native as u32,
            _phantom: PhantomData,
        }
    }

    fn to_native(self) -> Self::Native {
        self.offset as i32
    }
}

/// Reexported items that are used inside the `witx_gen` macro.
pub mod reexports {
    #[doc(no_inline)]
    pub use bitflags::bitflags;

    #[doc(no_inline)]
    pub use wasmer_runtime_core::{func, import::ImportObject, imports, memory::Memory, vm::Ctx};
}

/// A value that can be stored into WASM memory.
pub trait WasmValue: fmt::Debug + Copy {
    /// Offset between two elements of this type inside an array.
    const ARRAY_OFFSET: u32;

    /// Reads the value from memory at the given offset.
    fn read(memory: &Memory, offset: u32) -> Self;
    /// Writes the value to memory at the given offset.
    fn write(self, memory: &Memory, offset: u32);
}

impl WasmValue for u8 {
    const ARRAY_OFFSET: u32 = mem::size_of::<u8>() as u32;

    fn read(memory: &Memory, offset: u32) -> Self {
        memory.view::<u8>()[offset as usize].get()
    }

    fn write(self, memory: &Memory, offset: u32) {
        memory.view::<u8>()[offset as usize].set(self);
    }
}

macro_rules! primitive_wasmvalue_impl {
    ($t:ty) => {
        impl WasmValue for $t {
            const ARRAY_OFFSET: u32 = mem::size_of::<$t>() as u32;

            fn read(memory: &Memory, offset: u32) -> Self {
                let mut bytes = [0u8; mem::size_of::<Self>()];
                for i in 0..(mem::size_of::<Self>() as u32) {
                    bytes[i as usize] = <u8 as WasmValue>::read(memory, offset + i);
                }
                Self::from_le_bytes(bytes)
            }

            fn write(self, memory: &Memory, offset: u32) {
                let bytes = self.to_le_bytes();
                for i in 0..(mem::size_of::<Self>() as u32) {
                    <u8 as WasmValue>::write(bytes[i as usize], memory, offset + i);
                }
            }
        }
    };
}

primitive_wasmvalue_impl!(i8);
primitive_wasmvalue_impl!(u16);
primitive_wasmvalue_impl!(i16);
primitive_wasmvalue_impl!(u32);
primitive_wasmvalue_impl!(i32);
primitive_wasmvalue_impl!(u64);
primitive_wasmvalue_impl!(i64);
primitive_wasmvalue_impl!(f32);
primitive_wasmvalue_impl!(f64);

impl<T: WasmValue> WasmValue for WasmValuePtr<T> {
    const ARRAY_OFFSET: u32 = mem::size_of::<u32>() as u32;

    fn read(memory: &Memory, offset: u32) -> Self {
        WasmValuePtr {
            offset: WasmValue::read(memory, offset),
            _phantom: PhantomData,
        }
    }

    fn write(self, memory: &Memory, offset: u32) {
        self.offset.write(memory, offset);
    }
}

impl<T: WasmValue> WasmValue for WasmSlicePtr<T> {
    const ARRAY_OFFSET: u32 = mem::size_of::<u32>() as u32;

    fn read(memory: &Memory, offset: u32) -> Self {
        WasmSlicePtr {
            offset: WasmValue::read(memory, offset),
            _phantom: PhantomData,
        }
    }

    fn write(self, memory: &Memory, offset: u32) {
        self.offset.write(memory, offset);
    }
}
