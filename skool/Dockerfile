FROM clux/muslrust:1.65.0 as build
WORKDIR /app
COPY . .
RUN cargo build --release --bin skool

FROM gcr.io/distroless/static-debian11
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/skool /
EXPOSE 8000
ENTRYPOINT ["/skool"]
