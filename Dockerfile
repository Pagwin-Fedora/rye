FROM rust:alpine
RUN ["apk", "add", "libgit2", "git"]
COPY Rocket.toml /
COPY config.toml /
COPY rye /usr/bin/rye
ENV CONFIG_FILE="/config.toml"
CMD ["rye"]
