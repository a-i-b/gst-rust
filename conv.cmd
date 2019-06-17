gst-launch-1.0 -e filesrc location=capture.264 ! h264parse ! identity datarate = 125000 ! mp4mux ! filesink location=capture2.mp4
