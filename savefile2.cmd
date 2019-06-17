gst-launch-1.0 -v -e ksvideosrc ! videoconvert ! openh264enc bitrate=1000000 ! h264parse ! filesink location=capture.264
