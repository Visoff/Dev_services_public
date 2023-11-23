FROM rust

COPY . .

CMD [ "cargo", "run" ]