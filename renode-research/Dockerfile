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

# Install Renode 1.16.0 (ARM64 .NET portable version)
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

# rcyl: Install renode robot dependencies
RUN pip install -r /opt/renode_1.16.0-dotnet_portable/tests/requirements.txt  --break-system-packages

# Add Renode to PATH
ENV PATH="/opt/renode_${RENODE_VERSION}-dotnet_portable:${PATH}"

# Set working directory
WORKDIR /workspace

CMD ["/bin/bash"]
