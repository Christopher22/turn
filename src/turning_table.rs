extern crate libc;
use self::libc::{c_int, c_char, c_double};

use std::ptr;

struct D4Motor;

#[link(name = "d4lib", kind="static")]
extern "C" {
    fn d4lib_init(_: *const c_char) -> c_int;
    fn d4lib_release() -> c_int;

    fn d4motor_newMotor() -> *mut D4Motor;
    fn d4motor_openTurntable(motor: *mut D4Motor,
                             pluginsDir: *const c_char,
                             moduleName: *const c_char,
                             logDir: *const c_char)
                             -> c_int;
    fn d4motor_close(motor: *mut D4Motor);

    fn d4motor_move(motor: *mut D4Motor, amount: c_double) -> c_int;
    fn d4motor_getStatus(motor: *mut D4Motor, motorStatus: *mut c_int) -> c_int;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// The current status of the turntable.
pub enum Status {
    /// The turning table is currently moving.
    Moving,
    /// The turning table stopped.
    Stopped,
    /// An error occurres.
    Error(Error),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// An error which might occure during the usage or the initialization.
pub enum Error {
    /// The license dongle was unavaileble.
    NotLicensed,
    /// The turning table was not found.
    DeviceUnavailable,
    /// The turnign table was unable to move.
    Stalled,
    /// The turning table has no power.
    NoPower,
    /// An unknown error occurres.
    UnknownError,
}

impl From<c_int> for Status {
    fn from(code: c_int) -> Self {
        match code {
            0 => Status::Stopped,
            1 => Status::Moving,
            -1 => Status::Error(Error::Stalled),
            -3 => Status::Error(Error::NoPower),
            _ => Status::Error(Error::UnknownError),
        }
    }
}

/// The controller of a turningtable by DAVID/HP.
pub struct TurningTable(*mut D4Motor);

impl TurningTable {
    /// Creates a new controller
    fn new() -> Result<Self, Error> {
        if unsafe { d4lib_init(ptr::null()) } != 0 {
            return Err(Error::NotLicensed);
        }

        let handle = match unsafe { d4motor_newMotor() } {
            handle if handle.is_null() => return Err(Error::UnknownError),
            handle => handle,
        };

        match unsafe { d4motor_openTurntable(handle, ptr::null(), ptr::null(), ptr::null()) } {
            0 => Ok(TurningTable(handle)),
            -300 => Err(Error::DeviceUnavailable),
            _ => Err(Error::UnknownError),
        }
    }

    /// Returns the current status of the turning table.
    fn status(&self) -> Status {
        let mut status = 0;
        match unsafe { d4motor_getStatus(self.0, &mut status as *mut c_int) } {
            0 => Status::from(status),
            _ => Status::Error(Error::UnknownError),
        }
    }

    /// Turns the turning table.
    fn turn(&mut self, degrees: i16) {
        unsafe { d4motor_move(self.0, degrees as c_double) };
    }
}

impl Drop for TurningTable {
    fn drop(&mut self) {
        unsafe { d4motor_close(self.0) }
        unsafe { d4lib_release() };
    }
}
