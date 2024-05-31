FROM rust:latest as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM ubuntu:latest 

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

RUN echo -e "#!/usr/bin/env pwsh\n\
    ./get_data\n\
    ./better300" > /usr/src/app/start.sh \
    && chmod +x /usr/src/app/start.sh

CMD ["./better300"]



