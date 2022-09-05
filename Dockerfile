FROM rust:1-alpine3.16 AS dependencies
RUN apk add libgsasl clang gcc musl-dev

FROM dependencies AS build
COPY . .
RUN cargo test
RUN cargo build --release

FROM build
WORKDIR /root/
COPY --from=build target/release/vsmtp .
ENTRYPOINT [ "./vsmtp" ]
