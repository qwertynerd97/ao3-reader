FROM debian:bookworm-slim

# Enable ARMhf packages & update the package index
RUN dpkg --add-architecture armhf
RUN apt update

# Install BusyBox for my sanity
RUN apt install --yes busybox

# Install AO3 Reader's undocumented dependencies
RUN apt install --yes \
  autoconf \
  build-essential \
  cmake \
  g++-arm-linux-gnueabihf \
  gcc-arm-linux-gnueabihf \
  libtool

# Install AO3 Reader's dependencies
RUN apt install --yes \
  curl \
  git \
  jq \
  libsdl2-dev:armhf \
  patchelf \
  pkg-config \
  unzip \
  wget

# Install Rust & enable ARMhf cross compilation
ENV CARGO_HOME=/opt/cargo
ENV RUSTUP_HOME=/opt/rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly --profile minimal --target arm-unknown-linux-gnueabihf --no-modify-path
# RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --no-modify-path
ENV PATH="${PATH}:/opt/cargo/bin"
# RUN rustup toolchain install nightly
# RUN rustup target add arm-unknown-linux-gnueabihf
RUN chmod --recursive 777 /opt/cargo
RUN chmod --recursive 777 /opt/rustup

ENV LD_LIBRARY_PATH="/opt/ao3-reader/libs"
WORKDIR /opt/ao3-reader
ENTRYPOINT ["/bin/bash"]
