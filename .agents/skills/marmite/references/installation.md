# Marmite Installation Reference

Marmite is distributed as a single binary with no runtime dependencies.

**Never pipe a remote install script into a shell** (`curl -sS <url> | sh`, `wget -O- <url> | sh`, `iwr -useb <url> | iex`). Piping executes whatever the server returns — before any review or integrity check — with the user's privileges. A compromised site, DNS hijack, or MITM response means arbitrary code execution, potentially inside a project repository that holds credentials or deployment secrets. Prefer a package manager; if the script installer is used, follow the [verified script install](#script-installer-verified) flow below.

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
cargo install --git https://github.com/rochacbruno/marmite marmite

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

## Docker

```bash
# Build a site
docker run --rm -v $(pwd):/input ghcr.io/rochacbruno/marmite

# Build and serve
docker run --rm -p 8000:8000 -v $(pwd):/input ghcr.io/rochacbruno/marmite --serve
```

## Manual Download

1. Download from https://github.com/rochacbruno/marmite/releases/latest
2. Verify the archive against the published checksums:
   ```bash
   curl -sSLO https://github.com/rochacbruno/marmite/releases/latest/download/SHA256SUMS
   sha256sum -c --ignore-missing SHA256SUMS
   ```
3. Extract:
   ```bash
   # Linux/macOS
   tar -xzf marmite-*.tar.gz

   # Windows
   unzip marmite-*.zip
   ```
4. Move to a directory in PATH:
   ```bash
   mv marmite ~/.local/bin/
   chmod +x ~/.local/bin/marmite
   ```

## Windows

`pip install marmite`, or download the Windows `.zip` from the [releases page](https://github.com/rochacbruno/marmite/releases/latest), verify it against `SHA256SUMS`, extract, and add to PATH.

## Script Installer (Verified)

The install script detects OS and architecture, downloads the binary, and installs to `~/.local/bin`. The canonical source is [`example/scripts/install.sh`](https://github.com/rochacbruno/marmite/blob/main/example/scripts/install.sh) in the GitHub repository (served from `https://marmite.blog/install.sh`).

Use it only with a download-then-verify flow — download the script to a file, check it, and only then execute it:

```bash
# 1. Download (do NOT pipe to sh)
curl -sSLo /tmp/marmite-install.sh https://marmite.blog/install.sh

# 2. Verify it matches the copy in the repository
curl -sSLo /tmp/marmite-install.sh.ref \
  https://raw.githubusercontent.com/rochacbruno/marmite/main/example/scripts/install.sh
diff /tmp/marmite-install.sh /tmp/marmite-install.sh.ref \
  || { echo "install.sh does not match the repository copy"; exit 1; }

# 3. Review what will run (recommended)
less /tmp/marmite-install.sh

# 4. Execute
sh /tmp/marmite-install.sh
```

Options (passed after the script path):

```bash
# Custom install directory
sh /tmp/marmite-install.sh --bin-dir /usr/local/bin

# Update to latest
sh /tmp/marmite-install.sh --force
```

PowerShell (`install.ps1`) works the same way — download, review, then run:

```powershell
Invoke-WebRequest -UseBasicParsing -OutFile marmite-install.ps1 https://marmite.blog/install.ps1
Get-Content marmite-install.ps1   # review
.\marmite-install.ps1
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
