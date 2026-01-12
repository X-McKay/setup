FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

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

# Create a test user (simulates real user installation)
RUN useradd -m -s /bin/bash testuser && \
    echo "testuser ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Install Rust as testuser
USER testuser
WORKDIR /home/testuser
ENV HOME=/home/testuser
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/testuser/.cargo/bin:${PATH}"

# Copy the project (as root, then fix permissions)
USER root
WORKDIR /setup
COPY . .
RUN chown -R testuser:testuser /setup

# Build the CLI as testuser
USER testuser
RUN cd cli && cargo build --release

# Copy test script and set permissions
USER root
COPY tests/docker/test_installs.sh /test_installs.sh
RUN chmod +x /test_installs.sh && chown testuser:testuser /test_installs.sh

# Run tests as testuser
USER testuser
WORKDIR /home/testuser
ENV HOME=/home/testuser

CMD ["/test_installs.sh"]
