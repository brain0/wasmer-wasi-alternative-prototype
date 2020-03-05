//! Implementation of types and interfaces for WASI snapshot 1.
//!
//! This module is incomplete and lacks documentation.

use self::native::{NativeWasiImports, NativeWasiImportsExt};
use std::{cell::Cell, cmp::min, sync::Arc};
use witx_gen::{
    reexports::{Ctx, ImportObject, Memory},
    witx_gen, WasiValue, WasmSlicePtr, WasmValue,
};

witx_gen!("wasi_snapshot_preview1" => "WASI/phases/snapshot/witx/wasi_snapshot_preview1.witx");

/// Result type for WASI methods.
pub type WasiResult<T> = Result<T, Errno>;

/// Functions necessary to satisfy the WASI specification.
pub trait WasiImports: Send + Sync + 'static {
    /// Gets the command-line parameters.
    fn args_get(&self) -> WasiResult<&[String]>;

    /// Gets the environment. Is is common convention that each string is of the form
    /// `key=value`.
    fn environ_get(&self) -> WasiResult<&[String]>;

    /// Return the resolution of a clock. Implementations are required to provide a non-zero value for
    /// supported clocks. For unsupported clocks, return `Err(Errno::Inval)`.
    ///
    /// Note: This is similar to `clock_getres` in POSIX.
    fn clock_res_get(&self, id: Clockid) -> WasiResult<Timestamp>;

    /// Return the time value of a clock.
    ///
    /// Note: This is similar to `clock_gettime` in POSIX.
    fn clock_time_get(&self, id: Clockid, precision: Timestamp) -> WasiResult<Timestamp>;

    /// Provide file advisory information on a file descriptor.
    ///
    /// Note: This is similar to `posix_fadvise` in POSIX.
    fn fd_advise(&self, fd: Fd, offset: Filesize, len: Filesize, advice: Advice) -> WasiResult<()>;

    /// Force the allocation of space in a file.
    ///
    /// Note: This is similar to `posix_fallocate` in POSIX.
    fn fd_allocate(&self, fd: Fd, offset: Filesize, len: Filesize) -> WasiResult<()>;

    /// Close a file descriptor.
    ///
    /// Note: This is similar to `close` in POSIX.
    fn fd_close(&self, fd: Fd) -> WasiResult<()>;

    /// Synchronize the data of a file to disk.
    ///
    /// Note: This is similar to `fdatasync` in POSIX.
    fn fd_datasync(&self, fd: Fd) -> WasiResult<()>;

    /// Get the attributes of a file descriptor.
    ///
    /// Note: This returns similar flags to `fsync(fd, F_GETFL)` in POSIX, as well as additional fields.
    fn fd_fdstat_get(&self, fd: Fd) -> WasiResult<Fdstat>;

    /// Adjust the flags associated with a file descriptor.
    ///
    /// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
    fn fd_fdstat_set_flags(&self, fd: Fd, flags: Fdflags) -> WasiResult<()>;

    /// Adjust the rights associated with a file descriptor. This can only be used to remove rights,
    /// and returns `Err(Errno::Notcapable)` if called in a way that would attempt to add rights.
    fn fd_fdstat_set_rights(
        &self,
        fd: Fd,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
    ) -> WasiResult<()>;

    /// Return the attributes of an open file.
    fn fd_filestat_get(&self, fd: Fd) -> WasiResult<Filestat>;

    /// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled
    /// with zeros.
    ///
    /// Note: This is similar to `ftruncate` in POSIX.
    fn fd_filestat_set_size(&self, fd: Fd, size: Filesize) -> WasiResult<()>;

    /// Adjust the timestamps of an open file or directory.
    ///
    /// Note: This is similar to `futimens` in POSIX.
    fn fd_filestat_set_times(
        &self,
        fd: Fd,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()>;

    /// Read from a file descriptor, without using and updating the file descriptor's offset.
    ///
    /// Note: This is similar to `preadv` in POSIX.
    fn fd_pread(&self, fd: Fd, iovs: &[&mut [u8]], offset: Filesize) -> WasiResult<Size>;

    /// Return a description of the given preopened file descriptor.
    fn fd_prestat_get(&self, fd: Fd) -> WasiResult<Prestat>;

    /// Return the directory name of the given preopened file descriptor.
    fn fd_prestat_dir_name(&self, fd: Fd) -> WasiResult<String>;

    /// Write to a file descriptor, without using and updating the file descriptor's offset.
    ///
    /// Note: This is similar to `pwritev` in POSIX.
    fn fd_pwrite(&self, fd: Fd, bufs: &[&[u8]], offset: Filesize) -> WasiResult<Size>;

    /// Read from a file descriptor.
    ///
    /// Note: This is similar to `readv` in POSIX.
    fn fd_read(&self, fd: Fd, iovs: &[&mut [u8]]) -> WasiResult<Size>;

    /// Read one directory entry from a directory.
    ///
    /// The cookie of first entry in a directory is always `Dircookie(0)`.
    fn fd_readdir(&self, fd: Fd, cookie: Dircookie) -> WasiResult<Option<(Dirent, String)>>;

    /// Atomically replace a file descriptor by renumbering another file descriptor. Due to the strong
    /// focus on thread safety, this environment does not provide a mechanism to duplicate or renumber
    /// a file descriptor to an arbitrary number, like dup2(). This would be prone to race conditions,
    /// as an actual file descriptor with the same number could be allocated by a different thread at
    /// the same time.
    fn fd_renumber(&self, fd: Fd, to: Fd) -> WasiResult<()>;

    /// Move the offset of a file descriptor.
    ///
    /// Note: This is similar to `lseek` in POSIX.
    fn fd_seek(&self, fd: Fd, offset: Filedelta, whence: Whence) -> WasiResult<Filesize>;

    /// Synchronize the data and metadata of a file to disk.
    ///
    /// Note: This is similar to `fsync` in POSIX.
    fn fd_sync(&self, fd: Fd) -> WasiResult<()>;

    /// Return the current offset of a file descriptor.
    ///
    /// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
    fn fd_tell(&self, fd: Fd) -> WasiResult<Filesize>;

    /// Write to a file descriptor.
    ///
    /// Note: This is similar to `writev` in POSIX.
    fn fd_write(&self, fd: Fd, bufs: &[&[u8]]) -> WasiResult<Size>;

    /// Create a directory.
    ///
    /// Note: This is similar to `mkdirat` in POSIX.
    fn path_create_directory(&self, fd: Fd, path: &str) -> WasiResult<()>;

    /// Return the attributes of a file or directory.
    ///
    /// Note: This is similar to `stat` in POSIX.
    fn path_filestat_get(&self, fd: Fd, flags: Lookupflags, path: &str) -> WasiResult<Filestat>;

    /// Adjust the timestamps of a file or directory.
    ///
    /// Note: This is similar to `utimensat` in POSIX.
    fn path_filestat_set_times(
        &self,
        fd: Fd,
        flags: Lookupflags,
        path: &str,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()>;

    /// Create a hard link.
    ///
    /// Note: This is similar to `linkat` in POSIX.
    fn path_link(
        &self,
        old_fd: Fd,
        old_flags: Lookupflags,
        old_path: &str,
        new_fd: Fd,
        new_path: &str,
    ) -> WasiResult<()>;

    /// Open a file or directory. The returned file descriptor is not guaranteed to be the lowest-numbered
    /// file descriptor not currently open; it is randomized to prevent applications from depending on making
    /// assumptions about indexes, since this is error-prone in multi-threaded contexts. The returned file
    /// descriptor is guaranteed to be less than 2^31.
    ///
    /// Note: This is similar to `openat` in POSIX.
    fn path_open(
        &self,
        fd: Fd,
        dirflags: Lookupflags,
        path: &str,
        oflags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fdflags: Fdflags,
    ) -> WasiResult<Fd>;

    /// Read the contents of a symbolic link.
    ///
    /// Note: This is similar to `readlinkat` in POSIX.
    fn path_readlink(&self, fd: Fd, path: &str) -> WasiResult<String>;

    /// Remove a directory. Return `Err(Errno::Notempty)` if the directory is not empty.
    ///
    /// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
    fn path_remove_directory(&self, fd: Fd, path: &str) -> WasiResult<()>;

    /// Rename a file or directory.
    ///
    /// Note: This is similar to `renameat` in POSIX.
    fn path_rename(&self, fd: Fd, old_path: &str, new_fd: Fd, new_path: &str) -> WasiResult<()>;

    /// Create a symbolic link.
    ///
    /// Note: This is similar to `symlinkat` in POSIX.
    fn path_symlink(&self, old_path: &str, fd: Fd, new_path: &str) -> WasiResult<()>;

    /// Unlink a file. Return `Err(Errno::Isdir)` if the path refers to a directory.
    ///
    /// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
    fn path_unlink_file(&self, fd: Fd, path: &str) -> WasiResult<()>;

    /// Concurrently poll for the occurrence of a set of events.
    fn poll_oneoff(&self, subscriptions: &[Subscription]) -> WasiResult<Vec<Event>>;

    /// Terminate the process normally. An exit code of 0 indicates successful termination of the program.
    /// The meanings of other values is dependent on the environment.
    ///
    /// Implementations should always return `Err(rval)`.
    fn proc_exit(&self, rval: Exitcode) -> Result<std::convert::Infallible, Exitcode>;

    /// Send a signal to the process of the calling thread.
    ///
    /// Note: This is similar to `raise` in POSIX.
    fn proc_raise(&self, sig: Signal) -> WasiResult<()>;

    /// Write high-quality random data into a buffer. This function blocks when the implementation is
    /// unable to immediately provide sufficient high-quality random data. This function may execute slowly,
    /// so when large mounts of random data are required, it's advisable to use this function to seed a
    /// pseudo-random number generator, rather than to provide the random data directly.
    fn random_get(&self, buf: &mut [u8]) -> WasiResult<()>;

    /// Temporarily yield execution of the calling thread.
    ///
    /// Note: This is similar to `sched_yield` in POSIX.
    fn sched_yield(&self) -> WasiResult<()>;

    /// Receive a message from a socket.
    ///
    /// Note: This is similar to `recv` in POSIX, though it also supports reading the data into multiple
    /// buffers in the manner of `readv`.
    fn sock_recv(
        &self,
        fd: Fd,
        ri_data: &[&mut [u8]],
        ri_flags: Riflags,
    ) -> WasiResult<(Size, Roflags)>;

    /// Send a message on a socket.
    ///
    /// Note: This is similar to `send` in POSIX, though it also supports writing the data from multiple buffers
    /// in the manner of `writev`.
    fn sock_send(&self, fd: Fd, si_data: &[&[u8]], si_flags: Siflags) -> WasiResult<Size>;

    /// Shut down socket send and receive channels.
    ///
    /// Note: This is similar to `shutdown` in POSIX.
    fn sock_shutdown(&self, fd: Fd, how: Sdflags) -> WasiResult<()>;
}

/// Extension methods for the [`WasiImports`](trait.WasiImports.html) trait.
pub trait WasiImportsExt {
    /// Generates the imports for this object.
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

    fn read_from_bufs(
        memory: &Memory,
        iovs: WasmSlicePtr<native::ciovec>,
        iovs_len: native::size,
    ) -> Vec<Vec<u8>> {
        let iovs = iovs.with(memory, iovs_len);

        (0..iovs_len)
            .map(|i| {
                let native::ciovec { buf, buf_len } = iovs.read(i);
                let iov = buf.with(memory, buf_len);

                (0..buf_len).map(|i| iov.read(i)).collect()
            })
            .collect()
    }

    fn read_from_buf(
        memory: &Memory,
        wasm_buf: WasmSlicePtr<u8>,
        wasm_buf_len: native::size,
    ) -> Vec<u8> {
        let wasm_buf = wasm_buf.with(memory, wasm_buf_len);

        (0..wasm_buf_len).map(|i| wasm_buf.read(i)).collect()
    }

    fn read_string_from_buf(
        memory: &Memory,
        wasm_buf: WasmSlicePtr<u8>,
        wasm_buf_len: native::size,
    ) -> WasiResult<String> {
        let buf = Self::read_from_buf(memory, wasm_buf, wasm_buf_len);
        String::from_utf8(buf).map_err(|_| Errno::Inval)
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

macro_rules! try2 {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(_) => return (native::errno_inval, Default::default(), Default::default()),
        }
    };
}

macro_rules! to_result2 {
    ($e:expr) => {
        match $e {
            Ok((val1, val2)) => (native::errno_success, val1.to_native(), val2.to_native()),
            Err(err) => (err.to_native(), Default::default(), Default::default()),
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
        let fd = try1!(Fd::from_native(fd));
        let offset = try1!(Filesize::from_native(offset));
        let data = Self::read_from_bufs(ctx.memory(0), iovs, iovs_len);

        let slices: Vec<_> = data.iter().map(|v| &v[..]).collect();

        to_result1!(self.0.fd_pwrite(fd, &slices[..], offset))
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
        let fd = try1!(Fd::from_native(fd));
        let mut cookie = try1!(Dircookie::from_native(cookie));
        let mut dirent_buf = [0u8; <native::dirent as WasmValue>::SIZE as usize];

        let buf = buf.with(ctx.memory(0), buf_len);
        let mut offset = 0;

        'outer: while offset < buf_len {
            let (entry, name) = match try1!(self.0.fd_readdir(fd, cookie)) {
                Some(entry) => entry,
                None => break,
            };

            entry
                .to_native()
                .write(&Cell::from_mut(&mut dirent_buf[..]).as_slice_of_cells());

            for i in 0..dirent_buf.len() {
                buf.write(offset, dirent_buf[i]);
                offset += 1;
                if offset == buf_len {
                    break 'outer;
                }
            }

            let name = name.as_bytes();
            assert_eq!(name.len() as u32, entry.DNamlen.0);
            for i in 0..entry.DNamlen.0 as usize {
                buf.write(offset, name[i]);
                offset += 1;
                if offset == buf_len {
                    break 'outer;
                }
            }

            cookie = entry.DNext;
        }

        (native::errno_success, offset)
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
        let fd = try1!(Fd::from_native(fd));
        let data = Self::read_from_bufs(ctx.memory(0), iovs, iovs_len);

        let slices: Vec<_> = data.iter().map(|v| &v[..]).collect();

        to_result1!(self.0.fd_write(fd, &slices[..]))
    }

    fn path_create_directory(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let path = try0!(Self::read_string_from_buf(ctx.memory(0), path, path_len));

        to_result0!(self.0.path_create_directory(fd, &path))
    }

    fn path_filestat_get(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        flags: native::lookupflags,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> (native::errno, native::filestat) {
        let fd = try1!(Fd::from_native(fd));
        let flags = try1!(Lookupflags::from_native(flags));
        let path = try1!(Self::read_string_from_buf(ctx.memory(0), path, path_len));

        to_result1!(self.0.path_filestat_get(fd, flags, &path))
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
        let fd = try0!(Fd::from_native(fd));
        let flags = try0!(Lookupflags::from_native(flags));
        let path = try0!(Self::read_string_from_buf(ctx.memory(0), path, path_len));
        let atim = try0!(Timestamp::from_native(atim));
        let mtim = try0!(Timestamp::from_native(mtim));
        let fst_flags = try0!(Fstflags::from_native(fst_flags));

        to_result0!(self
            .0
            .path_filestat_set_times(fd, flags, &path, atim, mtim, fst_flags))
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
        let memory = ctx.memory(0);

        let old_fd = try0!(Fd::from_native(old_fd));
        let old_flags = try0!(Lookupflags::from_native(old_flags));
        let old_path = try0!(Self::read_string_from_buf(memory, old_path, old_path_len));
        let new_fd = try0!(Fd::from_native(new_fd));
        let new_path = try0!(Self::read_string_from_buf(memory, new_path, new_path_len));

        to_result0!(self
            .0
            .path_link(old_fd, old_flags, &old_path, new_fd, &new_path))
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
        fs_rights_inheriting: native::rights,
        fdflags: native::fdflags,
    ) -> (native::errno, native::fd) {
        let fd = try1!(Fd::from_native(fd));
        let dirflags = try1!(Lookupflags::from_native(dirflags));
        let path = try1!(Self::read_string_from_buf(ctx.memory(0), path, path_len));
        let oflags = try1!(Oflags::from_native(oflags));
        let fs_rights_base = try1!(Rights::from_native(fs_rights_base));
        let fs_rights_inheriting = try1!(Rights::from_native(fs_rights_inheriting));
        let fdflags = try1!(Fdflags::from_native(fdflags));

        to_result1!(self.0.path_open(
            fd,
            dirflags,
            &path,
            oflags,
            fs_rights_base,
            fs_rights_inheriting,
            fdflags
        ))
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
        let memory = ctx.memory(0);

        let fd = try1!(Fd::from_native(fd));
        let path = try1!(Self::read_string_from_buf(memory, path, path_len));

        let result = match self.0.path_readlink(fd, &path) {
            Ok(result) => result,
            Err(err) => return (err.to_native(), Default::default()),
        };

        Self::write_to_buf(
            memory,
            buf,
            buf_len,
            |buf| {
                let result = result.as_bytes();
                let len = min(buf.len(), result.len());

                buf[..len].copy_from_slice(&result[..len]);
                (native::errno_success, len as u32)
            },
            |r| r.1,
        )
    }

    fn path_remove_directory(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let path = try0!(Self::read_string_from_buf(ctx.memory(0), path, path_len));

        to_result0!(self.0.path_remove_directory(fd, &path))
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
        let memory = ctx.memory(0);

        let fd = try0!(Fd::from_native(fd));
        let old_path = try0!(Self::read_string_from_buf(memory, old_path, old_path_len));
        let new_fd = try0!(Fd::from_native(new_fd));
        let new_path = try0!(Self::read_string_from_buf(memory, new_path, new_path_len));

        to_result0!(self.0.path_rename(fd, &old_path, new_fd, &new_path))
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
        let memory = ctx.memory(0);

        let old_path = try0!(Self::read_string_from_buf(memory, old_path, old_path_len));
        let fd = try0!(Fd::from_native(fd));
        let new_path = try0!(Self::read_string_from_buf(memory, new_path, new_path_len));

        to_result0!(self.0.path_symlink(&old_path, fd, &new_path))
    }

    fn path_unlink_file(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        path: WasmSlicePtr<u8>,
        path_len: native::size,
    ) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let path = try0!(Self::read_string_from_buf(ctx.memory(0), path, path_len));

        to_result0!(self.0.path_unlink_file(fd, &path))
    }

    fn poll_oneoff(
        &self,
        ctx: &mut Ctx,
        r#in: WasmSlicePtr<native::subscription>,
        out: WasmSlicePtr<native::event>,
        nsubscriptions: native::size,
    ) -> (native::errno, native::size) {
        let memory = ctx.memory(0);
        let subscriptions = r#in.with(memory, nsubscriptions);
        let subscriptions: Vec<_> = match (0..nsubscriptions)
            .map(|i| Subscription::from_native(subscriptions.read(i)))
            .collect()
        {
            Ok(s) => s,
            Err(_) => return (native::errno_inval, Default::default()),
        };

        let mut results = match self.0.poll_oneoff(&subscriptions[..]) {
            Ok(results) => results,
            Err(err) => return (err.to_native(), Default::default()),
        };
        results.truncate(nsubscriptions as usize);

        let out = out.with(memory, nsubscriptions);
        for (i, result) in results.iter().enumerate() {
            out.write(i as u32, result.to_native());
        }

        (native::errno_success, results.len() as u32)
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
        Self::write_to_buf(
            ctx.memory(0),
            buf,
            buf_len,
            |buf| to_result0!(self.0.random_get(buf)),
            |_| buf_len,
        )
    }

    fn sock_recv(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        ri_data: WasmSlicePtr<native::iovec>,
        ri_data_len: native::size,
        ri_flags: native::riflags,
    ) -> (native::errno, native::size, native::roflags) {
        let fd = try2!(Fd::from_native(fd));
        let ri_flags = try2!(Riflags::from_native(ri_flags));

        Self::write_to_bufs(
            ctx.memory(0),
            ri_data,
            ri_data_len,
            |buf| to_result2!(self.0.sock_recv(fd, buf, ri_flags)),
            |&(e, s, _)| if e == native::errno_success { s } else { 0 },
        )
    }

    fn sock_send(
        &self,
        ctx: &mut Ctx,
        fd: native::fd,
        si_data: WasmSlicePtr<native::ciovec>,
        si_data_len: native::size,
        si_flags: native::siflags,
    ) -> (native::errno, native::size) {
        let fd = try1!(Fd::from_native(fd));
        let si_flags = try1!(Siflags::from_native(si_flags));
        let si_data = Self::read_from_bufs(ctx.memory(0), si_data, si_data_len);

        let slices: Vec<_> = si_data.iter().map(|v| &v[..]).collect();

        to_result1!(self.0.sock_send(fd, &slices[..], si_flags))
    }

    fn sock_shutdown(&self, _ctx: &mut Ctx, fd: native::fd, how: native::sdflags) -> native::errno {
        let fd = try0!(Fd::from_native(fd));
        let how = try0!(Sdflags::from_native(how));

        to_result0!(self.0.sock_shutdown(fd, how))
    }
}
