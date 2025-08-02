#!/bin/sh
# Marmite Install Script
# 
# Usage:
#   curl -sS https://marmite.blog/install.sh | sh
#   curl -sS https://marmite.blog/install.sh | sh -s -- --bin-dir /custom/bin/folder
#
# Environment variables:
#   MARMITE_BIN_DIR: Override the default binary directory
#   MARMITE_VERSION: Install a specific version (default: latest)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Github repository information
GITHUB_REPO="rochacbruno/marmite"
API_URL="https://api.github.com/repos/$GITHUB_REPO/releases/latest"

# Default installation directory
DEFAULT_BIN_DIR="$HOME/.local/bin"

# Parse command line arguments
BIN_DIR=""
FORCE=false
VERBOSE=false

while [ $# -gt 0 ]; do
    case $1 in
        --bin-dir)
            BIN_DIR="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Marmite Install Script"
            echo ""
            echo "Usage:"
            echo "  curl -sS https://marmite.blog/install.sh | sh"
            echo "  curl -sS https://marmite.blog/install.sh | sh -s -- [options]"
            echo ""
            echo "Options:"
            echo "  --bin-dir <dir>  Install binary to specified directory"
            echo "  --force          Force installation (overwrite existing)"
            echo "  --verbose        Enable verbose output"
            echo "  --help           Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  MARMITE_BIN_DIR  Override the default binary directory"
            echo "  MARMITE_VERSION  Install a specific version (default: latest)"
            exit 0
            ;;
        *)
            echo "${RED}Error: Unknown option $1${NC}"
            exit 1
            ;;
    esac
done

# Use environment variable if BIN_DIR not set via command line
if [ -z "$BIN_DIR" ] && [ -n "$MARMITE_BIN_DIR" ]; then
    BIN_DIR="$MARMITE_BIN_DIR"
fi

# Use default if still not set
if [ -z "$BIN_DIR" ]; then
    BIN_DIR="$DEFAULT_BIN_DIR"
fi

# Logging functions
log() {
    echo "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo "${RED}[ERROR]${NC} $1" >&2
}

verbose_log() {
    if [ "$VERBOSE" = true ]; then
        echo "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect the platform and architecture
detect_platform() {
    local platform
    local arch
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)
            platform="unknown-linux-gnu"
            ;;
        Darwin*)
            platform="apple-darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            platform="pc-windows-msvc"
            ;;
        *)
            log_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${arch}-${platform}"
}

# Download file using available tools
download_file() {
    local url="$1"
    local output="$2"
    
    verbose_log "Downloading $url to $output"
    
    if command_exists curl; then
        curl -fsSL "$url" -o "$output"
    elif command_exists wget; then
        wget -q "$url" -O "$output"
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
}

# Extract archive based on file extension
extract_archive() {
    local archive="$1"
    local destination="$2"
    
    verbose_log "Extracting $archive to $destination"
    
    case "$archive" in
        *.tar.gz)
            if command_exists tar; then
                tar -xzf "$archive" -C "$destination"
            else
                log_error "tar is required to extract .tar.gz files"
                exit 1
            fi
            ;;
        *.zip)
            if command_exists unzip; then
                unzip -q "$archive" -d "$destination"
            else
                log_error "unzip is required to extract .zip files"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported archive format: $archive"
            exit 1
            ;;
    esac
}

# Get latest release information from GitHub API
get_latest_release() {
    local target_platform="$1"
    local api_response
    local asset_name
    local download_url
    
    verbose_log "Fetching latest release information from GitHub API"
    
    # Download release information
    api_response=$(mktemp)
    download_file "$API_URL" "$api_response"
    
    # Find matching asset
    if command_exists grep && command_exists cut; then
        # Parse JSON manually for portability - look for appropriate archive format
        if echo "$target_platform" | grep -q "windows"; then
            # Windows uses .zip files
            asset_name=$(grep '"name":' "$api_response" | grep "$target_platform" | grep '\.zip' | cut -d'"' -f4 | head -n1)
        else
            # Unix-like systems use .tar.gz files
            asset_name=$(grep '"name":' "$api_response" | grep "$target_platform" | grep '\.tar\.gz' | cut -d'"' -f4 | head -n1)
        fi
        
        if [ -z "$asset_name" ]; then
            # Fallback: try to find any asset containing the platform
            asset_name=$(grep '"name":' "$api_response" | grep "$target_platform" | cut -d'"' -f4 | head -n1)
        fi
        
        # Extract download URL from the same asset block (within 30 lines after the name)
        download_url=$(grep -A30 '"name": *"'"$asset_name"'"' "$api_response" | grep '"browser_download_url"' | cut -d'"' -f4 | head -n1)
    else
        log_error "grep and cut are required for JSON parsing"
        rm -f "$api_response"
        exit 1
    fi
    
    rm -f "$api_response"
    
    if [ -z "$asset_name" ] || [ -z "$download_url" ]; then
        log_error "No matching release found for platform: $target_platform"
        exit 1
    fi
    
    verbose_log "Found asset: $asset_name"
    verbose_log "Download URL: $download_url"
    
    echo "$download_url"
}

# Create directory if it doesn't exist
ensure_directory() {
    local dir="$1"
    
    if [ ! -d "$dir" ]; then
        verbose_log "Creating directory: $dir"
        mkdir -p "$dir" || {
            log_error "Failed to create directory: $dir"
            exit 1
        }
    fi
}

# Check if marmite is already installed
check_existing_installation() {
    local bin_path="$1/marmite"
    
    if [ -f "$bin_path" ] && [ "$FORCE" != true ]; then
        log_warn "Marmite is already installed at $bin_path"
        log_warn "Use --force to overwrite or specify a different --bin-dir"
        exit 1
    fi
}

# Install marmite binary
install_marmite() {
    local target_platform
    local download_url
    local temp_dir
    local archive_file
    local binary_name
    local final_binary_path
    
    log "Starting Marmite installation..."
    
    # Detect platform
    target_platform=$(detect_platform)
    log "Detected platform: $target_platform"
    
    # Get download URL
    download_url=$(get_latest_release "$target_platform")
    
    # Check existing installation
    check_existing_installation "$BIN_DIR"
    
    # Create temporary directory
    temp_dir=$(mktemp -d)
    verbose_log "Using temporary directory: $temp_dir"
    
    # Determine archive filename and binary name
    archive_file="$temp_dir/$(basename "$download_url")"
    if echo "$download_url" | grep -q "windows"; then
        binary_name="marmite.exe"
    else
        binary_name="marmite"
    fi
    
    # Download archive
    log "Downloading Marmite..."
    download_file "$download_url" "$archive_file"
    
    # Extract archive
    log "Extracting archive..."
    extract_archive "$archive_file" "$temp_dir"
    
    # Ensure destination directory exists
    ensure_directory "$BIN_DIR"
    
    # Find and move binary
    final_binary_path="$BIN_DIR/marmite"
    if [ -f "$temp_dir/$binary_name" ]; then
        mv "$temp_dir/$binary_name" "$final_binary_path"
    else
        # Search for binary in subdirectories
        binary_found=$(find "$temp_dir" -name "$binary_name" -type f | head -n1)
        if [ -n "$binary_found" ]; then
            mv "$binary_found" "$final_binary_path"
        else
            log_error "Could not find marmite binary in downloaded archive"
            rm -rf "$temp_dir"
            exit 1
        fi
    fi
    
    # Make binary executable
    chmod +x "$final_binary_path"
    
    # Clean up
    rm -rf "$temp_dir"
    
    log_success "Marmite installed successfully to $final_binary_path"
    
    # Check if binary directory is in PATH
    if ! echo "$PATH" | grep -q "$BIN_DIR"; then
        log_warn "Warning: $BIN_DIR is not in your PATH"
        log_warn "Add the following line to your shell profile (e.g., ~/.bashrc, ~/.zshrc):"
        echo "  export PATH=\"$BIN_DIR:\$PATH\""
    fi
    
    # Test installation
    log "Testing installation..."
    if "$final_binary_path" --version >/dev/null 2>&1; then
        version=$("$final_binary_path" --version 2>/dev/null | head -n1)
        log_success "Installation verified: $version"
    else
        log_warn "Installation completed, but version check failed"
    fi
    
    echo ""
    log_success "🎉 Marmite is ready to use!"
    echo ""
    echo "Quick start:"
    echo "  marmite myblog --init-site --name 'My Blog' --tagline 'My thoughts'"
    echo "  marmite myblog --new 'My First Post'"  
    echo "  marmite myblog --serve"
    echo ""
    echo "For more information, visit: https://marmite.blog"
}

# Main execution
main() {
    # Check required tools
    if ! command_exists uname; then
        log_error "uname is required but not available"
        exit 1
    fi
    
    # Start installation
    install_marmite
}

# Run main function
main "$@"