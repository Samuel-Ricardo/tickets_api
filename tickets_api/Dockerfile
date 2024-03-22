FROM rust:1.76 as build

WORKDIR /app

COPY . .

RUN cargo build --release

FROM  rust:1.76

WORKDIR /usr/local/bin

COPY --from=build /app/target/release/tickets_api .

#COPY --from=build /app .

EXPOSE 8080

CMD [ "./tickets_api" ]
