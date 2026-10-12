FROM rust:latest

RUN apt-get update && apt-get install -y \
    make \
    curl \
    unzip \
    pkg-config \
    alsa-utils \
    libasound2-dev \
    libudev-dev \
    libvulkan-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir /bomberman

WORKDIR /bomberman

CMD ["make"]
