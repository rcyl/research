FROM debian:bookworm

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    wget \
    curl \
    git \
    build-essential \
    python3 \
    python3-pip \
    # ARM toolchain for building firmware
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi \
    # GDB for ARM debugging
    gdb-multiarch \
    # Utilities
    unzip minicom \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Rust to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Add thumb toolchain
RUN rustup target add thumbv7em-none-eabihf

# Create python symlink (required by libopencm3 build)
RUN ln -s /usr/bin/python3 /usr/bin/python

# Add claude code
RUN curl -fsSL https://claude.ai/install.sh | bash

# USER $USER_NAME
WORKDIR /src
ENV IS_SANDBOX=1
RUN echo "alias claude-yolo='claude --dangerously-skip-permissions'" >> "$HOME/.bashrc"

ENV PATH="/root/.local/bin:${PATH}"

# Install Renode 1.16.0 (ARM64 .NET portable version)
ARG RENODE_VERSION=1.16.0
ARG TARGETARCH
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz \
        && tar -xzf renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz -C /opt \
        && rm renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz; \
    else \
        wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz \
        && tar -xzf renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz -C /opt \
        && rm renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz; \
    fi

# Create symlink for consistent path and install robot dependencies
RUN ln -s /opt/renode_1.16.0-dotnet_portable /opt/renode \
    && pip install -r /opt/renode/tests/requirements.txt --break-system-packages

# Add Renode to PATH
ENV PATH="/opt/renode:${PATH}"

# Set working directory
WORKDIR /workspace

CMD ["/bin/bash"]
