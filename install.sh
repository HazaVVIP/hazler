#!/usr/bin/env bash

set -e

# Hazler Installation Script
# Version: 0.2.0
# Supports: Linux, macOS, Windows (WSL)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SILENT_MODE=false
SKIP_DEPS=false
VERSION="${HAZLER_VERSION:-latest}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --silent|-s)
            SILENT_MODE=true
            shift
            ;;
        --skip-deps)
            SKIP_DEPS=true
            shift
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --help|-h)
            echo "Hazler Installation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --silent, -s       Silent mode (non-interactive)"
            echo "  --skip-deps        Skip dependency installation"
            echo "  --version VERSION  Install specific version"
            echo "  --help, -h         Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Print colored message
print_info() {
    if [ "$SILENT_MODE" = false ]; then
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

print_success() {
    if [ "$SILENT_MODE" = false ]; then
        echo -e "${GREEN}[SUCCESS]${NC} $1"
    fi
}

print_warning() {
    if [ "$SILENT_MODE" = false ]; then
        echo -e "${YELLOW}[WARNING]${NC} $1"
    fi
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""
    
    case "$(uname -s)" in
        Linux*)     os=linux;;
        Darwin*)    os=macos;;
        MINGW*|MSYS*|CYGWIN*)    os=windows;;
        *)          os=unknown;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)  arch=x86_64;;
        aarch64|arm64) arch=aarch64;;
        armv7l)        arch=armv7;;
        *)             arch=unknown;;
    esac
    
    echo "${os}-${arch}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install system dependencies
install_dependencies() {
    if [ "$SKIP_DEPS" = true ]; then
        print_info "Skipping dependency installation (--skip-deps)"
        return 0
    fi
    
    local platform=$1
    local os=$(echo "$platform" | cut -d'-' -f1)
    local arch=$(echo "$platform" | cut -d'-' -f2)
    
    print_info "Installing system dependencies for $os ($arch)..."
    
    case "$os" in
        linux)
            # Detect Linux distribution
            if [ -f /etc/os-release ]; then
                . /etc/os-release
                DISTRO=$ID
            else
                print_error "Cannot detect Linux distribution"
                return 1
            fi
            
            # Check if running as root
            local USE_SUDO=""
            if [ "$(id -u)" -ne 0 ]; then
                if command_exists sudo; then
                    USE_SUDO="sudo"
                else
                    print_error "Not running as root and sudo is not available."
                    print_error "Please run as root or install sudo, then install dependencies manually."
                    return 1
                fi
            fi
            
            case "$DISTRO" in
                ubuntu|debian)
                    print_info "Detected Debian/Ubuntu"
                    $USE_SUDO apt update
                    $USE_SUDO apt install -y build-essential pkg-config libssl-dev
                    ;;
                fedora|rhel|centos)
                    print_info "Detected Fedora/RHEL/CentOS"
                    $USE_SUDO dnf install -y gcc pkg-config openssl-devel
                    ;;
                arch|manjaro)
                    print_info "Detected Arch/Manjaro"
                    $USE_SUDO pacman -S --noconfirm base-devel pkg-config openssl
                    ;;
                *)
                    print_warning "Unsupported Linux distribution: $DISTRO"
                    print_warning "Please install these packages manually:"
                    print_warning "  - build tools (gcc, make, etc.)"
                    print_warning "  - pkg-config"
                    print_warning "  - openssl development libraries"
                    ;;
            esac
            ;;
        macos)
            print_info "Detected macOS"
            # Check if Homebrew is installed
            if ! command_exists brew; then
                print_warning "Homebrew not found. Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            
            # OpenSSL is usually pre-installed on macOS, but ensure we have the latest
            if ! brew list openssl@3 >/dev/null 2>&1; then
                print_info "Installing OpenSSL 3..."
                brew install openssl@3
            else
                print_info "OpenSSL 3 already installed"
            fi
            ;;
        windows)
            print_error "╔════════════════════════════════════════════════════════════════╗"
            print_error "║  Automated installation on Windows is not supported yet       ║"
            print_error "╚════════════════════════════════════════════════════════════════╝"
            echo ""
            print_info "Please follow these manual installation steps:"
            echo ""
            print_info "1. Install Visual Studio Build Tools:"
            print_info "   → https://visualstudio.microsoft.com/downloads/"
            echo ""
            print_info "2. Install OpenSSL for Windows:"
            print_info "   → https://slproweb.com/products/Win32OpenSSL.html"
            echo ""
            print_info "3. Install Rust (if not already installed):"
            print_info "   → https://rustup.rs/"
            echo ""
            print_info "4. Install Hazler using cargo:"
            print_info "   → cargo install --git https://github.com/HazaVVIP/hazler hazler-cli"
            echo ""
            print_warning "After installation, you may need to restart your terminal."
            echo ""
            # Exit explicitly with error code
            exit 1
            ;;
        *)
            print_error "Unsupported operating system"
            return 1
            ;;
    esac
    
    print_success "Dependencies installed successfully"
}

# Check and install Rust
check_rust() {
    print_info "Checking Rust installation..."
    
    if command_exists rustc && command_exists cargo; then
        local rust_version=$(rustc --version | awk '{print $2}')
        print_success "Rust is already installed: $rust_version"
        return 0
    else
        print_warning "Rust is not installed"
        print_info "Installing Rust via rustup..."
        
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source cargo env
        if [ -f "$HOME/.cargo/env" ]; then
            . "$HOME/.cargo/env"
        fi
        
        if command_exists rustc && command_exists cargo; then
            print_success "Rust installed successfully"
        else
            print_error "Failed to install Rust"
            print_error "Please install Rust manually from: https://rustup.rs/"
            return 1
        fi
    fi
}

# Install Hazler
install_hazler() {
    print_info "Installing Hazler..."
    
    # Check if we're in the hazler directory
    if [ -f "Cargo.toml" ] && grep -q "hazler" "Cargo.toml"; then
        print_info "Building and installing from local source..."
        # cargo install handles both compilation and installation efficiently
        # This avoids the double compilation that would occur with 'cargo build' followed by 'cargo install'
        # Use --locked to respect Cargo.lock (important for pinned dependencies like native-tls v0.2.14)
        cargo install --path crates/hazler-cli --locked
    else
        print_info "Installing from GitHub..."
        # Use --locked to respect the Cargo.lock from the repository
        cargo install --git https://github.com/HazaVVIP/hazler hazler-cli --locked
    fi
    
    print_success "Hazler installed successfully"
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    # Ensure cargo bin is in PATH
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if command_exists hazler; then
        local version=$(hazler --version 2>&1 || echo "unknown")
        print_success "Hazler is installed: $version"
        print_success "You can now run: hazler --help"
        
        # Check if cargo bin is in PATH permanently
        if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
            print_warning "Note: ~/.cargo/bin is not in your PATH"
            print_warning "Add this line to your ~/.bashrc or ~/.zshrc:"
            print_warning "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
            print_warning ""
            print_warning "Then run: source ~/.bashrc  (or source ~/.zshrc)"
        fi
    else
        print_error "Hazler installation could not be verified"
        print_error "Please check the installation logs above for errors"
        return 1
    fi
}

# Main installation flow
main() {
    if [ "$SILENT_MODE" = false ]; then
        echo ""
        print_info "================================"
        print_info "  Hazler Installation Script"
        print_info "  Version: 0.2.0"
        print_info "================================"
        echo ""
    fi
    
    # Detect platform
    PLATFORM=$(detect_platform)
    OS=$(echo "$PLATFORM" | cut -d'-' -f1)
    print_info "Detected platform: $PLATFORM"
    if [ "$SILENT_MODE" = false ]; then
        echo ""
    fi
    
    # Install dependencies
    if ! install_dependencies "$PLATFORM"; then
        print_error "Failed to install dependencies"
        exit 1
    fi
    if [ "$SILENT_MODE" = false ]; then
        echo ""
    fi
    
    # Check/install Rust
    if ! check_rust; then
        print_error "Failed to set up Rust"
        exit 1
    fi
    if [ "$SILENT_MODE" = false ]; then
        echo ""
    fi
    
    # Install Hazler
    if ! install_hazler; then
        print_error "Failed to install Hazler"
        exit 1
    fi
    if [ "$SILENT_MODE" = false ]; then
        echo ""
    fi
    
    # Verify installation
    if ! verify_installation; then
        print_error "Installation verification failed"
        exit 1
    fi
    
    if [ "$SILENT_MODE" = false ]; then
        echo ""
        print_success "================================"
        print_success "  Installation Complete! 🎉"
        print_success "================================"
        echo ""
        print_info "Quick start:"
        print_info "  hazler --help"
        print_info "  hazler https://example.com"
        echo ""
    else
        print_success "Hazler installed successfully"
    fi
}

# Run main installation
main "$@"
