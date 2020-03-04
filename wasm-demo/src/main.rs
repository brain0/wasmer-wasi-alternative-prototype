use std::{env, fs::File, io::Read, string::String};
use wasmer_runtime::{instantiate, Func};
use wasmer_wasi_alternative_prototype::wasi_snapshot_preview1::*;

struct Wasi {
    arguments: Vec<String>,
    environment: Vec<String>,
}

impl WasiImports for Wasi {
    fn args_get(&self) -> WasiResult<&[String]> {
        unimplemented!("args_get")
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

    fn fd_read(&self, _: Fd, _: &[&mut [u8]]) -> WasiResult<Size> {
        unimplemented!("fd_read")
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

    fn proc_exit(&self, c: Exitcode) -> Result<std::convert::Infallible, Exitcode> {
        Err(c)
    }

    fn proc_raise(&self, _: Signal) -> WasiResult<()> {
        unimplemented!("proc_raise")
    }

    fn sched_yield(&self) -> WasiResult<()> {
        unimplemented!("sched_yield")
    }

    fn sock_shutdown(&self, _: Fd, _: Sdflags) -> WasiResult<()> {
        unimplemented!("sock_shutdown")
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

    let instance = {
        let mut wasm = Vec::new();

        {
            let mut file = File::open(&wasm_filename).expect("Failed to open file");
            file.read_to_end(&mut wasm).expect("Failed to read file");
        }

        let import_object = wasi.into_imports();
        instantiate(&wasm[..], &import_object).expect("Failed to instantiate")
    };

    let start: Func<()> = instance
        .func("_start")
        .expect("Unable to find _start function");

    let code = match start.call() {
        Ok(()) => 0,
        Err(e) => match e.0.downcast_ref::<native::exitcode>() {
            Some(&code) => code,
            None => panic!("Failed to get exit code."),
        }
    };

    println!("WASI program exited with exit code {}.", code);
}
