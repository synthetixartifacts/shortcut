# ShortCut Build Container
# Used for compiling Tauri apps, NOT for running them
# Desktop apps need native GUI access which Docker cannot provide

FROM rust:1.83-bookworm

# Install Node.js 20
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install Tauri dependencies for Linux builds
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libxdo-dev \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Tauri CLI
RUN cargo install tauri-cli

WORKDIR /app

# Copy package files first for better caching
COPY package*.json ./
RUN npm install

# Copy Rust files for dependency caching
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
COPY src-tauri/build.rs src-tauri/build.rs

# Create dummy source files to compile dependencies
RUN mkdir -p src-tauri/src && \
    echo 'fn main() {}' > src-tauri/src/main.rs && \
    echo 'pub fn run() {}' > src-tauri/src/lib.rs

# Pre-compile Rust dependencies (this takes a while but is cached)
WORKDIR /app/src-tauri
RUN cargo build --release 2>/dev/null || true

# Back to app root
WORKDIR /app

# Copy the rest of the application
COPY . .

# Default command: build the application
CMD ["npm", "run", "tauri", "build"]
