use std::str;
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use rumqtt::client::{Notification};
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
    let mut file_path = String::from("/Users/alex/workspace/gst-rust/capturex.mp4");
    if args.len() > 1 {
        file_path = args[1].clone();
    }
    println!("Capturing from webcam to {}...", file_path);

    gst::init()?;
    let pipeline = gst::parse_launch(&buid_pipeline())?;

    let pl = pipeline.clone().dynamic_cast::<gst::Pipeline>().unwrap();
    let filesink = pl.get_by_name("fsrc").unwrap();
    let _ = filesink.set_property("location", &file_path);

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

    let pipeline_ctrl = pipeline.clone();
    thread::spawn(move || {
        start_mqtt_recv(&pipeline_ctrl);
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
   "filesrc name=fsrc ! qtdemux ! queue ! h264parse ! vtdec_hw ! videoconvert ! osxvideosink".into()
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
fn start_mqtt_recv(pipeline: &gst::Element) {
    pretty_env_logger::init();
    let broker = "localhost";
    let port = 1883;

    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new("ctrlplay", broker, port)
                                    .set_keep_alive(10)
                                    .set_inflight(3)
                                    .set_request_channel_capacity(3)
                                    .set_reconnect_opts(reconnection_options)
                                    .set_clean_session(false);

    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
    mqtt_client.subscribe("ctrl/video", QoS::AtLeastOnce).unwrap();

    for notification in notifications {
        match notification {
            Notification::Publish(p) => 
            {
                let payload = p.payload.clone();
                let s = match str::from_utf8(&payload) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                }; 
                if s.starts_with("start") {
                    println!("Starting pipe...");
                    pipeline.set_state(gst::State::Playing).unwrap();
                } else if s.starts_with("stop") {
                    println!("Stopping pipe...");
                    pipeline.set_state(gst::State::Paused).unwrap();
                } else if s.starts_with("seek") {
                    println!("Seeking...");
                }
            },
            _ => (),
        }
    }
}