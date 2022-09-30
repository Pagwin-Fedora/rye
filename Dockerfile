FROM busybox:latest
RUN ["ls"]
RUN ["cargo", "build", "--release"]
