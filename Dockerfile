FROM rust:alpine
COPY . /src
WORKDIR /src
RUN ["cargo", "fetch"]
RUN ["rustup", "toolchain", "install", "nightly"]
RUN ["rustup", "override", "set", "nightly"]
RUN ["cargo", "build", "--release"]
CMD ["target/relese/rye"]
