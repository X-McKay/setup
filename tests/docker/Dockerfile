FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV HOME=/root

# Install base dependencies needed for building and testing
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    sudo \
    cron \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the project
WORKDIR /setup
COPY . .

# Build the CLI
RUN cd cli && cargo build --release

# Copy test script
COPY tests/docker/test_installs.sh /test_installs.sh
RUN chmod +x /test_installs.sh

# Run tests
CMD ["/test_installs.sh"]
