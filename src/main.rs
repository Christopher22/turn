#[macro_use]
extern crate clap;
use clap::Arg;

mod turning_table;

use std::io;
use std::io::prelude::*;

use turning_table::{TurningTable, Status};

/// Writes an error into STDERR.
macro_rules! write_error (
    ($($arg:tt)*) => { {
        writeln!(&mut ::std::io::stderr(), $($arg)*).expect("failed printing to stderr");
    } }
);

/// Parses the degrees of a rotation.
fn parse_rotation<T: AsRef<str>>(input: T) -> Result<i16, String> {
    match input.as_ref().parse::<i16>() {
        Ok(deg @ -360...360) => Ok(deg),
        Ok(_) => Err("Rotation was greater than 360 degrees.".to_owned()),
        Err(_) => Err("Rotation was not a valid number.".to_owned()),
    }
}

/// Turns the turntable and print any occurring errors.
fn turn(table: &mut TurningTable, degrees: i16, blocking: bool) {
    table.turn(degrees);

    // Block, if requested
    while blocking && table.status() == Status::Moving {
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    if let Status::Error(err) = table.status() {
        write_error!("[ERROR] {}", err);
    }
}

fn main() {

    // Parses the arguments
    let args = app_from_crate!()
        .arg(Arg::with_name("noblock")
            .short("n")
            .long("noblock")
            .help("Do not block the execution until the turning table has stopped."))
        .get_matches();

    // Initialize the table
    let mut table = match TurningTable::new() {
        Ok(table) => table,
        Err(err) => {
            write_error!("[ERROR] {}", err);
            return;
        }
    };

    // Read degrees from STDIN.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match parse_rotation(line.expect("Valid line")) {
            Ok(deg) => turn(&mut table, deg, !args.is_present("noblock")),
            Err(message) => write_error!("[ERROR] {}", message),
        }
    }
}
