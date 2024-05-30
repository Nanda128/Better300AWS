FROM rust:latest as builder

RUN apt-get update && apt-get install -y libcurl4

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

#Debian didnt have the necessary glibc library so we're using ubuntu!
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    sqlite3 \
    libsqlite3-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/get_data /usr/src/app/
COPY --from=builder /usr/src/app/target/release/better300 /usr/src/app/

EXPOSE 8061

RUN echo -e "#!/bin/bash\n\
    ./get_data\n\
    ./better300" > /usr/src/app/start.sh \
    && chmod +x /usr/src/app/start.sh

CMD ["./better300"]
