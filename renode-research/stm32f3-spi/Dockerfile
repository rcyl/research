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
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Create python symlink
RUN ln -s /usr/bin/python3 /usr/bin/python

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add ARM target for embedded development
RUN rustup target add thumbv7em-none-eabihf

# Install Renode 1.16.0 (portable version with .NET included)
ARG RENODE_VERSION=1.16.0
ARG TARGETARCH
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz \
        && mkdir -p /opt/renode \
        && tar -xzf renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz -C /opt --strip-components=0 \
        && rm renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz; \
    else \
        wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz \
        && mkdir -p /opt/renode \
        && tar -xzf renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz -C /opt --strip-components=0 \
        && rm renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz; \
    fi

# Add Renode to PATH
ENV PATH="/opt/renode_${RENODE_VERSION}-dotnet_portable:${PATH}"

WORKDIR /workspace

CMD ["/bin/bash"]
