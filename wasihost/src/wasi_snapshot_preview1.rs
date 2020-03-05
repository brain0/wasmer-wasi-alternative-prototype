//! High-level abstraction for executing binaries conforming to WASI snapshot preview 1.

use std::{error::Error, fs::File, io::Read, marker::PhantomData, path::Path, sync::Arc};
use wasihost_core::{string_representation::StringRepresentation, wasi_snapshot_preview1::*};
use wasmer_runtime::{instantiate, Func};

/// Host functions for WASI.
#[derive(Debug)]
pub struct WasiHost<S: StringRepresentation> {
    arguments: Vec<S::Owned>,
    environment: Vec<S::Owned>,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S: StringRepresentation> WasiHost<S> {
    /// Creates a new WASI host.
    pub fn new(
        arguments: impl IntoIterator<Item = impl Into<S::Owned>>,
        environment: impl IntoIterator<Item = impl Into<S::Owned>>,
    ) -> Arc<Self> {
        let arguments = arguments.into_iter().map(|s| s.into()).collect();
        let environment = environment.into_iter().map(|s| s.into()).collect();

        Arc::new(WasiHost {
            arguments,
            environment,
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

    fn write_bufs<W: std::io::Write>(mut writer: W, bufs: &[&[u8]]) -> WasiResult<Size> {
        for buf in bufs {
            writer.write_all(buf).map_err(|_| Errno::Inval)?;
        }
        writer.flush().map_err(|_| Errno::Inval)?;

        Ok(Size(bufs.iter().map(|b| b.len()).sum::<usize>() as u32))
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
        unimplemented!("clock_res_get")
    }

    fn clock_time_get(&self, _: Clockid, _: Timestamp) -> WasiResult<Timestamp> {
        unimplemented!("clock_time_get")
    }

    fn fd_advise(&self, _: Fd, _: Filesize, _: Filesize, _: Advice) -> WasiResult<()> {
        unimplemented!("fd_advise")
    }

    fn fd_allocate(&self, _: Fd, _: Filesize, _: Filesize) -> WasiResult<()> {
        unimplemented!("fd_allocate")
    }

    fn fd_close(&self, _: Fd) -> WasiResult<()> {
        unimplemented!("fd_close")
    }

    fn fd_datasync(&self, _: Fd) -> WasiResult<()> {
        unimplemented!("fd_datasync")
    }

    fn fd_fdstat_get(&self, _: Fd) -> WasiResult<Fdstat> {
        unimplemented!("fd_fdstat_get")
    }

    fn fd_fdstat_set_flags(&self, _: Fd, _: Fdflags) -> WasiResult<()> {
        unimplemented!("fd_fdstat_set_flags")
    }

    fn fd_fdstat_set_rights(&self, _: Fd, _: Rights, _: Rights) -> WasiResult<()> {
        unimplemented!("fd_fdstat_set_rights")
    }

    fn fd_filestat_get(&self, _: Fd) -> WasiResult<Filestat> {
        unimplemented!("fd_filestat_get")
    }

    fn fd_filestat_set_size(&self, _: Fd, _: Filesize) -> WasiResult<()> {
        unimplemented!("fd_filestat_set_size")
    }

    fn fd_filestat_set_times(
        &self,
        _: Fd,
        _: Timestamp,
        _: Timestamp,
        _: Fstflags,
    ) -> WasiResult<()> {
        unimplemented!("fd_filestat_set_times")
    }

    fn fd_pread(&self, _: Fd, _: &[&mut [u8]], _: Filesize) -> WasiResult<Size> {
        unimplemented!("fd_pread")
    }

    fn fd_prestat_get(&self, _: Fd) -> WasiResult<Prestat> {
        Err(Errno::Badf)
    }

    fn fd_prestat_dir_name(&self, _: Fd) -> WasiResult<S::Owned> {
        unimplemented!("fd_prestat_dir_name")
    }

    fn fd_pwrite(&self, _: Fd, _: &[&[u8]], _: Filesize) -> WasiResult<Size> {
        unimplemented!("fd_pwrite")
    }

    fn fd_read(&self, _: Fd, _: &[&mut [u8]]) -> WasiResult<Size> {
        unimplemented!("fd_read")
    }

    fn fd_readdir(&self, _: Fd, _: Dircookie) -> WasiResult<Option<(Dirent, S::Owned)>> {
        unimplemented!("fd_readdir")
    }

    fn fd_renumber(&self, _: Fd, _: Fd) -> WasiResult<()> {
        unimplemented!("fd_renumber")
    }

    fn fd_seek(&self, _: Fd, _: Filedelta, _: Whence) -> WasiResult<Filesize> {
        unimplemented!("fd_seek")
    }

    fn fd_sync(&self, _: Fd) -> WasiResult<()> {
        unimplemented!("fd_sync")
    }

    fn fd_tell(&self, _: Fd) -> WasiResult<Filesize> {
        unimplemented!("fd_tell")
    }

    fn fd_write(&self, fd: Fd, bufs: &[&[u8]]) -> WasiResult<Size> {
        match fd.0 {
            1 => Self::write_bufs(std::io::stdout(), bufs),
            2 => Self::write_bufs(std::io::stderr(), bufs),
            _ => Err(Errno::Badf)?,
        }
    }

    fn path_create_directory(&self, _fd: Fd, _path: &S::Borrowed) -> WasiResult<()> {
        todo!("path_create_directory")
    }

    fn path_filestat_get(
        &self,
        _fd: Fd,
        _flags: Lookupflags,
        _path: &S::Borrowed,
    ) -> WasiResult<Filestat> {
        todo!("path_filestat_get")
    }

    fn path_filestat_set_times(
        &self,
        _fd: Fd,
        _flags: Lookupflags,
        _path: &S::Borrowed,
        _atim: Timestamp,
        _mtim: Timestamp,
        _fst_flags: Fstflags,
    ) -> WasiResult<()> {
        todo!("path_filestat_set_times")
    }

    fn path_open(
        &self,
        _fd: Fd,
        _dirflags: Lookupflags,
        _path: &S::Borrowed,
        _oflags: Oflags,
        _fs_rights_base: Rights,
        _fs_rights_inheriting: Rights,
        _fdflags: Fdflags,
    ) -> WasiResult<Fd> {
        todo!("path_open")
    }

    fn path_link(
        &self,
        _old_fd: Fd,
        _old_flags: Lookupflags,
        _old_path: &S::Borrowed,
        _new_fd: Fd,
        _new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        todo!("path_link")
    }

    fn path_readlink(&self, _fd: Fd, _path: &S::Borrowed) -> WasiResult<S::Owned> {
        todo!("path_readlink")
    }

    fn path_remove_directory(&self, _fd: Fd, _path: &S::Borrowed) -> WasiResult<()> {
        todo!("path_remove_directory")
    }

    fn path_rename(
        &self,
        _fd: Fd,
        _old_path: &S::Borrowed,
        _new_fd: Fd,
        _new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        todo!("path_rename")
    }

    fn path_symlink(
        &self,
        _old_path: &S::Borrowed,
        _fd: Fd,
        _new_path: &S::Borrowed,
    ) -> WasiResult<()> {
        todo!("path_symlink")
    }

    fn path_unlink_file(&self, _fd: Fd, _path: &S::Borrowed) -> WasiResult<()> {
        todo!("path_unlink_file")
    }

    fn poll_oneoff(&self, _subscriptions: &[Subscription]) -> WasiResult<Vec<Event>> {
        todo!("poll_oneoff")
    }

    fn proc_exit(&self, c: Exitcode) -> Result<std::convert::Infallible, Exitcode> {
        Err(c)
    }

    fn proc_raise(&self, _: Signal) -> WasiResult<()> {
        unimplemented!("proc_raise")
    }

    fn random_get(&self, buf: &mut [u8]) -> WasiResult<()> {
        getrandom::getrandom(buf).map_err(|_| Errno::Io)
    }

    fn sched_yield(&self) -> WasiResult<()> {
        unimplemented!("sched_yield")
    }

    fn sock_recv(
        &self,
        _fd: Fd,
        _ri_data: &[&mut [u8]],
        _ri_flags: Riflags,
    ) -> WasiResult<(Size, Roflags)> {
        todo!("sock_recv")
    }

    fn sock_send(&self, _fd: Fd, _si_data: &[&[u8]], _si_flags: Siflags) -> WasiResult<Size> {
        todo!("sock_send")
    }

    fn sock_shutdown(&self, _: Fd, _: Sdflags) -> WasiResult<()> {
        todo!("sock_shutdown")
    }
}
