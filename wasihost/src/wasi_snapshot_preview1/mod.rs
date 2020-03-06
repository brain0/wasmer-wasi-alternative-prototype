//! High-level abstraction for executing binaries conforming to WASI snapshot preview 1.

mod atomic;
mod wasi_fd;

use parking_lot::Mutex;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::{
    collections::hash_map::{Entry, HashMap},
    error::Error,
    fs::File,
    io::{IoSlice, IoSliceMut, Read},
    marker::PhantomData,
    mem,
    path::Path,
    sync::Arc,
};
use wasihost_core::{string_representation::StringRepresentation, wasi_snapshot_preview1::*};
use wasmer_runtime::{instantiate, Func};

pub use self::wasi_fd::{CharacterDevice, Stderr, Stdin, Stdout, WasiFd};

/// Host functions for WASI.
#[derive(Debug)]
pub struct WasiHost<S: StringRepresentation> {
    arguments: Vec<S::Owned>,
    environment: Vec<S::Owned>,
    fds: Mutex<HashMap<Fd, Arc<WasiFd<S>>>>,
    fd_distribution: Uniform<u32>,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S: StringRepresentation> WasiHost<S> {
    /// Creates a new WASI host.
    pub fn new(
        arguments: impl IntoIterator<Item = impl Into<S::Owned>>,
        environment: impl IntoIterator<Item = impl Into<S::Owned>>,
        fd_initialzer: impl WasiFdInitializer<S>,
    ) -> Arc<Self> {
        let arguments = arguments.into_iter().map(|s| s.into()).collect();
        let environment = environment.into_iter().map(|s| s.into()).collect();
        let fds = Mutex::new(
            fd_initialzer
                .initialize()
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect(),
        );
        let fd_distribution = (0..2u32.pow(31)).into();

        Arc::new(WasiHost {
            arguments,
            environment,
            fds,
            fd_distribution,
            _phantom: PhantomData,
        })
    }

    /// Runs a WASM file on this WASI host.
    pub fn run_file(
        self: Arc<Self>,
        wasm_file: impl AsRef<Path>,
    ) -> Result<native::exitcode, Box<dyn Error>> {
        let mut wasm_binary = Vec::new();

        {
            let mut file = File::open(&wasm_file)?;
            file.read_to_end(&mut wasm_binary)?;
        }

        self.run_binary(&wasm_binary[..])
    }

    /// Runs a WASM binary from memory on this WASI host.
    pub fn run_binary(
        self: Arc<Self>,
        wasm_binary: &[u8],
    ) -> Result<native::exitcode, Box<dyn Error>> {
        let instance = {
            let import_object = self.into_imports();
            instantiate(&wasm_binary, &import_object)?
        };

        let start: Func<'_, ()> = instance.func("_start")?;

        Ok(match start.call() {
            Ok(()) => 0,
            Err(e) => match e.0.downcast_ref::<native::exitcode>() {
                Some(&code) => code,
                None => Err(e)?,
            },
        })
    }

    fn with_fd<R>(&self, fd: Fd, f: impl FnOnce(&WasiFd<S>) -> WasiResult<R>) -> WasiResult<R> {
        let fd = {
            let fds = self.fds.lock();
            fds.get(&fd).map(|fd| fd.clone()).ok_or(Errno::Badf)
        };
        fd.and_then(|fd| f(&fd))
    }

    fn with_fds<R>(
        &self,
        fd1: Fd,
        fd2: Fd,
        f: impl FnOnce(&WasiFd<S>, &WasiFd<S>) -> WasiResult<R>,
    ) -> WasiResult<R> {
        let fds = {
            let fds = self.fds.lock();
            fds.get(&fd1).ok_or(Errno::Badf).and_then(|fd1| {
                fds.get(&fd2)
                    .map(|fd2| (fd1.clone(), fd2.clone()))
                    .ok_or(Errno::Badf)
            })
        };
        fds.and_then(|fds| f(&fds.0, &fds.1))
    }

    fn allocate_fd(&self, fd: WasiFd<S>) -> WasiResult<Fd> {
        let mut rng = thread_rng();
        let mut fds = self.fds.lock();

        // TODO: If there are too many open file descriptors, this is likely to take long.
        loop {
            let new_fd = Fd(rng.sample(self.fd_distribution));

            if let Entry::Vacant(entry) = fds.entry(new_fd) {
                entry.insert(Arc::new(fd));
                return Ok(new_fd);
            }
        }
    }
}

impl<S: StringRepresentation> WasiImports for WasiHost<S> {
    type StringRepresentation = S;

    fn args_get(&self) -> WasiResult<&[S::Owned]> {
        Ok(&self.arguments[..])
    }

    fn environ_get(&self) -> WasiResult<&[S::Owned]> {
        Ok(&self.environment[..])
    }

    fn clock_res_get(&self, _: Clockid) -> WasiResult<Timestamp> {
        todo!("clock_res_get")
    }

    fn clock_time_get(&self, _: Clockid, _: Timestamp) -> WasiResult<Timestamp> {
        todo!("clock_time_get")
    }

    fn fd_advise(&self, fd: Fd, offset: Filesize, len: Filesize, advice: Advice) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.advise(offset, len, advice))
    }

    fn fd_allocate(&self, fd: Fd, offset: Filesize, len: Filesize) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.allocate(offset, len))
    }

    fn fd_close(&self, fd: Fd) -> WasiResult<()> {
        let fd = {
            let mut fds = self.fds.lock();

            match fds.remove(&fd) {
                Some(fd) => fd,
                None => return Err(Errno::Badf),
            }
        };

        // Always drop the fd outside of the lock.
        drop(fd);

        Ok(())
    }

    fn fd_datasync(&self, fd: Fd) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.datasync())
    }

    fn fd_fdstat_get(&self, fd: Fd) -> WasiResult<Fdstat> {
        self.with_fd(fd, |fd| fd.fdstat_get())
    }

    fn fd_fdstat_set_flags(&self, fd: Fd, flags: Fdflags) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.fdstat_set_flags(flags))
    }

    fn fd_fdstat_set_rights(
        &self,
        fd: Fd,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
    ) -> WasiResult<()> {
        self.with_fd(fd, |fd| {
            fd.fdstat_set_rights(fs_rights_base, fs_rights_inheriting)
        })
    }

    fn fd_filestat_get(&self, fd: Fd) -> WasiResult<Filestat> {
        self.with_fd(fd, |fd| fd.filestat_get())
    }

    fn fd_filestat_set_size(&self, fd: Fd, size: Filesize) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.filestat_set_size(size))
    }

    fn fd_filestat_set_times(
        &self,
        fd: Fd,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.filestat_set_times(atim, mtim, fst_flags))
    }

    fn fd_pread(&self, fd: Fd, iovs: &mut [IoSliceMut<'_>], offset: Filesize) -> WasiResult<Size> {
        self.with_fd(fd, |fd| fd.pread(iovs, offset))
    }

    fn fd_prestat_get(&self, fd: Fd) -> WasiResult<Prestat> {
        self.with_fd(fd, |fd| fd.prestat_get())
    }

    fn fd_prestat_dir_name(&self, fd: Fd) -> WasiResult<S::Owned> {
        self.with_fd(fd, |fd| fd.prestat_dir_name())
    }

    fn fd_pwrite(&self, fd: Fd, bufs: &[IoSlice<'_>], offset: Filesize) -> WasiResult<Size> {
        self.with_fd(fd, |fd| fd.pwrite(bufs, offset))
    }

    fn fd_read(&self, fd: Fd, iovs: &mut [IoSliceMut<'_>]) -> WasiResult<Size> {
        self.with_fd(fd, |fd| fd.read(iovs))
    }

    fn fd_readdir(&self, fd: Fd, cookie: Dircookie) -> WasiResult<Option<(Dirent, S::Owned)>> {
        self.with_fd(fd, |fd| fd.readdir(cookie))
    }

    fn fd_renumber(&self, fd: Fd, to: Fd) -> WasiResult<()> {
        if fd != to {
            let old_to = {
                let mut fds = self.fds.lock();

                let fd = fds.get(&fd).map(|fd| fd.clone()).ok_or(Errno::Badf)?;
                let to = fds.get_mut(&to).ok_or(Errno::Badf)?;

                mem::replace(to, fd)
            };

            // Always drop the fd outside of the lock.
            drop(old_to);
        }

        Ok(())
    }

    fn fd_seek(&self, fd: Fd, offset: Filedelta, whence: Whence) -> WasiResult<Filesize> {
        self.with_fd(fd, |fd| fd.seek(offset, whence))
    }

    fn fd_sync(&self, fd: Fd) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.sync())
    }

    fn fd_tell(&self, fd: Fd) -> WasiResult<Filesize> {
        self.with_fd(fd, |fd| fd.tell())
    }

    fn fd_write(&self, fd: Fd, bufs: &[IoSlice<'_>]) -> WasiResult<Size> {
        self.with_fd(fd, |fd| fd.write(bufs))
    }

    fn path_create_directory(&self, fd: Fd, path: &S::Borrowed) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.path_create_directory(path))
    }

    fn path_filestat_get(
        &self,
        fd: Fd,
        flags: Lookupflags,
        path: &S::Borrowed,
    ) -> WasiResult<Filestat> {
        self.with_fd(fd, |fd| fd.path_filestat_get(flags, path))
    }

    fn path_filestat_set_times(
        &self,
        fd: Fd,
        flags: Lookupflags,
        path: &S::Borrowed,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()> {
        self.with_fd(fd, |fd| {
            fd.path_filestat_set_times(flags, path, atim, mtim, fst_flags)
        })
    }

    fn path_open(
        &self,
        fd: Fd,
        dirflags: Lookupflags,
        path: &S::Borrowed,
        oflags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fdflags: Fdflags,
    ) -> WasiResult<Fd> {
        let newfd = self.with_fd(fd, |fd| {
            fd.path_open(
                dirflags,
                path,
                oflags,
                fs_rights_base,
                fs_rights_inheriting,
                fdflags,
            )
        })?;

        self.allocate_fd(newfd)
    }

    fn path_link(
        &self,
        old_fd: Fd,
        old_flags: Lookupflags,
        old_path: &S::Borrowed,
        new_fd: Fd,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.with_fds(old_fd, new_fd, |old_fd, new_fd| {
            old_fd.path_link(old_flags, old_path, new_fd, new_path)
        })
    }

    fn path_readlink(&self, fd: Fd, path: &S::Borrowed) -> WasiResult<S::Owned> {
        self.with_fd(fd, |fd| fd.path_readlink(path))
    }

    fn path_remove_directory(&self, fd: Fd, path: &S::Borrowed) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.path_remove_directory(path))
    }

    fn path_rename(
        &self,
        fd: Fd,
        old_path: &S::Borrowed,
        new_fd: Fd,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.with_fds(fd, new_fd, |fd, new_fd| {
            fd.path_rename(old_path, new_fd, new_path)
        })
    }

    fn path_symlink(
        &self,
        old_path: &S::Borrowed,
        fd: Fd,
        new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.path_symlink(old_path, new_path))
    }

    fn path_unlink_file(&self, fd: Fd, path: &S::Borrowed) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.path_unlink_file(path))
    }

    fn poll_oneoff(&self, _subscriptions: &[Subscription]) -> WasiResult<Vec<Event>> {
        Err(Errno::Nosys)
    }

    fn proc_exit(&self, c: Exitcode) -> Result<std::convert::Infallible, Exitcode> {
        Err(c)
    }

    fn proc_raise(&self, _: Signal) -> WasiResult<()> {
        Err(Errno::Nosys)
    }

    fn random_get(&self, buf: &mut [u8]) -> WasiResult<()> {
        getrandom::getrandom(buf).map_err(|_| Errno::Io)
    }

    fn sched_yield(&self) -> WasiResult<()> {
        std::thread::yield_now();
        Ok(())
    }

    fn sock_recv(
        &self,
        fd: Fd,
        ri_data: &mut [IoSliceMut<'_>],
        ri_flags: Riflags,
    ) -> WasiResult<(Size, Roflags)> {
        self.with_fd(fd, |fd| fd.sock_recv(ri_data, ri_flags))
    }

    fn sock_send(&self, fd: Fd, si_data: &[IoSlice<'_>], si_flags: Siflags) -> WasiResult<Size> {
        self.with_fd(fd, |fd| fd.sock_send(si_data, si_flags))
    }

    fn sock_shutdown(&self, fd: Fd, how: Sdflags) -> WasiResult<()> {
        self.with_fd(fd, |fd| fd.sock_shutdown(how))
    }
}

/// Defines how the file descriptor map is initialized.
pub trait WasiFdInitializer<S> {
    /// Initializes the file descriptor map.
    fn initialize(self) -> HashMap<Fd, WasiFd<S>>;
}

/// Default initializer for the file descriptor map. Initialized
/// standard input, output and error.
#[derive(Debug, Default)]
pub struct DefaultWasiFdInitializer;

impl<S: StringRepresentation> WasiFdInitializer<S> for DefaultWasiFdInitializer {
    fn initialize(self) -> HashMap<Fd, WasiFd<S>> {
        let mut fds = HashMap::with_capacity(3);

        let flags = Fdflags::empty();
        let rights = Rights::all();

        fds.insert(Fd(0), WasiFd::from_character_device(Stdin, flags, rights));
        fds.insert(Fd(1), WasiFd::from_character_device(Stdout, flags, rights));
        fds.insert(Fd(2), WasiFd::from_character_device(Stderr, flags, rights));

        fds
    }
}
