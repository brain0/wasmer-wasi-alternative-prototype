#![forbid(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]
#![warn(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
#![allow(unused_variables)]

use std::{env, fs::File, io::Read, string::String};
use wasmer_runtime::{instantiate, Func};
use wasmer_wasi_alternative_prototype::wasi_snapshot_preview1::*;

struct Wasi {
    arguments: Vec<String>,
    environment: Vec<String>,
}

impl Wasi {
    fn write_bufs<W: std::io::Write>(mut writer: W, bufs: &[&[u8]]) -> WasiResult<Size> {
        for buf in bufs {
            writer.write_all(buf).map_err(|_| Errno::Inval)?;
        }
        writer.flush().map_err(|_| Errno::Inval)?;

        Ok(Size(bufs.iter().map(|b| b.len()).sum::<usize>() as u32))
    }
}

impl WasiImports for Wasi {
    fn args_get(&self) -> WasiResult<&[String]> {
        Ok(&self.arguments[..])
    }

    fn environ_get(&self) -> WasiResult<&[String]> {
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

    fn fd_prestat_dir_name(&self, _: Fd) -> WasiResult<String> {
        unimplemented!("fd_prestat_dir_name")
    }

    fn fd_pwrite(&self, _: Fd, _: &[&[u8]], _: Filesize) -> WasiResult<Size> {
        unimplemented!("fd_pwrite")
    }

    fn fd_read(&self, _: Fd, _: &[&mut [u8]]) -> WasiResult<Size> {
        unimplemented!("fd_read")
    }

    fn fd_readdir(&self, _: Fd, _: Dircookie) -> WasiResult<Option<(Dirent, String)>> {
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

    fn path_create_directory(&self, fd: Fd, path: &str) -> WasiResult<()> {
        todo!("path_create_directory")
    }

    fn path_filestat_get(&self, fd: Fd, flags: Lookupflags, path: &str) -> WasiResult<Filestat> {
        todo!("path_filestat_get")
    }

    fn path_filestat_set_times(
        &self,
        fd: Fd,
        flags: Lookupflags,
        path: &str,
        atim: Timestamp,
        mtim: Timestamp,
        fst_flags: Fstflags,
    ) -> WasiResult<()> {
        todo!("path_filestat_set_times")
    }

    fn path_open(
        &self,
        fd: Fd,
        dirflags: Lookupflags,
        path: &str,
        oflags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fdflags: Fdflags,
    ) -> WasiResult<Fd> {
        todo!("path_open")
    }

    fn path_link(
        &self,
        old_fd: Fd,
        old_flags: Lookupflags,
        old_path: &str,
        new_fd: Fd,
        new_path: &str,
    ) -> WasiResult<()> {
        todo!("path_link")
    }

    fn path_readlink(&self, fd: Fd, path: &str) -> WasiResult<String> {
        todo!("path_readlink")
    }

    fn path_remove_directory(&self, fd: Fd, path: &str) -> WasiResult<()> {
        todo!("path_remove_directory")
    }

    fn path_rename(&self, fd: Fd, old_path: &str, new_fd: Fd, new_path: &str) -> WasiResult<()> {
        todo!("path_rename")
    }

    fn path_symlink(&self, old_path: &str, fd: Fd, new_path: &str) -> WasiResult<()> {
        todo!("path_symlink")
    }

    fn path_unlink_file(&self, fd: Fd, path: &str) -> WasiResult<()> {
        todo!("path_unlink_file")
    }

    fn poll_oneoff(&self, subscriptions: &[Subscription]) -> WasiResult<Vec<Event>> {
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
        fd: Fd,
        ri_data: &[&mut [u8]],
        ri_flags: Riflags,
    ) -> WasiResult<(Size, Roflags)> {
        todo!("sock_recv")
    }

    fn sock_send(&self, fd: Fd, si_data: &[&[u8]], si_flags: Siflags) -> WasiResult<Size> {
        todo!("sock_send")
    }

    fn sock_shutdown(&self, _: Fd, _: Sdflags) -> WasiResult<()> {
        todo!("sock_shutdown")
    }
}

fn main() {
    let wasm_filename = env::args().skip(1).next().unwrap();

    let arguments = env::args().skip(2).collect();
    let environment = env::vars()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();

    let wasi = Wasi {
        arguments,
        environment,
    };

    eprintln!("Compiling WASI ...");

    let instance = {
        let mut wasm = Vec::new();

        {
            let mut file = File::open(&wasm_filename).expect("Failed to open file");
            file.read_to_end(&mut wasm).expect("Failed to read file");
        }

        let import_object = wasi.into_imports();
        instantiate(&wasm[..], &import_object).expect("Failed to instantiate")
    };

    eprintln!("Looking for entry point ...");

    let start: Func<'_, ()> = instance
        .func("_start")
        .expect("Unable to find _start function");

    eprintln!("Running WASI binary ...");

    let code = match start.call() {
        Ok(()) => 0,
        Err(e) => match e.0.downcast_ref::<native::exitcode>() {
            Some(&code) => code,
            None => panic!("Failed to get exit code."),
        },
    };

    eprintln!("WASI program exited with exit code {}.", code);
}
