FROM clux/muslrust:1.63.0 as build-env
WORKDIR /app
COPY . .
RUN cargo build --release --bin skool

FROM gcr.io/distroless/static:nonroot
COPY --from=build-env --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/skool /
EXPOSE 8000
ENTRYPOINT ["/skool"]
