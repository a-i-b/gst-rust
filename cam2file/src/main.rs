extern crate gstreamer as gst;
use gst::prelude::*;
extern crate glib;
extern crate ctrlc;
use std::error::Error as StdError;
use std::thread;
use std::time::Duration;
use std::fs;
use std::env;

#[path = "common.rs"]
mod run_common;

extern crate failure;
use failure::Error;

#[macro_use]
extern crate failure_derive;

/*
#[derive(Debug, Fail)]
#[fail(display = "Missing element {}", _0)]
struct MissingElement(&'static str);

#[derive(Debug, Fail)]
#[fail(display = "No such pad {} in {}", _0, _1)]
struct NoSuchPad(&'static str, String);

#[derive(Debug, Fail)]
#[fail(display = "Usage: {} URI FEC_PERCENTAGE", _0)]
struct UsageError(String);
*/

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
    let args: Vec<String> = env::args().collect();
    let mut file_path = String::from("capture.mp4");
    if args.len() > 1 {
        file_path = args[1].clone();
    }
    println!("Capturing from webcam to {}...", file_path);

    gst::init()?;
    let pipeline = gst::parse_launch(&buid_pipeline())?;

    let pl = pipeline.clone().dynamic_cast::<gst::Pipeline>().unwrap();
    let filesink = pl.get_by_name("fs").unwrap();
    let _ = filesink.set_property("location", &file_path);

    pipeline.set_state(gst::State::Playing)?;

    let ctrl_pipe = pipeline.clone();
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        let ev = gst::Event::new_eos().build();
        ctrl_pipe.send_event(ev);
    }).expect("Error setting Ctrl-C handler");

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let fp = file_path.clone();
            let x = fs::metadata(fp).unwrap().len();        
            println!("File size: {}", x);
        }
    });

    wait_loop(&pipeline)?;
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

fn main() {
    match run_common::run(srv_main) {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }
}

#[cfg(target_os = "macos")]
fn buid_pipeline() -> String {
   "no" 
}

#[cfg(target_os = "windows")]
fn buid_pipeline() -> String {
    String::from("ksvideosrc ! videoconvert ! x264enc ! mp4mux ! filesink name=fs buffer-mode=full buffer-size=10000 sync=false")
}

fn wait_loop(pipeline: &gst::Element)  -> Result<(), Error> {
    let bus = pipeline
        .get_bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline
                    .set_state(gst::State::Null)
                    .expect("Unable to set the pipeline to the `Null` state");

                return Err(ErrorMessage {
                    src: msg
                        .get_src()
                        .map(|s| String::from(s.get_path_string()))
                        .unwrap_or_else(|| String::from("None")),
                    error: err.get_error().description().into(),
                    debug: Some(err.get_debug().unwrap().to_string()),
                    cause: err.get_error(),
                }
                .into());
            }
            _ => (),
        }
    }

    Ok(())    
}