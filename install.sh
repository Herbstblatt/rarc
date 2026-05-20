#!/bin/bash
#
# Installation script for rarc compiler and nolibc-rars headers
# This script downloads and installs:
#   1. The rarc binary from GitHub releases
#   2. The nolibc-rars headers to /usr/local/include/rars
#
# Usage: ./install.sh [--prefix PREFIX] [--no-headers] [--github-owner OWNER] [--github-repo REPO]
#
# Options:
#   --prefix PREFIX      Installation prefix (default: /usr/local)
#   --no-headers         Skip header installation
#   --github-owner OWNER GitHub owner (default: Herbstblatt)
#   --github-repo REPO   GitHub repo for binary (default: rarc)
#   --help              Show this help message

set -e

# Configuration
GITHUB_OWNER="${GITHUB_OWNER:-Herbstblatt}"
GITHUB_REPO="${GITHUB_REPO:-rarc}"
GITHUB_HEADERS_OWNER="Herbstblatt"
GITHUB_HEADERS_REPO="nolibc-rars"
PREFIX="${PREFIX:-/usr/local}"
INSTALL_HEADERS=true

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --no-headers)
            INSTALL_HEADERS=false
            shift
            ;;
        --github-owner)
            GITHUB_OWNER="$2"
            shift 2
            ;;
        --github-repo)
            GITHUB_REPO="$2"
            shift 2
            ;;
        --help)
            head -20 "$0"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}" >&2
            exit 1
            ;;
    esac
done

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    if ! command -v tar &> /dev/null; then
        log_error "tar command not found. Please install tar."
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

# Detect system architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    
    case "$arch" in
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
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        log_error "Unsupported OS: $OSTYPE"
        exit 1
    fi
}

# Download file using curl or wget
download_file() {
    local url=$1
    local output=$2
    
    if command -v curl &> /dev/null; then
        curl -L -f --progress-bar "$url" -o "$output"
    else
        wget --progress=dot:giga "$url" -O "$output"
    fi
}

# Get latest release tag
get_latest_release() {
    local owner=$1
    local repo=$2
    local api_url="https://api.github.com/repos/${owner}/${repo}/releases/latest"
    
    if command -v curl &> /dev/null; then
        curl -s "$api_url" | grep -oP '"tag_name": "\K(.*)(?=")'
    else
        wget -qO- "$api_url" | grep -oP '"tag_name": "\K(.*)(?=")'
    fi
}

# Install rarc binary
install_binary() {
    local os arch tag
    os=$(detect_os)
    arch=$(detect_arch)
    
    log_info "Detecting latest release..."
    tag=$(get_latest_release "$GITHUB_OWNER" "$GITHUB_REPO")
    
    if [ -z "$tag" ]; then
        log_error "Could not determine latest release"
        exit 1
    fi
    
    log_info "Latest release: $tag"
    
    # Construct download URL
    local filename="rarc-${os}-${arch}.tar.gz"
    local download_url="https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/${tag}/${filename}"
    
    log_info "Downloading from: $download_url"
    
    # Create temporary directory
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT
    
    local archive="$tmpdir/$filename"
    download_file "$download_url" "$archive"
    
    log_info "Extracting archive..."
    tar -xzf "$archive" -C "$tmpdir"
    
    # Find the binary (could be at different paths depending on archive structure)
    local binary
    if [ -f "$tmpdir/rarc" ]; then
        binary="$tmpdir/rarc"
    elif [ -f "$tmpdir/bin/rarc" ]; then
        binary="$tmpdir/bin/rarc"
    else
        log_error "Could not find rarc binary in archive"
        exit 1
    fi
    
    # Create bin directory if needed
    mkdir -p "$PREFIX/bin"
    
    log_info "Installing binary to $PREFIX/bin/rarc"
    cp "$binary" "$PREFIX/bin/rarc"
    chmod +x "$PREFIX/bin/rarc"
    
    log_info "Binary installed successfully"
    
    # Check if PREFIX/bin is in PATH
    if [[ ":$PATH:" != *":$PREFIX/bin:"* ]]; then
        log_warn "Note: $PREFIX/bin is not in your PATH"
        log_warn "Consider adding it to your PATH or use the full path: $PREFIX/bin/rarc"
    fi
}

# Install headers
install_headers() {
    if [ "$INSTALL_HEADERS" = false ]; then
        log_info "Skipping header installation (--no-headers flag set)"
        return
    fi
    
    log_info "Installing nolibc-rars headers..."
    
    local tag
    tag=$(get_latest_release "$GITHUB_HEADERS_OWNER" "$GITHUB_HEADERS_REPO")
    
    if [ -z "$tag" ]; then
        log_error "Could not determine latest release for nolibc-rars"
        exit 1
    fi
    
    log_info "Latest nolibc-rars release: $tag"
    
    # Construct download URL for headers archive
    local download_url="https://github.com/${GITHUB_HEADERS_OWNER}/${GITHUB_HEADERS_REPO}/archive/refs/tags/${tag}.tar.gz"
    
    log_info "Downloading from: $download_url"
    
    # Create temporary directory
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT
    
    local archive="$tmpdir/nolibc-rars-${tag}.tar.gz"
    download_file "$download_url" "$archive"
    
    log_info "Extracting archive..."
    tar -xzf "$archive" -C "$tmpdir"
    
    # Create include directory
    mkdir -p "$PREFIX/include/rars"
    
    # Find the extracted directory (typically nolibc-rars-VERSION)
    local extract_dir
    extract_dir=$(find "$tmpdir" -maxdepth 1 -type d -name "nolibc-rars-*" | head -1)
    
    if [ -z "$extract_dir" ]; then
        log_error "Could not find extracted nolibc-rars directory"
        exit 1
    fi
    
    # Copy header files
    log_info "Copying header files to $PREFIX/include/rars"
    cp "$extract_dir"/*.h "$PREFIX/include/rars/"
    
    log_info "Headers installed successfully"
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    if ! [ -x "$PREFIX/bin/rarc" ]; then
        log_warn "rarc binary not found or not executable at $PREFIX/bin/rarc"
    else
        log_info "✓ rarc binary found at $PREFIX/bin/rarc"
        rarc_version=$("$PREFIX/bin/rarc" --version 2>/dev/null || echo "unknown")
        log_info "  Version: $rarc_version"
    fi
    
    if [ "$INSTALL_HEADERS" = true ]; then
        if [ -d "$PREFIX/include/rars" ]; then
            local header_count
            header_count=$(ls "$PREFIX/include/rars"/*.h 2>/dev/null | wc -l)
            log_info "✓ Headers directory found at $PREFIX/include/rars with $header_count header files"
        else
            log_warn "Headers directory not found at $PREFIX/include/rars"
        fi
    fi
}

# Main installation flow
main() {
    log_info "Starting installation of rarc and nolibc-rars headers"
    log_info "Installation prefix: $PREFIX"
    
    check_prerequisites
    install_binary
    install_headers
    verify_installation
    
    log_info ""
    log_info "Installation completed successfully!"
    log_info "To use rarc in your PATH, add the following to your shell profile:"
    log_info "  export PATH=\"$PREFIX/bin:\$PATH\""
}

main "$@"
