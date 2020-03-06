#![allow(unused_variables)] // Remove when everything is implemented.

mod character_device;

use super::atomic::{AtomicFdflags, AtomicRights};
use std::{
    io::{IoSlice, IoSliceMut},
    marker::PhantomData,
};
use wasihost_core::{
    string_representation::StringRepresentation,
    wasi_snapshot_preview1::{
        Advice, Dircookie, Dirent, Errno, Fdflags, Fdstat, Filedelta, Filesize, Filestat, Filetype,
        Fstflags, Lookupflags, Oflags, Prestat, Riflags, Rights, Roflags, Sdflags, Siflags, Size,
        Timestamp, WasiResult, Whence,
    },
};

#[allow(unreachable_pub)] // false positive
pub use character_device::{CharacterDevice, Stderr, Stdin, Stdout};

#[derive(Debug)]
enum WasiFdInner {
    CharacterDevice(Box<dyn CharacterDevice>),
}

/// A WASI file descriptor.
#[derive(Debug)]
pub struct WasiFd<S> {
    inner: WasiFdInner,
    flags: AtomicFdflags,
    rights: AtomicRights,
    rights_inheriting: AtomicRights,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S: StringRepresentation> WasiFd<S> {
    /// Creates a WASI file descriptor from a character device.
    pub fn from_character_device<C: CharacterDevice>(
        character_device: C,
        flags: Fdflags,
        rights: Rights,
    ) -> Self {
        let inner = WasiFdInner::CharacterDevice(Box::new(character_device));
        let flags = AtomicFdflags::new(flags);
        let rights = AtomicRights::new(rights);
        let rights_inheriting = AtomicRights::new(Rights::empty());

        WasiFd {
            inner,
            flags,
            rights,
            rights_inheriting,
            _phantom: PhantomData,
        }
    }

    fn check_rights(&self, required: Rights) -> WasiResult<()> {
        Self::check_rights_with(self.rights.get(), required)
    }

    fn check_rights_with(actual: Rights, required: Rights) -> WasiResult<()> {
        if actual.contains(required) {
            Ok(())
        } else {
            Err(Errno::Notcapable)
        }
    }

    fn get_filetype(&self) -> Filetype {
        match self.inner {
            WasiFdInner::CharacterDevice(_) => Filetype::CharacterDevice,
        }
    }

    pub(super) fn advise(&self, offset: Filesize, len: Filesize, advice: Advice) -> WasiResult<()> {
        self.check_rights(Rights::FD_ADVISE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn allocate(&self, offset: Filesize, len: Filesize) -> WasiResult<()> {
        self.check_rights(Rights::FD_ALLOCATE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn datasync(&self) -> WasiResult<()> {
        self.check_rights(Rights::FD_DATASYNC)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn fdstat_get(&self) -> WasiResult<Fdstat> {
        Ok(Fdstat {
            fs_filetype: self.get_filetype(),
            fs_flags: self.flags.get(),
            fs_rights_base: self.rights.get(),
            fs_rights_inheriting: self.rights_inheriting.get(),
        })
    }

    pub(super) fn fdstat_set_flags(&self, flags: Fdflags) -> WasiResult<()> {
        self.check_rights(Rights::FD_FDSTAT_SET_FLAGS)?;

        self.flags.set(flags);
        Ok(())
    }

    pub(super) fn fdstat_set_rights(
        &self,
        rights: Rights,
        rights_inheriting: Rights,
    ) -> WasiResult<()> {
        loop {
            let old_rights = self.rights.get();
            let old_rights_inheriting = self.rights_inheriting.get();

            if !old_rights.contains(rights) || !old_rights_inheriting.contains(rights_inheriting) {
                break Err(Errno::Notcapable);
            }

            if self.rights.compare_and_swap(old_rights, rights)
                && self
                    .rights_inheriting
                    .compare_and_swap(old_rights_inheriting, rights_inheriting)
            {
                break Ok(());
            }
        }
    }

    pub(super) fn filestat_get(&self) -> WasiResult<Filestat> {
        self.check_rights(Rights::FD_FILESTAT_GET)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn filestat_set_size(&self, size: Filesize) -> WasiResult<()> {
        self.check_rights(Rights::FD_FILESTAT_SET_SIZE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn filestat_set_times(
        &self,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()> {
        self.check_rights(Rights::FD_FILESTAT_SET_TIMES)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn pread(&self, iovs: &mut [IoSliceMut<'_>], offset: Filesize) -> WasiResult<Size> {
        self.check_rights(Rights::FD_READ | Rights::FD_SEEK)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn prestat_get(&self) -> WasiResult<Prestat> {
        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn prestat_dir_name(&self) -> WasiResult<S::Owned> {
        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn pwrite(&self, bufs: &[IoSlice<'_>], offset: Filesize) -> WasiResult<Size> {
        self.check_rights(Rights::FD_WRITE | Rights::FD_SEEK)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn read(&self, iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size> {
        self.check_rights(Rights::FD_READ)?;

        match self.inner {
            WasiFdInner::CharacterDevice(ref d) => d.read(iovs),
        }
    }

    pub(super) fn readdir(&self, cookie: Dircookie) -> WasiResult<Option<(Dirent, S::Owned)>> {
        self.check_rights(Rights::FD_READDIR)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn seek(&self, offset: Filedelta, whence: Whence) -> WasiResult<Filesize> {
        {
            let rights = self.rights.get();

            if let Err(err) = Self::check_rights_with(rights, Rights::FD_SEEK) {
                if whence == Whence::Cur && offset.0 == 0 {
                    Self::check_rights_with(rights, Rights::FD_TELL)?;
                } else {
                    return Err(err);
                }
            };
        }

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn sync(&self) -> WasiResult<()> {
        self.check_rights(Rights::FD_SYNC)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn tell(&self) -> WasiResult<Filesize> {
        {
            let rights = self.rights.get();

            if let Err(_) = Self::check_rights_with(rights, Rights::FD_TELL) {
                Self::check_rights_with(rights, Rights::FD_SEEK)?;
            }
        }

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn write(&self, bufs: &[IoSlice<'_>]) -> WasiResult<Size> {
        self.check_rights(Rights::FD_WRITE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(ref d) => d.write(bufs),
        }
    }

    pub(super) fn path_create_directory(&self, path: &S::Borrowed) -> WasiResult<()> {
        self.check_rights(Rights::PATH_CREATE_DIRECTORY)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_filestat_get(
        &self,
        flags: Lookupflags,
        path: &S::Borrowed,
    ) -> WasiResult<Filestat> {
        self.check_rights(Rights::PATH_FILESTAT_GET)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_filestat_set_times(
        &self,
        flags: Lookupflags,
        path: &S::Borrowed,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()> {
        self.check_rights(Rights::PATH_FILESTAT_SET_TIMES)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_open(
        &self,
        dirflags: Lookupflags,
        path: &S::Borrowed,
        oflags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fdflags: Fdflags,
    ) -> WasiResult<Self> {
        {
            let rights = self.rights.get();

            Self::check_rights_with(rights, Rights::PATH_OPEN)?;
            if fdflags.contains(Fdflags::DSYNC) {
                if let Err(_) = Self::check_rights_with(rights, Rights::FD_DATASYNC) {
                    Self::check_rights_with(rights, Rights::FD_SYNC)?;
                }
            }
            if fdflags.contains(Fdflags::RSYNC) {
                Self::check_rights_with(rights, Rights::FD_SYNC)?;
            }
            if oflags.contains(Oflags::CREAT) {
                Self::check_rights_with(rights, Rights::PATH_CREATE_FILE)?;
            }
            if oflags.contains(Oflags::TRUNC) {
                Self::check_rights_with(rights, Rights::PATH_FILESTAT_SET_SIZE)?;
            }
        }

        {
            let rights_inheriting = self.rights_inheriting.get();

            Self::check_rights_with(rights_inheriting, fs_rights_base)?;
            Self::check_rights_with(rights_inheriting, fs_rights_inheriting)?;
        }

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_link(
        &self,
        old_flags: Lookupflags,
        old_path: &S::Borrowed,
        new_fd: &Self,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.check_rights(Rights::PATH_LINK_SOURCE)?;
        new_fd.check_rights(Rights::PATH_LINK_TARGET)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_readlink(&self, path: &S::Borrowed) -> WasiResult<S::Owned> {
        self.check_rights(Rights::PATH_READLINK)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_remove_directory(&self, path: &S::Borrowed) -> WasiResult<()> {
        self.check_rights(Rights::PATH_REMOVE_DIRECTORY)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_rename(
        &self,
        old_path: &S::Borrowed,
        new_fd: &Self,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.check_rights(Rights::PATH_RENAME_SOURCE)?;
        new_fd.check_rights(Rights::PATH_RENAME_TARGET)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_symlink(
        &self,
        old_path: &S::Borrowed,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.check_rights(Rights::PATH_SYMLINK)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn path_unlink_file(&self, path: &S::Borrowed) -> WasiResult<()> {
        self.check_rights(Rights::PATH_UNLINK_FILE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn sock_recv(
        &self,
        ri_data: &mut [IoSliceMut<'_>],
        ri_flags: Riflags,
    ) -> WasiResult<(Size, Roflags)> {
        self.check_rights(Rights::FD_READ)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn sock_send(&self, si_data: &[IoSlice<'_>], si_flags: Siflags) -> WasiResult<Size> {
        self.check_rights(Rights::FD_WRITE)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }

    pub(super) fn sock_shutdown(&self, how: Sdflags) -> WasiResult<()> {
        self.check_rights(Rights::SOCK_SHUTDOWN)?;

        match self.inner {
            WasiFdInner::CharacterDevice(_) => Err(Errno::Notsup),
        }
    }
}
