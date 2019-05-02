#[macro_use]
extern crate gstreamer as gst;
use gst::prelude::*;

extern crate glib;

use std::error::Error as StdError;

#[path = "common.rs"]
mod run_common;

use std::env;

extern crate failure;
use failure::Error;

#[macro_use]
extern crate failure_derive;

#[derive(Debug, Fail)]
#[fail(display = "Missing element {}", _0)]
struct MissingElement(&'static str);

#[derive(Debug, Fail)]
#[fail(display = "No such pad {} in {}", _0, _1)]
struct NoSuchPad(&'static str, String);

#[derive(Debug, Fail)]
#[fail(display = "Usage: {} URI FEC_PERCENTAGE", _0)]
struct UsageError(String);

#[derive(Debug, Fail)]
#[fail(
    display = "Received error from {}: {} (debug: {:?})",
    src, error, debug
)]
struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<String>,
    #[cause]
    cause: glib::Error,
}

fn srv_main() -> Result<(), Error> {
    gst::init()?;
    let pipeline = gst::Pipeline::new(None);
    Ok(())
}

fn main() {
    println!("Starting RIST server...");
    match run_common::run(srv_main) {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }
}
