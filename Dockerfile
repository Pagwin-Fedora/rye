FROM busybox:latest
COPY rye /bin/rye
COPY Rocket.toml /
CMD ["/bin/rye"]
