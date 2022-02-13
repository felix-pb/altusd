#######################
#     build stage     #
#######################
FROM rust:1.58.1 AS build
WORKDIR /app
# first install the dependencies to leverage docker's build cache.
RUN cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
# then copy the source code and build the binaries.
COPY src src
RUN cargo build --release

#######################
#     final stage     #
#######################
FROM ubuntu:20.04
WORKDIR /app
# first install ca-certificates to use secure websockets.
RUN apt-get update && apt-get install -y ca-certificates
## then copy over just the binaries to keep a small image size.
COPY --from=build /app/target/release/altusd /usr/local/bin/altusd
COPY --from=build /app/target/release/client /usr/local/bin/client
# expose port 8080 for the websocket server.
EXPOSE 8080
CMD [ "altusd" ]
