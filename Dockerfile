FROM rust:alpine
ENV RYE_CONFIG="/config.toml"
# if you change the port in Rocket.toml you'll want to change it here as well
EXPOSE 9090
RUN ["apk", "add", "libgit2", "git"]
RUN ["mkdir", "/repos"]
COPY Rocket.toml /
COPY config.toml /
COPY rye /usr/bin/rye
CMD ["rye"]
