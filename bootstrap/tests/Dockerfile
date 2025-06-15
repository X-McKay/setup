FROM ubuntu:24.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install required packages
RUN apt-get update && apt-get install -y \
    bash \
    curl \
    wget \
    git \
    sudo \
    systemd-sysv \
    net-tools \
    cron \
    && rm -rf /var/lib/apt/lists/*

# Create test user
RUN useradd -m -s /bin/bash testuser && \
    echo "testuser ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Set working directory
WORKDIR /app

# Copy test files
COPY bootstrap/tests/ /app/
# Copy scripts
COPY bootstrap/scripts/ /app/scripts/

# Set permissions
RUN chown -R testuser:testuser /app && \
    chmod -R 755 /app/scripts

# Switch to test user
USER testuser

# Set environment variables
ENV TEST_MODE=1
ENV TEST_USER=testuser
ENV TEST_HOME=/home/testuser

# Create necessary directories
RUN mkdir -p /home/testuser/.monitoring \
    /home/testuser/.backup \
    /home/testuser/.bootstrap

# Command to run tests
CMD ["bash", "run_tests.sh"]
