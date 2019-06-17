REM SET GST_DEBUG=3
gst-launch-1.0 -v -e filesrc location=capture.264 ! h264parse ! openh264dec ! videorate ! video/x-raw,framerate=30/1 ! autovideosink
