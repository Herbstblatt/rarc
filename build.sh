#!/bin/bash
#
# Local build script for creating rarc release archives
# This script builds rarc for the current platform and creates distribution packages
#
# Usage: ./build.sh [--all] [--target TARGET] [--help]
#
# Options:
#   --all       Build for all supported targets (requires cross-compilation setup)
#   --target T  Build only for specific target (e.g., x86_64-unknown-linux-gnu)
#   --help      Show this help message

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR"
INSTALL_DIR="$PROJECT_DIR/install.sh"
BUILD_DIR="${PROJECT_DIR}/target/release"
DIST_DIR="${PROJECT_DIR}/dist"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Parse arguments
BUILD_ALL=false
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            BUILD_ALL=true
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --help)
            head -13 "$0"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Determine OS and architecture
get_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

get_arch() {
    case "$(uname -m)" in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l|armv7)
            echo "armv7"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Build for a specific target
build_target() {
    local target=$1
    
    log_info "Building for target: $target"
    
    cargo build -p rarc --release --target "$target" 2>&1 | tail -20
    
    if [ ! -f "${BUILD_DIR}/../${target}/release/rarc" ]; then
        log_error "Build failed for $target"
        return 1
    fi
    
    log_info "✓ Build successful for $target"
}

# Create distribution archive
create_archive() {
    local target=$1
    local os=$2
    local arch=$3
    
    log_info "Creating distribution archive for $os-$arch"
    
    local binary_path
    if [ "$target" = "native" ]; then
        binary_path="$BUILD_DIR/rarc"
    else
        binary_path="${BUILD_DIR}/../${target}/release/rarc"
    fi
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found at $binary_path"
        return 1
    fi
    
    # Create temp directory for archive
    local temp_dir="/tmp/rarc-${os}-${arch}"
    rm -rf "$temp_dir"
    mkdir -p "$temp_dir/bin"
    
    # Copy binary and documentation
    cp "$binary_path" "$temp_dir/bin/rarc"
    chmod +x "$temp_dir/bin/rarc"
    cp "$PROJECT_DIR/INSTALL.md" "$temp_dir/" 2>/dev/null || true
    cp "$PROJECT_DIR/README.md" "$temp_dir/" 2>/dev/null || true
    
    # Create archive
    mkdir -p "$DIST_DIR"
    tar -czf "${DIST_DIR}/rarc-${os}-${arch}.tar.gz" -C /tmp "rarc-${os}-${arch}"
    
    log_info "✓ Archive created: ${DIST_DIR}/rarc-${os}-${arch}.tar.gz"
    
    # Cleanup
    rm -rf "$temp_dir"
}

# Main build flow
main() {
    log_info "Building rarc release archives"
    
    if [ ! -x "$INSTALL_DIR" ]; then
        log_warn "install.sh not found or not executable"
    fi
    
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    if [ "$BUILD_ALL" = true ]; then
        log_info "Building for all supported targets..."
        
        local targets=(
            "x86_64-unknown-linux-gnu"
            "aarch64-unknown-linux-gnu"
            "armv7-unknown-linux-gnueabihf"
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
        )
        
        for target in "${targets[@]}"; do
            # Check if target is installed
            if ! rustup target list | grep -q "^${target}"; then
                log_warn "Target $target not installed. Installing..."
                rustup target add "$target"
            fi
            
            # For cross-compilation, check if cross is available
            if [[ "$target" == *"gnu"* ]] || [[ "$target" == *"darwin"* ]]; then
                if ! echo "$target" | grep -q "$(rustup target list --installed | grep -E 'x86_64-unknown-linux|aarch64-unknown-linux|armv7-unknown-linux|.*-apple-darwin')"; then
                    log_warn "Skipping $target (requires cross-compilation setup)"
                    continue
                fi
            fi
            
            build_target "$target" || log_warn "Build failed for $target, skipping archive..."
        done
    elif [ -n "$TARGET" ]; then
        log_info "Building for target: $TARGET"
        build_target "$TARGET"
    else
        log_info "Building for native target"
        cargo build -p rarc --release
    fi
    
    # Create archives
    log_info "Creating distribution archives..."
    
    local os
    local arch
    os=$(get_os)
    arch=$(get_arch)
    
    if [ "$BUILD_ALL" = true ]; then
        log_info "Creating archives for all built targets..."
        # This would need more sophisticated detection
        # For now, just create for the native build
        create_archive "native" "$os" "$arch"
    elif [ -n "$TARGET" ]; then
        # Extract OS and arch from target
        if [[ "$TARGET" == *"linux"* ]]; then
            os="linux"
        elif [[ "$TARGET" == *"darwin"* ]]; then
            os="macos"
        fi
        
        if [[ "$TARGET" == *"x86_64"* ]]; then
            arch="x86_64"
        elif [[ "$TARGET" == *"aarch64"* ]]; then
            arch="aarch64"
        elif [[ "$TARGET" == *"armv7"* ]]; then
            arch="armv7"
        fi
        
        create_archive "$TARGET" "$os" "$arch"
    else
        create_archive "native" "$os" "$arch"
    fi
    
    log_info ""
    log_info "Build complete!"
    log_info "Archives available in: $DIST_DIR"
    ls -lh "$DIST_DIR"/*.tar.gz 2>/dev/null || log_warn "No archives found"
}

main "$@"
