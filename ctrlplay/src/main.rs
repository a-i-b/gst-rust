use std::str;
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use rumqtt::client::{Notification};

fn main() {
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
                println!(
                    "topic = {}, \
                     qos = {:?}, \
                     payload = {:?}",
                    p.topic_name,
                    p.qos,
                    s);
            },
            _ => (),
        }
    }
}