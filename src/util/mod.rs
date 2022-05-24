#![allow(clippy::single_component_path_imports)]

mod fnv;
mod graph;
mod id;
#[cfg(feature = "env")]
mod str_to_bool;

pub use self::fnv::Key;

#[cfg(feature = "env")]
pub(crate) use self::str_to_bool::str_to_bool;
pub(crate) use self::{graph::ChildGraph, id::Id};

pub(crate) mod color;

pub(crate) const SUCCESS_CODE: i32 = 0;
// While sysexists.h defines EX_USAGE as 64, this doesn't seem to be used much in practice but
// instead 2 seems to be frequently used.
// Examples
// - GNU `ls` returns 2
// - Python's `argparse` returns 2
pub(crate) const USAGE_CODE: i32 = 2;

#[cfg(not(feature = "ios_system"))]
pub(crate) fn safe_exit(code: i32) -> ! {
    use std::io::Write;

    let _ = std::io::stdout().lock().flush();
    let _ = std::io::stderr().lock().flush();

    std::process::exit(code)
}

#[cfg(feature = "ios_system")]
pub(crate) fn safe_exit(code: i32) -> ! {
    #[cfg(feature = "ios_system")]
    #[link(name = "ios_system", kind = "framework")]
    extern "C" {
        fn ios_exit(code: i32) -> !;
        fn ios_stdoutFd() -> i32;
        fn ios_stderrFd() -> i32;
    }
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::io::FromRawFd;

    unsafe {
        let fd = ios_stdoutFd();
        if fd > 0 {
            _ = File::from_raw_fd(fd).flush();
        }

        let fd = ios_stderrFd();
        if fd > 0 {
            _ = File::from_raw_fd(fd).flush();
        }

        ios_exit(code)
    }
}

#[cfg(not(feature = "unicode"))]
pub(crate) fn eq_ignore_case(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(feature = "unicode")]
pub(crate) use unicase::eq as eq_ignore_case;
