FROM rust:1 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:stable-slim
COPY --from=build /app/target/release/ovenlight /usr/local/bin/ovenlight
ENV PORT=3003
EXPOSE 3003
CMD ["ovenlight"]
