FROM rust:1.96-bookworm

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential debhelper fakeroot \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN dpkg-buildpackage -us -uc -b

RUN mkdir -p /out && cp /ratatui-todo-list_*.deb /out/

CMD ["sh", "-c", "cp /out/*.deb /output/"]
