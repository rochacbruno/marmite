---
title: Installation
slug: installation
date: 2025-08-02 18:02:50
tags: [installation, docs, guide]
authors: [marmite]
description: How to install Marmite - a fast, minimal static site generator
---

<!-- .install -->

## Additional Details

### Install Script Options

The install script will detect your operating system and architecture, download the appropriate binary, install it to `~/.local/bin` (or a custom directory), and verify the installation.

To install to a custom directory:

```bash
curl -sS https://marmite.blog/install.sh | sh -s -- --bin-dir /usr/local/bin
```

You can also use environment variables:

```bash
export MARMITE_BIN_DIR=/custom/path
curl -sS https://marmite.blog/install.sh | sh
```

### Install from Source

```bash
# Or install from GitHub directly
cargo install --git https://github.com/rochacbruno/marmite
```

Check [[Python Installation with pip]] for more details on the pip/uvx method.

### Manual Installation

1. Download the appropriate binary for your platform from the [releases page](https://github.com/rochacbruno/marmite/releases/latest)

2. Extract the archive:
   ```bash
   # For Linux/macOS
   tar -xzf marmite-*.tar.gz
   
   # For Windows
   unzip marmite-*.zip
   ```

3. Move the binary to a directory in your PATH:
   ```bash
   # Linux/macOS
   sudo mv marmite /usr/local/bin/
   
   # Or to user directory (no sudo required)
   mkdir -p ~/.local/bin
   mv marmite ~/.local/bin/
   ```

4. Make sure the binary is executable (Linux/macOS):
   ```bash
   chmod +x /usr/local/bin/marmite
   ```

### Windows

1. Download the Windows binary from the [releases page](https://github.com/rochacbruno/marmite/releases/latest)
2. Extract the ZIP file
3. Add the directory containing `marmite.exe` to your PATH

Or use PowerShell:
```powershell
iwr -useb https://marmite.blog/install.ps1 | iex
```

## Verify Installation

After installation, verify that Marmite is working:

```bash
marmite --version
```

You should see output like:
```
marmite 0.2.6
```

## Updating Marmite

To update to the latest version:

### Using the install script
```bash
curl -sS https://marmite.blog/install.sh | sh -s -- --force
```

### Using cargo
```bash
cargo install --force marmite
```

## Troubleshooting

### Command not found

If you get a "command not found" error, make sure the installation directory is in your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### Permission denied

If you get permission errors during installation:
- Use the `--bin-dir` option to install to a user-writable directory
- Or run the install command with `sudo` (not recommended)

### SSL/TLS errors

If you encounter SSL errors when downloading:
```bash
# Use wget instead of curl
wget -O- https://marmite.blog/install.sh | sh
```

## Next Steps

Once Marmite is installed, you can:

1. Create your first site:
   ```bash
   marmite myblog --init-site --name "My Blog"
   ```

2. Create your first post:
   ```bash
   marmite myblog --new "Hello World"
   ```

3. Build and serve your site:
   ```bash
   marmite myblog --build --serve
   ```

Check out the [Getting Started](/getting-started.html) guide for a complete tutorial!

## Uninstalling

To uninstall Marmite:

```bash
# If installed with the script
rm ~/.local/bin/marmite

# If installed with cargo
cargo uninstall marmite

# If installed manually
rm /usr/local/bin/marmite
```