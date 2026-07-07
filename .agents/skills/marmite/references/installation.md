# Marmite Installation Reference

Marmite is distributed as a single binary with no runtime dependencies.

## Quick Install (Recommended)

```bash
curl -sS https://marmite.blog/install.sh | sh
```

Detects OS and architecture, downloads the binary, installs to `~/.local/bin`.

Custom install directory:
```bash
curl -sS https://marmite.blog/install.sh | sh -s -- --bin-dir /usr/local/bin
```

Update to latest:
```bash
curl -sS https://marmite.blog/install.sh | sh -s -- --force
```

## Python (pip / uvx)

```bash
pip install marmite

# Or run without installing
uvx marmite
```

## Rust (cargo)

```bash
# From crates.io
cargo install marmite

# From GitHub source
cargo install --git https://github.com/rochacbruno/marmite

# Pre-compiled binary via cargo-binstall
cargo binstall marmite
```

Update:
```bash
cargo install --force marmite
```

## Package Managers

### macOS (Homebrew)

```bash
brew install marmite
```

### Arch Linux

```bash
pacman -S marmite
```

### FreeBSD

```bash
pkg install marmite
```

## Windows

PowerShell installer:
```powershell
iwr -useb https://marmite.blog/install.ps1 | iex
```

Or download the Windows binary from the [releases page](https://github.com/rochacbruno/marmite/releases/latest), extract, and add to PATH.

## Manual Download

1. Download from https://github.com/rochacbruno/marmite/releases/latest
2. Extract:
   ```bash
   # Linux/macOS
   tar -xzf marmite-*.tar.gz

   # Windows
   unzip marmite-*.zip
   ```
3. Move to a directory in PATH:
   ```bash
   mv marmite ~/.local/bin/
   chmod +x ~/.local/bin/marmite
   ```

## Docker

```bash
# Build a site
docker run --rm -v $(pwd):/input ghcr.io/rochacbruno/marmite

# Build and serve
docker run --rm -p 8000:8000 -v $(pwd):/input ghcr.io/rochacbruno/marmite --serve
```

## Verify Installation

```bash
marmite --version
```

## Troubleshooting

### Command not found

Ensure the install directory is in PATH:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add this line to `~/.bashrc` or `~/.zshrc` to make it permanent.

### Permission denied

Use `--bin-dir` to install to a user-writable directory instead of system paths.

## Uninstall

```bash
# Script install
rm ~/.local/bin/marmite

# Cargo install
cargo uninstall marmite

# Manual install
rm /usr/local/bin/marmite
```
