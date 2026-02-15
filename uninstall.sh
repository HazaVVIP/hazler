#!/usr/bin/env bash

set -e

# Hazler Uninstallation Script
# Version: 0.2.0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Uninstall Hazler
uninstall_hazler() {
    print_info "Uninstalling Hazler..."
    
    # Check if cargo is available
    if ! command_exists cargo; then
        print_error "cargo not found. Please install Rust or remove manually."
        return 1
    fi
    
    # Uninstall using cargo
    if cargo uninstall hazler-cli 2>/dev/null; then
        print_success "Hazler uninstalled successfully"
    else
        # Check if binary exists manually
        local binary_path="$HOME/.cargo/bin/hazler"
        if [ -f "$binary_path" ]; then
            print_warning "cargo uninstall failed, removing binary manually..."
            rm -f "$binary_path"
            print_success "Hazler binary removed"
        else
            print_warning "Hazler binary not found, may already be uninstalled"
        fi
    fi
}

# Clean up configuration and cache files
cleanup_files() {
    print_info "Cleaning up configuration and cache files..."
    
    local files_removed=0
    
    # Remove config directory (if exists)
    if [ -d "$HOME/.config/hazler" ]; then
        rm -rf "$HOME/.config/hazler"
        print_info "Removed config directory: ~/.config/hazler"
        files_removed=$((files_removed + 1))
    fi
    
    # Remove cache directory (if exists)
    if [ -d "$HOME/.cache/hazler" ]; then
        rm -rf "$HOME/.cache/hazler"
        print_info "Removed cache directory: ~/.cache/hazler"
        files_removed=$((files_removed + 1))
    fi
    
    # Remove data directory (if exists)
    if [ -d "$HOME/.local/share/hazler" ]; then
        rm -rf "$HOME/.local/share/hazler"
        print_info "Removed data directory: ~/.local/share/hazler"
        files_removed=$((files_removed + 1))
    fi
    
    if [ $files_removed -eq 0 ]; then
        print_info "No configuration or cache files found"
    else
        print_success "Cleaned up $files_removed directories"
    fi
}

# Verify uninstallation
verify_uninstallation() {
    print_info "Verifying uninstallation..."
    
    if command_exists hazler; then
        print_warning "Hazler binary still found in PATH"
        print_warning "You may need to remove it manually"
        return 1
    else
        print_success "Hazler has been completely uninstalled"
    fi
}

# Main uninstallation flow
main() {
    echo ""
    print_info "=================================="
    print_info "  Hazler Uninstallation Script"
    print_info "  Version: 0.2.0"
    print_info "=================================="
    echo ""
    
    # Ask for confirmation
    read -p "Are you sure you want to uninstall Hazler? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi
    echo ""
    
    # Uninstall Hazler
    if ! uninstall_hazler; then
        print_error "Failed to uninstall Hazler"
        exit 1
    fi
    echo ""
    
    # Ask about cleanup
    read -p "Do you want to remove configuration and cache files? (y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        cleanup_files
        echo ""
    fi
    
    # Verify uninstallation
    verify_uninstallation
    
    echo ""
    print_success "=================================="
    print_success "  Uninstallation Complete!"
    print_success "=================================="
    echo ""
    print_info "Thank you for using Hazler!"
    print_info "If you want to reinstall: curl -sSfL https://raw.githubusercontent.com/HazaVVIP/hazler/main/install.sh | bash"
    echo ""
}

# Run main uninstallation
main "$@"
