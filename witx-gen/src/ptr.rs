use super::WasmValue;
use std::{cell::Cell, marker::PhantomData, mem};
use wasmer_runtime_core::memory::Memory;

trait MemoryExt {
    fn get_memory_as_slice(&self) -> &[Cell<u8>];
}

impl MemoryExt for Memory {
    fn get_memory_as_slice(&self) -> &[Cell<u8>] {
        let memory_view = self.view::<u8>();
        let result = &*memory_view;
        // Safety: result actually borrows from self, not from memory_view.
        //         This is a limitation of deref, and this should be fixed in
        //         wasmer_runtime_core by providing a method for this.
        unsafe { std::slice::from_raw_parts(result.as_ptr(), result.len()) }
    }
}

/// Pointer to a WASM value.
#[derive(Debug, Copy, Clone)]
pub struct WasmValuePtr<T: WasmValue> {
    offset: u32,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T: WasmValue> WasmValuePtr<T> {
    fn get_memory<'a>(self, memory: &'a Memory) -> &'a [Cell<u8>] {
        let start = self.offset as usize;
        let end = start + <T as WasmValue>::SIZE as usize;

        &memory.get_memory_as_slice()[start..end]
    }

    /// Reads the value from WASM memory.
    pub fn read(self, memory: &Memory) -> T {
        <T as WasmValue>::read(self.get_memory(memory))
    }

    /// Writes the value to WASM memory.
    pub fn write(self, memory: &Memory, value: T) {
        <T as WasmValue>::write(value, self.get_memory(memory));
    }
}

impl<T: WasmValue> WasmValue for WasmValuePtr<T> {
    const SIZE: u32 = mem::size_of::<u32>() as u32;
    const ARRAY_OFFSET: u32 = mem::size_of::<u32>() as u32;

    fn read(mem: &[Cell<u8>]) -> Self {
        WasmValuePtr {
            offset: WasmValue::read(mem),
            _phantom: PhantomData,
        }
    }

    fn write(self, mem: &[Cell<u8>]) {
        self.offset.write(mem)
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

/// Pointer to a WASM slice.
#[derive(Debug, Copy, Clone)]
pub struct WasmSlicePtr<T: WasmValue> {
    offset: u32,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T: WasmValue> WasmSlicePtr<T> {
    /// Get the slice of WASM memory associated with this pointer from the specified memory,
    /// with the specified length.
    pub fn with(self, memory: &Memory, length: u32) -> WasmMemorySlice<'_, T> {
        let start = self.offset as usize;
        let end = start + (length * T::ARRAY_OFFSET) as usize;

        WasmMemorySlice {
            memory: &memory.get_memory_as_slice()[start..end],
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

/// A slice of WASM memory.
#[derive(Debug)]
pub struct WasmMemorySlice<'a, T: WasmValue> {
    memory: &'a [Cell<u8>],
    _phantom: PhantomData<fn(T) -> T>,
}

impl<'a, T: WasmValue> WasmMemorySlice<'a, T> {
    fn get_memory(&self, index: u32) -> &[Cell<u8>] {
        let start = (index + T::ARRAY_OFFSET) as usize;
        let end = start + T::SIZE as usize;

        &self.memory[start..end]
    }

    /// Reads the i'th value from WASM memory.
    pub fn read(&self, index: u32) -> T {
        <T as WasmValue>::read(self.get_memory(index))
    }

    /// Writes the i'th value to WASM memory.
    pub fn write(&self, index: u32, value: T) {
        <T as WasmValue>::write(value, self.get_memory(index));
    }
}

impl<T: WasmValue> WasmValue for WasmSlicePtr<T> {
    const SIZE: u32 = mem::size_of::<u32>() as u32;
    const ARRAY_OFFSET: u32 = mem::size_of::<u32>() as u32;

    fn read(mem: &[Cell<u8>]) -> Self {
        WasmSlicePtr {
            offset: WasmValue::read(mem),
            _phantom: PhantomData,
        }
    }

    fn write(self, mem: &[Cell<u8>]) {
        self.offset.write(mem)
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
