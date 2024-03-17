FROM rust as builder

WORKDIR /app

COPY . .

RUN cargo build --release

RUN strip target/release/dev_services

FROM gcr.io/distroless/cc-debian12 as runtime

COPY --from=builder /app/target/release/dev_services /app

CMD [ "/app" ]
