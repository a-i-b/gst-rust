gst-launch-1.0 -v -e ksvideosrc ! videoconvert ! openh264enc bitrate=1000000 ! queue ! rtph264pay ! udpsink host=127.0.0.1 auto-multicast=true port=7777 sync=false 