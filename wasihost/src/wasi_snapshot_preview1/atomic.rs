#![allow(dead_code)]

use wasihost_core::wasi_snapshot_preview1::{Fdflags, Rights};

use std::{
    fmt,
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
};

pub(crate) struct AtomicFdflags(AtomicU16);

impl fmt::Debug for AtomicFdflags {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(fmt)
    }
}

impl Clone for AtomicFdflags {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl AtomicFdflags {
    pub(crate) fn new(flags: Fdflags) -> Self {
        Self(AtomicU16::new(flags.bits()))
    }

    pub(crate) fn get(&self) -> Fdflags {
        let bits = self.0.load(Ordering::Relaxed);
        unsafe { Fdflags::from_bits_unchecked(bits) }
    }

    pub(crate) fn set(&self, flags: Fdflags) {
        self.0.store(flags.bits(), Ordering::Relaxed);
    }
}

pub(crate) struct AtomicRights(AtomicU64);

impl fmt::Debug for AtomicRights {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(fmt)
    }
}

impl Clone for AtomicRights {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl AtomicRights {
    pub(crate) fn new(rights: Rights) -> Self {
        Self(AtomicU64::new(rights.bits()))
    }

    pub(crate) fn get(&self) -> Rights {
        let bits = self.0.load(Ordering::Relaxed);
        unsafe { Rights::from_bits_unchecked(bits) }
    }

    pub(crate) fn set(&self, rights: Rights) {
        self.0.store(rights.bits(), Ordering::Relaxed);
    }

    pub(crate) fn compare_and_swap(&self, current: Rights, new: Rights) -> bool {
        self.0
            .compare_and_swap(current.bits(), new.bits(), Ordering::Relaxed)
            == current.bits()
    }
}
