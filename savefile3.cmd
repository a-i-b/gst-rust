gst-launch-1.0 -e ksvideosrc ! videoconvert ! openh264enc bitrate=1000000 ! tee name=t1 ! queue ! h264parse ! filesink location=capture.264  t1. ! queue ! h264parse ! mp4mux ! filesink location=capture1.mp4 sync=false
