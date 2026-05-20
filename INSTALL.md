# Installation Guide for rarc

This document describes how to install the `rarc` compiler and `nolibc-rars` headers.

## Quick Start

### Using Pre-built Binaries (Recommended)

The easiest way to install is to use the pre-built binaries from GitHub:

```bash
./install.sh
```

This will:
- Download the latest `rarc` binary for your system architecture
- Install it to `/usr/local/bin/rarc`
- Download and install `nolibc-rars` headers to `/usr/local/include/rars`

**Note:** The script requires either `curl` or `wget` to be installed.

### Using Makefile

If you prefer using Make:

```bash
make install
```

### Custom Installation Prefix

To install to a custom location (e.g., `/opt/rarc`):

```bash
./install.sh --prefix /opt/rarc
```

Or with Make:

```bash
make install PREFIX=/opt/rarc
```

## Installation Methods

### Method 1: Pre-built Binaries (Recommended)

**Advantages:**
- Fast and simple
- No compilation required
- Cross-platform support (Linux x86_64, macOS x86_64)

**Requirements:**
- `curl` or `wget`
- `tar`
- `bash` or compatible shell

**Usage:**

```bash
# Install to /usr/local (may require sudo)
./install.sh

# Or with custom prefix
./install.sh --prefix ~/.local

# Install binary only (skip headers)
./install.sh --no-headers

# Install from custom GitHub repository
./install.sh --github-owner MyOrg --github-repo my-rarc-fork
```

### Method 2: Building from Source

**Advantages:**
- Latest development version
- Customizable build options
- No dependency on GitHub releases

**Requirements:**
- Rust toolchain (1.70+)
- `cargo`
- `git`

**Usage:**

```bash
# Build release binary
cargo build -p rarc --release

# Install to system
make -f Makefile.install install-devel PREFIX=/usr/local

# Or manually install
sudo cp target/release/rarc /usr/local/bin/
sudo mkdir -p /usr/local/include/rars
sudo cp nolibc-rars/*.h /usr/local/include/rars/
```

## Verifying Installation

After installation, verify everything is set up correctly:

```bash
# Check if rarc is installed and in PATH
which rarc
rarc --version  # May not work if --version not implemented

# Check if headers are installed
ls /usr/local/include/rars/

# Expected files:
# - ctype.h
# - errno.h
# - getopt.h
# - math.h
# - nolibc.h
# - rars.h
# - stdarg.h
# - stddef.h
# - stdint.h
# - stdio.h
# - stdlib.h
# - string.h
# - time.h
# - types.h
```

## Troubleshooting

### "rarc: command not found"

The binary was installed to a location not in your `$PATH`. Add it:

```bash
# If installed to /usr/local/bin:
export PATH="/usr/local/bin:$PATH"

# If installed to custom location:
export PATH="/your/custom/prefix/bin:$PATH"

# Add to shell profile for persistence (~/.bashrc, ~/.zshrc, etc.):
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
```

### "curl: command not found" or "wget: command not found"

Install one of these tools:

```bash
# Ubuntu/Debian
sudo apt-get install curl

# CentOS/RHEL
sudo yum install curl

# macOS
brew install curl

# Or use wget:
# Ubuntu/Debian
sudo apt-get install wget
```

### "Could not determine latest release"

Network issue connecting to GitHub API. Possible causes:
- No internet connection
- GitHub is blocked by firewall
- GitHub API rate limiting (try again later)

Check connection:

```bash
curl -I https://api.github.com/repos/Herbstblatt/rarc/releases/latest
```

### "Headers not installed"

If headers installation failed but binary installed:

```bash
# Install headers separately
./install.sh --prefix /usr/local --no-headers

# Or manually:
mkdir -p /usr/local/include/rars
curl -L https://github.com/Herbstblatt/nolibc-rars/archive/refs/heads/main.tar.gz | \
  tar xz --strip-components=1 -C /usr/local/include/rars --wildcards '*/[*.h'
```

### "Unsupported architecture"

If your system is not supported, you can:
1. Build from source (see Method 2)
2. Request binary support for your architecture on GitHub
3. Manually cross-compile for your target

## Using rarc with Installed Headers

Once installed, you can use rarc with the headers in your C code:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

int main() {
    printf("Hello from RARS!\n");
    return 0;
}
```

Compile with rarc:

```bash
rarc program.c -o program.s
```

## Supported Architectures

The installation script auto-detects your system and downloads the appropriate binary:

| OS      | Architectures       |
|---------|-------------------|
| Linux   | x86_64 |
| macOS   | x86_64 |

For unsupported architectures, build from source.

## Uninstalling

To remove the installation:

```bash
# Remove binary
rm /usr/local/bin/rarc

# Remove headers
rm -rf /usr/local/include/rars

# Or use Makefile:
make uninstall PREFIX=/usr/local
```
