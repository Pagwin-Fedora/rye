FROM rust:alpine
RUN ["apk", "add", "libgit2", "git"]
COPY Rocket.toml /
COPY config.toml /
COPY rye /usr/bin/rye
ENV RYE_CONFIG="/config.toml"
CMD ["rye"]
