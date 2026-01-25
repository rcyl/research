FROM localhost/claude:latest

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
    unzip minicom coreutils \
    && rm -rf /var/lib/apt/lists/*

# Add thumb toolchain
RUN rustup target add thumbv7em-none-eabihf

# Create python symlink (required by libopencm3 build)
RUN ln -s /usr/bin/python3 /usr/bin/python

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
