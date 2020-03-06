//! Defines string representations for the high level WASI APIs.

use std::ffi::CString;
#[cfg(unix)]
use std::{
    ffi::OsString,
    ops::Deref,
    os::unix::ffi::{OsStrExt, OsStringExt},
};

/// A string representation for the WASI high level APIs.
pub trait StringRepresentation: Deref + Sized + Send + Sync + 'static {
    /// Tries to convert a vector of owned bytes into a string. If the conversion fails,
    /// the WASI method that attempted the conversion will return `errno_inval`.
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ()>;

    /// Converts a string into a byte slice.
    fn as_bytes(&self) -> &[u8];
}

impl StringRepresentation for String {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ()> {
        String::from_utf8(bytes).map_err(|_| ())
    }

    fn as_bytes(&self) -> &[u8] {
        self.deref().as_bytes()
    }
}

impl StringRepresentation for Vec<u8> {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ()> {
        Ok(bytes)
    }

    fn as_bytes(&self) -> &[u8] {
        self
    }
}

#[cfg(unix)]
impl StringRepresentation for OsString {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ()> {
        Ok(OsString::from_vec(bytes))
    }

    fn as_bytes(&self) -> &[u8] {
        self.deref().as_bytes()
    }
}

impl StringRepresentation for CString {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ()> {
        CString::new(bytes).map_err(|_| ())
    }

    fn as_bytes(&self) -> &[u8] {
        self.to_bytes()
    }
}
