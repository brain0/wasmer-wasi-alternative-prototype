use libc::{
    clock_getres, clock_gettime, clockid_t, timespec, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID,
    CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID,
};
use wasihost_core::wasi_snapshot_preview1::{Clockid, Errno, Timestamp, WasiResult};

fn get_unix_clockid(id: Clockid) -> Option<clockid_t> {
    match id {
        Clockid::Realtime => Some(CLOCK_REALTIME),
        Clockid::Monotonic => Some(CLOCK_MONOTONIC),
        Clockid::ProcessCputimeId => Some(CLOCK_PROCESS_CPUTIME_ID),
        Clockid::ThreadCputimeId => Some(CLOCK_THREAD_CPUTIME_ID),
        _ => None,
    }
}

fn clock_error_from_errno() -> Errno {
    match errno::errno().0 {
        libc::EINVAL => Errno::Inval,
        libc::EPERM => Errno::Perm,
        _ => Errno::Inval,
    }
}

fn clock_timestamp_from_timespce(timespec: timespec) -> Timestamp {
    Timestamp((timespec.tv_sec as u64 * 1_000_000_000).wrapping_add(timespec.tv_nsec as u64))
}

pub(crate) fn preview1_clock_res_get(id: Clockid) -> WasiResult<Timestamp> {
    let unix_clock_id = get_unix_clockid(id).ok_or(Errno::Inval)?;

    let mut timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    if unsafe { clock_getres(unix_clock_id, &mut timespec) } == 0 {
        Ok(clock_timestamp_from_timespce(timespec))
    } else {
        Err(clock_error_from_errno())
    }
}

pub(crate) fn preview1_clock_time_get(id: Clockid, _precision: Timestamp) -> WasiResult<Timestamp> {
    let unix_clock_id = get_unix_clockid(id).ok_or(Errno::Inval)?;

    let mut timespec: timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    if unsafe { clock_gettime(unix_clock_id, &mut timespec) } == 0 {
        Ok(clock_timestamp_from_timespce(timespec))
    } else {
        Err(clock_error_from_errno())
    }
}
