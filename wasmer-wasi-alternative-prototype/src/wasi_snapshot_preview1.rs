#![allow(unused_variables)]
#![allow(missing_docs)]

//! Implementation of types and interfaces for WASI snashot 1.
//!
//! This module is incomplete and lacks documentation.

use self::native::{NativeWasiImports, NativeWasiImportsExt};
use std::sync::Arc;
use witx_gen::{
    reexports::{Ctx, ImportObject, Memory},
    witx_gen, WasiValue, WasmSlicePtr,
};

witx_gen!("wasi_snapshot_preview1" => "WASI/phases/snapshot/witx/wasi_snapshot_preview1.witx");
//include!("../../macro_debug.rs");

pub type WasiResult<T> = Result<T, Errno>;

pub trait WasiImports: Send + Sync + 'static {
    fn args_get(&self) -> WasiResult<&[String]>;
    fn environ_get(&self) -> WasiResult<&[String]>;
    fn clock_res_get(&self, id: Clockid) -> WasiResult<Timestamp>;
    fn clock_time_get(&self, id: Clockid, precision: Timestamp) -> WasiResult<Timestamp>;
    fn fd_advise(&self, fd: Fd, offset: Filesize, len: Filesize, advice: Advice) -> WasiResult<()>;
    fn fd_allocate(&self, fd: Fd, offset: Filesize, len: Filesize) -> WasiResult<()>;
    fn fd_close(&self, fd: Fd) -> WasiResult<()>;
    fn fd_datasync(&self, fd: Fd) -> WasiResult<()>;
    fn fd_fdstat_get(&self, fd: Fd) -> WasiResult<Fdstat>;
    fn fd_fdstat_set_flags(&self, fd: Fd, flags: Fdflags) -> WasiResult<()>;
    fn fd_fdstat_set_rights(
        &self,
        fd: Fd,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
    ) -> WasiResult<()>;
    fn fd_filestat_get(&self, fd: Fd) -> WasiResult<Filestat>;
    fn fd_filestat_set_size(&self, fd: Fd, size: Filesize) -> WasiResult<()>;
    fn fd_filestat_set_times(
        &self,
        fd: Fd,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()>;
    fn fd_pread(&self, fd: Fd, iovs: &[&mut [u8]], offset: Filesize) -> WasiResult<Size>;
    fn fd_prestat_get(&self, fd: Fd) -> WasiResult<Prestat>;
    fn fd_prestat_dir_name(&self, fd: Fd) -> WasiResult<String>;
    fn fd_read(&self, fd: Fd, iovs: &[&mut [u8]]) -> WasiResult<Size>;
    fn fd_renumber(&self, fd: Fd, to: Fd) -> WasiResult<()>;
    fn fd_seek(&self, fd: Fd, offset: Filedelta, whence: Whence) -> WasiResult<Filesize>;
    fn fd_sync(&self, fd: Fd) -> WasiResult<()>;
    fn fd_tell(&self, fd: Fd) -> WasiResult<Filesize>;
    fn proc_exit(&self, rval: Exitcode) -> Result<std::convert::Infallible, Exitcode>;
    fn proc_raise(&self, sig: Signal) -> WasiResult<()>;
    fn sched_yield(&self) -> WasiResult<()>;
    fn sock_shutdown(&self, fd: Fd, how: Sdflags) -> WasiResult<()>;
}

pub trait WasiImportsExt {
    fn into_imports(self) -> ImportObject;
}

impl<T: WasiImports> WasiImportsExt for T {
    fn into_imports(self) -> ImportObject {
        Arc::new(self).into_imports()
    }
}

impl<T: WasiImports> WasiImportsExt for Arc<T> {
    fn into_imports(self) -> ImportObject {
        NativeWasiAdapter(self).into_imports()
    }
}

struct NativeWasiAdapter<T>(Arc<T>);

impl<T> Clone for NativeWasiAdapter<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> NativeWasiAdapter<T> {
    fn fill_bufs(
        strings: WasiResult<&[String]>,
        memory: &Memory,
        ptrs: WasmSlicePtr<WasmSlicePtr<u8>>,
        buf: WasmSlicePtr<u8>,
    ) -> native::errno {
        match strings {
            Ok(strings) => {
                let mut index = 0;
                let ptrs = ptrs.with(memory, strings.len() as u32);
                for (i, s) in strings.iter().enumerate() {
                    let buf = buf.add(index);
                    ptrs.write(i as u32, buf);

                    let s = s.as_bytes();
                    let len = s.len() as u32;
                    let buf = buf.with(memory, len + 1);

                    for (i, b) in s.iter().copied().enumerate() {
                        buf.write(i as u32, b);
                    }
                    buf.write(len, 0);

                    index += len + 1;
                }

                native::errno_success
            }
            Err(err) => err.to_native(),
        }
    }

    fn to_sizes(strings: WasiResult<&[String]>) -> (native::errno, native::size, native::size) {
        match strings {
            Ok(strings) => (
                native::errno_success,
                strings.len() as u32,
                strings.iter().map(|s| s.as_bytes().len() as u32 + 1).sum(),
            ),
            Err(err) => (err.to_native(), 0, 0),
        }
    }

    fn write_to_bufs<F: FnOnce(&[&mut [u8]]) -> R, S: FnOnce(&R) -> native::size, R>(
        memory: &Memory,
        iovs: WasmSlicePtr<native::iovec>,
        iovs_len: native::size,
        f: F,
        s: S,
    ) -> R {
        let iovecs: Vec<_> = {
            let iovs = iovs.with(memory, iovs_len);
            (0..iovs_len).map(|i| iovs.read(i)).collect()
        };

        let mut bufs: Vec<_> = iovecs
            .iter()
            .map(|s| vec![0u8; s.buf_len as usize])
            .collect();
        let buf_slices: Vec<_> = bufs.iter_mut().map(|v| &mut v[..]).collect();

        let result = f(&buf_slices[..]);
        let size = s(&result);

        if size > 0 {
            let mut read = 0;
            'outer: for i in 0..iovs_len {
                let cur_wasm = iovecs[i as usize];
                let cur_native = &buf_slices[i as usize];

                let cur_wasm_slice = cur_wasm.buf.with(memory, cur_wasm.buf_len);

                for j in 0..cur_wasm.buf_len {
                    if read >= size {
                        break 'outer;
                    }

                    cur_wasm_slice.write(j, cur_native[j as usize]);
                    read += 1;
                }
            }
        }

        result
    }

    fn write_to_buf<F: FnOnce(&mut [u8]) -> R, S: FnOnce(&R) -> native::size, R>(
        memory: &Memory,
        wasm_buf: WasmSlicePtr<u8>,
        wasm_buf_len: native::size,
        f: F,
        s: S,
    ) -> R {
        let mut buf = vec![0u8; wasm_buf_len as usize];

        let result = f(&mut buf[..]);
        let size = s(&result);

        let wasm_buf = wasm_buf.with(memory, wasm_buf_len);

        if size > 0 {
            let mut read = 0;
            for i in 0..wasm_buf_len {
                if read >= size {
                    break;
                }

                wasm_buf.write(i, buf[i as usize]);
                read += 1;
            }
        }

        result
    }
}

macro_rules! try0 {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(_) => return native::errno_inval,
        }
    };
}

macro_rules! to_result0 {
    ($e:expr) => {
        match $e {
            Ok(()) => (native::errno_success),
            Err(err) => err.to_native(),
        }
    };
}

macro_rules! try1 {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(_) => return (native::errno_inval, Default::default()),
        }
    };
}

macro_rules! to_result1 {
    ($e:expr) => {
        match $e {
            Ok(val) => (native::errno_success, val.to_native()),
            Err(err) => (err.to_native(), Default::default()),
        }
    };
}

impl<T: WasiImports> NativeWasiImports for NativeWasiAdapter<T> {
    fn args_get(
        &self,
        ctx: &mut Ctx,
        argv: WasmSlicePtr<WasmSlicePtr<u8>>,
        argv_buf: WasmSlicePtr<u8>,
    ) -> native::errno {
        Self::fill_bufs(self.0.args_get(), ctx.memory(0), argv, argv_buf)
    }

    fn args_sizes_get(&self, _ctx: &mut Ctx) -> (native::errno, native::size, native::size) {
        Self::to_sizes(self.0.args_get())
    }

    fn environ_get(
        &self,
        ctx: &mut Ctx,
        environ: WasmSlicePtr<WasmSlicePtr<u8>>,
        environ_buf: WasmSlicePtr<u8>,
    ) -> native::errno {
        Self::fill_bufs(self.0.environ_get(), ctx.memory(0), environ, environ_buf)
    }

    fn environ_sizes_get(&self, _ctx: &mut Ctx) -> (native::errno, native::size, native::size) {
        Self::to_sizes(self.0.environ_get())
    }

    fn clock_res_get(
        &self,
        _ctx: &mut Ctx,
        id: native::clockid,
    ) -> (native::errno, native::timestamp) {
        let id = try1!(Clockid::from_native(id));

        to_result1!(self.0.clock_res_get(id))
    }

    fn clock_time_get(
        &self,
        _ctx: &mut Ctx,
        id: native::clockid,
        precision: native::timestamp,
    ) -> (native::errno, native::timestamp) {
        let id = try1!(Clockid::from_native(id));
        let precision = try1!(Timestamp::from_native(precision));

        to_result1!(self.0.clock_time_get(id, precision))
    }

    fn fd_advise(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        offset: native::filesize,
        len: native::filesize,
        advice: native::advice,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let offset = try0!(Filesize::from_native(offset));
        let len = try0!(Filesize::from_native(len));
        let advice = try0!(Advice::from_native(advice));

        to_result0!(self.0.fd_advise(fd, offset, len, advice))
    }

    fn fd_allocate(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        offset: native::filesize,
        len: native::filesize,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let offset = try0!(Filesize::from_native(offset));
        let len = try0!(Filesize::from_native(len));

        to_result0!(self.0.fd_allocate(fd, offset, len))
    }

    fn fd_close(&self, _ctx: &mut Ctx, fd: native::fd) -> native::errno {
        let fd = try0!(Fd::from_native(fd));

        to_result0!(self.0.fd_close(fd))
    }

    fn fd_datasync(&self, _ctx: &mut Ctx, fd: native::fd) -> native::errno {
        let fd = try0!(Fd::from_native(fd));

        to_result0!(self.0.fd_datasync(fd))
    }

    fn fd_fdstat_get(&self, _ctx: &mut Ctx, fd: native::fd) -> (native::errno, native::fdstat) {
        let fd = try1!(Fd::from_native(fd));

        to_result1!(self.0.fd_fdstat_get(fd))
    }

    fn fd_fdstat_set_flags(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        flags: native::fdflags,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let flags = try0!(Fdflags::from_native(flags));

        to_result0!(self.0.fd_fdstat_set_flags(fd, flags))
    }

    fn fd_fdstat_set_rights(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        fs_rights_base: native::rights,
        fs_rights_inheriting: native::rights,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let fs_rights_base = try0!(Rights::from_native(fs_rights_base));
        let fs_rights_inheriting = try0!(Rights::from_native(fs_rights_inheriting));

        to_result0!(self
            .0
            .fd_fdstat_set_rights(fd, fs_rights_base, fs_rights_inheriting))
    }

    fn fd_filestat_get(&self, _ctx: &mut Ctx, fd: native::fd) -> (native::errno, native::filestat) {
        let fd = try1!(Fd::from_native(fd));

        to_result1!(self.0.fd_filestat_get(fd))
    }

    fn fd_filestat_set_size(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        size: native::filesize,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let size = try0!(Filesize::from_native(size));

        to_result0!(self.0.fd_filestat_set_size(fd, size))
    }

    fn fd_filestat_set_times(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        atim: native::timestamp,
        mtim: native::timestamp,
        fst_flags: native::fstflags,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let atim = try0!(Timestamp::from_native(atim));
        let mtim = try0!(Timestamp::from_native(mtim));
        let fst_flags = try0!(Fstflags::from_native(fst_flags));

        to_result0!(self.0.fd_filestat_set_times(fd, atim, mtim, fst_flags))
    }

    fn fd_pread(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        iovs: WasmSlicePtr<native::iovec>,
        iovs_len: native::size,
        offset: native::filesize,
    ) -> (native::errno, native::size) {
        let fd = try1!(Fd::from_native(fd));
        let offset = try1!(Filesize::from_native(offset));

        Self::write_to_bufs(
            ctx.memory(0),
            iovs,
            iovs_len,
            |buf| to_result1!(self.0.fd_pread(fd, buf, offset)),
            |&(e, s)| if e == native::errno_success { s } else { 0 },
        )
    }

    fn fd_prestat_get(&self, _ctx: &mut Ctx, fd: native::fd) -> (native::errno, native::prestat) {
        let fd = try1!(Fd::from_native(fd));

        to_result1!(self.0.fd_prestat_get(fd))
    }

    fn fd_prestat_dir_name(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));

        let dirname = match self.0.fd_prestat_dir_name(fd) {
            Ok(s) => s,
            Err(e) => return e.to_native(),
        };
        let dirname = dirname.as_bytes();

        Self::write_to_buf(
            ctx.memory(0),
            path,
            path_len,
            |buf| {
                if dirname.len() > buf.len() {
                    (native::errno_overflow, 0)
                } else {
                    buf[..dirname.len()].copy_from_slice(dirname);
                    (native::errno_success, dirname.len() as u32)
                }
            },
            |s| s.1,
        )
        .0
    }

    fn fd_pwrite(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        iovs: WasmSlicePtr<native::ciovec>,
        iovs_len: native::size,
        offset: native::filesize,
    ) -> (native::errno, native::size) {
        todo!("fd_pwrite")
    }

    fn fd_read(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        iovs: WasmSlicePtr<native::iovec>,
        iovs_len: native::size,
    ) -> (native::errno, native::size) {
        let fd = try1!(Fd::from_native(fd));

        Self::write_to_bufs(
            ctx.memory(0),
            iovs,
            iovs_len,
            |buf| to_result1!(self.0.fd_read(fd, buf)),
            |&(e, s)| if e == native::errno_success { s } else { 0 },
        )
    }

    fn fd_readdir(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        buf: WasmSlicePtr<u8>,
        buf_len: native::size,
        cookie: native::dircookie,
    ) -> (native::errno, native::size) {
        todo!("fd_readdir")
    }

    fn fd_renumber(&self, _ctx: &mut Ctx, fd: native::fd, to: native::fd) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let to = try0!(Fd::from_native(to));

        to_result0!(self.0.fd_renumber(fd, to))
    }

    fn fd_seek(
        &self,
        _ctx: &mut Ctx,
        fd: native::fd,
        offset: native::filedelta,
        whence: native::whence,
    ) -> (native::errno, native::filesize) {
        let fd = try1!(Fd::from_native(fd));
        let offset = try1!(Filedelta::from_native(offset));
        let whence = try1!(Whence::from_native(whence));

        to_result1!(self.0.fd_seek(fd, offset, whence))
    }

    fn fd_sync(&self, _ctx: &mut Ctx, fd: native::fd) -> native::errno {
        let fd = try0!(Fd::from_native(fd));

        to_result0!(self.0.fd_sync(fd))
    }

    fn fd_tell(&self, _ctx: &mut Ctx, fd: native::fd) -> (native::errno, native::filesize) {
        let fd = try1!(Fd::from_native(fd));

        to_result1!(self.0.fd_tell(fd))
    }

    fn fd_write(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        iovs: WasmSlicePtr<native::ciovec>,
        iovs_len: native::size,
    ) -> (native::errno, native::size) {
        todo!("fd_write")
    }

    fn path_create_directory(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        todo!("path_create_directory")
    }

    fn path_filestat_get(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        flags: native::lookupflags,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> (native::errno, native::filestat) {
        todo!("path_filestat_get")
    }

    fn path_filestat_set_times(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        flags: native::lookupflags,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
        atim: native::timestamp,
        mtim: native::timestamp,
        fst_flags: native::fstflags,
    ) -> native::errno {
        todo!("path_filestat_set_times")
    }

    fn path_link(
        &self,
        ctx: &mut Ctx,
        old_fd: native::fd,
        old_flags: native::lookupflags,
        old_path: WasmSlicePtr<u8>,
        old_path_len: native::size,
        new_fd: native::fd,
        new_path: WasmSlicePtr<u8>,
        new_path_len: native::size,
    ) -> native::errno {
        todo!("path_link")
    }

    fn path_open(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        dirflags: native::lookupflags,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
        oflags: native::oflags,
        fs_rights_base: native::rights,
        fs_rights_inherting: native::rights,
        fdflags: native::fdflags,
    ) -> (native::errno, native::fd) {
        todo!("path_open")
    }

    fn path_readlink(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
        buf: WasmSlicePtr<u8>,
        buf_len: native::size,
    ) -> (native::errno, native::size) {
        todo!("path_readlink")
    }

    fn path_remove_directory(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        todo!("path_remove_directory")
    }

    fn path_rename(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        old_path: WasmSlicePtr<u8>,
        old_path_len: native::size,
        new_fd: native::fd,
        new_path: WasmSlicePtr<u8>,
        new_path_len: native::size,
    ) -> native::errno {
        todo!("path_rename")
    }

    fn path_symlink(
        &self,
        ctx: &mut Ctx,
        old_path: WasmSlicePtr<u8>,
        old_path_len: native::size,
        fd: native::fd,
        new_path: WasmSlicePtr<u8>,
        new_path_len: native::size,
    ) -> native::errno {
        todo!("path_symlink")
    }

    fn path_unlink_file(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        todo!("path_unlink_file")
    }

    fn poll_oneoff(
        &self,
        ctx: &mut Ctx,
        r#in: WasmSlicePtr<native::subscription>,
        out: WasmSlicePtr<native::event>,
        nsubscriptions: native::size,
    ) -> (native::errno, native::size) {
        todo!("poll_oneoff")
    }

    fn proc_exit(
        &self,
        _ctx: &mut Ctx,
        rval: native::exitcode,
    ) -> Result<std::convert::Infallible, native::exitcode> {
        self.0
            .proc_exit(Exitcode::from_native(rval).unwrap())
            .map_err(|e| e.0)
    }

    fn proc_raise(&self, _ctx: &mut Ctx, sig: native::signal) -> native::errno {
        let sig = try0!(Signal::from_native(sig));

        to_result0!(self.0.proc_raise(sig))
    }

    fn sched_yield(&self, _ctx: &mut Ctx) -> native::errno {
        to_result0!(self.0.sched_yield())
    }

    fn random_get(
        &self,
        ctx: &mut Ctx,
        buf: WasmSlicePtr<u8>,
        buf_len: native::size,
    ) -> native::errno {
        todo!("random_get")
    }

    fn sock_recv(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        ri_data: WasmSlicePtr<native::iovec>,
        ri_data_len: native::size,
        ri_flags: native::riflags,
    ) -> (native::errno, native::size, native::roflags) {
        todo!("sock_recv")
    }

    fn sock_send(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        si_data: WasmSlicePtr<native::ciovec>,
        si_data_len: native::size,
        si_flags: native::siflags,
    ) -> (native::errno, native::size) {
        todo!("sock_send")
    }

    fn sock_shutdown(&self, _ctx: &mut Ctx, fd: native::fd, how: native::sdflags) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let how = try0!(Sdflags::from_native(how));

        to_result0!(self.0.sock_shutdown(fd, how))
    }
}
