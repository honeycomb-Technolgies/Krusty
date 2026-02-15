#!/bin/sh
set -e

# Krusty installer
# Usage: curl -fsSL https://raw.githubusercontent.com/honeycomb-Technologies/Krusty/main/install.sh | sh

REPO="honeycomb-Technologies/Krusty"
BINARY="krusty"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) PLATFORM="x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) PLATFORM="aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            EXT="tar.gz"
            ;;
        Darwin)
            case "$ARCH" in
                x86_64) PLATFORM="x86_64-apple-darwin" ;;
                arm64) PLATFORM="aarch64-apple-darwin" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            EXT="tar.gz"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="x86_64-pc-windows-msvc"
            EXT="zip"
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -sL "https://api.github.com/repos/$REPO/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
install() {
    detect_platform

    VERSION="${VERSION:-$(get_latest_version)}"
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi

    echo "Installing Krusty $VERSION for $PLATFORM..."

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/krusty-$PLATFORM.$EXT"
    CHECKSUM_URL="$DOWNLOAD_URL.sha256"
    TMP_DIR="$(mktemp -d)"

    echo "Downloading from $DOWNLOAD_URL..."
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/krusty.$EXT"

    # Download and verify checksum if available
    if curl -fsSL "$CHECKSUM_URL" -o "$TMP_DIR/krusty.$EXT.sha256" 2>/dev/null; then
        echo "Verifying checksum..."
        cd "$TMP_DIR"
        if command -v sha256sum >/dev/null 2>&1; then
            if ! sha256sum -c "krusty.$EXT.sha256" >/dev/null 2>&1; then
                echo "Error: Checksum verification failed!"
                echo "The downloaded file may be corrupted. Please try again."
                rm -rf "$TMP_DIR"
                exit 1
            fi
            echo "Checksum verified."
        elif command -v shasum >/dev/null 2>&1; then
            # macOS uses shasum
            if ! shasum -a 256 -c "krusty.$EXT.sha256" >/dev/null 2>&1; then
                echo "Error: Checksum verification failed!"
                echo "The downloaded file may be corrupted. Please try again."
                rm -rf "$TMP_DIR"
                exit 1
            fi
            echo "Checksum verified."
        else
            echo "Warning: No sha256sum or shasum found, skipping verification."
        fi
    else
        echo "Note: No checksum file available for verification."
    fi

    echo "Extracting..."
    cd "$TMP_DIR"
    if [ "$EXT" = "tar.gz" ]; then
        tar xzf "krusty.$EXT"
    else
        unzip -q "krusty.$EXT"
    fi

    echo "Installing to $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
    mv "$BINARY" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY"

    rm -rf "$TMP_DIR"

    echo ""
    echo "Krusty installed successfully!"
    echo ""

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo "Add this to your shell config (.bashrc, .zshrc, etc.):"
            echo ""
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            echo ""
            ;;
    esac

    echo "Run 'krusty' to start."
}

install
