FROM ekidd/rust-musl-builder:beta as builder

WORKDIR /home/rust/src/skool
RUN sudo chown -R rust:rust /home/rust/src/skool
ADD . .
RUN cargo build --release --bin skool

FROM alpine:latest

RUN addgroup -S appuser \
    && adduser -S -g appuser appuser

RUN apk update \
	&& apk add --no-cache ca-certificates \
	&& rm -rf /var/cache/apk/*

WORKDIR /usr/src/app

COPY --from=builder /home/rust/src/skool/target/x86_64-unknown-linux-musl/release/skool ./

RUN chown -R appuser:appuser /usr/src/app
USER appuser

EXPOSE 8000
ENTRYPOINT ["./skool"]
