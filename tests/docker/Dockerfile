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

# Copy .tool-versions for mise (needs to be owned by testuser)
COPY --chown=testuser:testuser bootstrap/dotfiles/tool-versions /home/testuser/.tool-versions

# Install mise and Rust as testuser (mirrors bootstrap.sh flow)
USER testuser
WORKDIR /home/testuser
ENV HOME=/home/testuser
ENV PATH="/home/testuser/.local/bin:${PATH}"

RUN curl -fsSL https://mise.run | sh
# python-build-standalone still omits attestations for the install-only
# archives that mise uses here, so disable that verification in Docker e2e.
RUN MISE_PYTHON_GITHUB_ATTESTATIONS=false mise install --yes
ENV PATH="/home/testuser/.local/share/mise/shims:${PATH}"

# Copy the project (as root, then fix permissions)
USER root
WORKDIR /setup
COPY . .
RUN chown -R testuser:testuser /setup

# Build the CLI as testuser
USER testuser
WORKDIR /home/testuser
# Build from $HOME so the home-level .tool-versions file applies to mise shims.
RUN cargo build --release --manifest-path /setup/cli/Cargo.toml

# Copy test script and set permissions
USER root
COPY tests/docker/test_installs.sh /test_installs.sh
RUN chmod +x /test_installs.sh && chown testuser:testuser /test_installs.sh

# Run tests as testuser
USER testuser
WORKDIR /home/testuser
ENV HOME=/home/testuser

CMD ["/test_installs.sh"]
