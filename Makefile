# Installation and build targets for rarc project

.PHONY: install install-binary install-headers install-devel uninstall help

PREFIX ?= /usr/local
GITHUB_OWNER ?= Herbstblatt
GITHUB_REPO ?= rarc

help:
	@echo "Available targets:"
	@echo "  make install             - Install rarc binary and headers (default prefix: /usr/local)"
	@echo "  make install-binary      - Install rarc binary only"
	@echo "  make install-headers     - Install nolibc-rars headers only"
	@echo "  make install-devel       - Install from source (requires Rust toolchain)"
	@echo "  make uninstall           - Remove installed binary and headers"
	@echo ""
	@echo "Options:"
	@echo "  PREFIX=<path>            - Installation prefix (default: /usr/local)"
	@echo "  GITHUB_OWNER=<owner>     - GitHub owner (default: Herbstblatt)"
	@echo "  GITHUB_REPO=<repo>       - GitHub repo name (default: rarc)"
	@echo ""
	@echo "Examples:"
	@echo "  make install PREFIX=/opt/rarc"
	@echo "  make install-devel       # Compile from source"
	@echo "  make uninstall PREFIX=/opt/rarc"

install: install-binary install-headers

install-binary:
	@echo "Installing rarc binary to $(PREFIX)..."
	@./install.sh --prefix "$(PREFIX)" --no-headers --github-owner "$(GITHUB_OWNER)" --github-repo "$(GITHUB_REPO)"

install-headers:
	@echo "Installing nolibc-rars headers to $(PREFIX)/include/rars..."
	@./install.sh --prefix "$(PREFIX)" --github-owner "$(GITHUB_OWNER)" --github-repo "$(GITHUB_REPO)" 2>&1 | grep -v "Downloading.*binary"

install-devel:
	@echo "Building rarc from source..."
	@cd rarc && cargo build --release
	@mkdir -p $(PREFIX)/bin
	@cp target/release/rarc $(PREFIX)/bin/rarc
	@chmod +x $(PREFIX)/bin/rarc
	@echo "Installing nolibc-rars headers..."
	@mkdir -p $(PREFIX)/include/rars
	@cp nolibc-rars/*.h $(PREFIX)/include/rars/
	@echo ""
	@echo "Installation completed!"
	@echo "Binary installed to: $(PREFIX)/bin/rarc"
	@echo "Headers installed to: $(PREFIX)/include/rars"

uninstall:
	@echo "Removing rarc binary from $(PREFIX)/bin..."
	@rm -f $(PREFIX)/bin/rarc
	@echo "Removing headers from $(PREFIX)/include/rars..."
	@rm -rf $(PREFIX)/include/rars
	@echo "Uninstallation completed"

.SILENT: help
