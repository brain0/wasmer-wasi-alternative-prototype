use wasihost_core::wasi_snapshot_preview1::{Clockid, Errno, Timestamp, WasiResult};

pub(crate) fn preview1_clock_res_get(id: Clockid) -> WasiResult<Timestamp> {
    Err(Errno::Nosys)
}

pub(crate) fn preview1_clock_time_get(id: Clockid, precision: Timestamp) -> WasiResult<Timestamp> {
    Err(Errno::Nosys)
}
