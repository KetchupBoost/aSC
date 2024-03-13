FROM rust AS build
WORKDIR /app

COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
COPY .env .
RUN cargo fetch
COPY .sqlx .sqlx
RUN cargo build --release

FROM debian:stable-slim
COPY --from=build /app/target/release/a_sc /usr/bin/a_sc
EXPOSE 3000
CMD ["a_sc"]