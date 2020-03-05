//! Defines string representations for the high level WASI APIs.

use std::ffi::{CStr, CString};
#[cfg(unix)]
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
};

/// A trait that defines the string representation used in the WASI high-level APIs.
pub trait StringRepresentation: Send + Sync + 'static {
    /// The type of an owned string.
    type Owned: Send + Sync + 'static;
    /// The type of a borrowed string.
    type Borrowed: ?Sized + Send + Sync + 'static;

    /// Tries to convert a vector of owned bytes into an owned string.
    /// If the conversion fails, the WASI method that attempted the conversion
    /// will return errno_inval.
    fn owned_from_bytes(bytes: Vec<u8>) -> Result<Self::Owned, ()>;

    /// Converts an owned string into a byte slice.
    fn owned_as_bytes(string: &Self::Owned) -> &[u8];

    /// Converts an owned string into a borrowed string.
    fn borrow(string: &Self::Owned) -> &Self::Borrowed;
}

/// String representation using `std::string::String` (UTF8).
#[derive(Debug)]
pub struct Utf8;

impl StringRepresentation for Utf8 {
    type Owned = String;
    type Borrowed = str;

    fn owned_from_bytes(bytes: Vec<u8>) -> Result<String, ()> {
        String::from_utf8(bytes).map_err(|_| ())
    }

    fn owned_as_bytes(string: &String) -> &[u8] {
        string.as_bytes()
    }

    fn borrow(string: &String) -> &str {
        string
    }
}

/// String representation using raw byte arrays. Conversion to this
/// string type never fails, since any sequence of bytes represents
/// a valid string.
#[derive(Debug)]
pub struct Bytes;

impl StringRepresentation for Bytes {
    type Owned = Vec<u8>;
    type Borrowed = [u8];

    fn owned_from_bytes(bytes: Vec<u8>) -> Result<Vec<u8>, ()> {
        Ok(bytes)
    }

    fn owned_as_bytes(string: &Vec<u8>) -> &[u8] {
        string
    }

    fn borrow(string: &Vec<u8>) -> &[u8] {
        string
    }
}

#[cfg(unix)]
/// String representation using `std::ffi::OsString`.
#[derive(Debug)]
pub struct Os;

#[cfg(unix)]
impl StringRepresentation for Os {
    type Owned = OsString;
    type Borrowed = OsStr;

    fn owned_from_bytes(bytes: Vec<u8>) -> Result<OsString, ()> {
        Ok(OsString::from_vec(bytes))
    }

    fn owned_as_bytes(string: &OsString) -> &[u8] {
        string.as_bytes()
    }

    fn borrow(string: &OsString) -> &OsStr {
        string
    }
}

/// String representation using `std::ffi::CString`.
#[derive(Debug)]
pub struct C;

impl StringRepresentation for C {
    type Owned = CString;
    type Borrowed = CStr;

    fn owned_from_bytes(bytes: Vec<u8>) -> Result<CString, ()> {
        CString::new(bytes).map_err(|_| ())
    }

    fn owned_as_bytes(string: &CString) -> &[u8] {
        string.to_bytes()
    }

    fn borrow(string: &CString) -> &CStr {
        string
    }
}
