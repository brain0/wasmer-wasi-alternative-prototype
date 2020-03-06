use std::{
    fmt::Debug,
    io::{stderr, stdin, stdout, IoSlice, IoSliceMut, Read, Write},
};
use wasihost_core::wasi_snapshot_preview1::{Size, WasiResult};

/// Describes a character device.
pub trait CharacterDevice: Debug + Send + Sync + 'static {
    /// Reads data from the character device into `iovs`.
    fn read(&self, iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size>;
    /// Writes data from `bufs` to the character device.
    fn write(&self, bufs: &[IoSlice<'_>]) -> WasiResult<Size>;
}

/// The host's standard input.
#[derive(Debug)]
pub struct Stdin;

impl CharacterDevice for Stdin {
    fn read(&self, iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size> {
        Ok(stdin().lock().read_vectored(iovs).map(|s| Size(s as u32))?)
    }

    fn write(&self, _bufs: &[IoSlice<'_>]) -> WasiResult<Size> {
        Ok(Size(0))
    }
}

/// The host's standard output.
#[derive(Debug)]
pub struct Stdout;

impl CharacterDevice for Stdout {
    fn read(&self, _iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size> {
        Ok(Size(0))
    }

    fn write(&self, bufs: &[IoSlice<'_>]) -> WasiResult<Size> {
        Ok(stdout()
            .lock()
            .write_vectored(bufs)
            .map(|s| Size(s as u32))?)
    }
}

/// The host's standard error.
#[derive(Debug)]
pub struct Stderr;

impl CharacterDevice for Stderr {
    fn read(&self, _iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size> {
        Ok(Size(0))
    }

    fn write(&self, bufs: &[IoSlice<'_>]) -> WasiResult<Size> {
        Ok(stderr()
            .lock()
            .write_vectored(bufs)
            .map(|s| Size(s as u32))?)
    }
}
