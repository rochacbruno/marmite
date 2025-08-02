# Marmite Install Script for Windows
# 
# Usage:
#   iwr -useb https://marmite.blog/install.ps1 | iex
#
# Environment variables:
#   MARMITE_BIN_DIR: Override the default binary directory
#   MARMITE_VERSION: Install a specific version (default: latest)

param(
    [string]$BinDir = "",
    [switch]$Force = $false,
    [switch]$Help = $false
)

# Script configuration
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# GitHub repository information
$GitHubRepo = "rochacbruno/marmite"
$ApiUrl = "https://api.github.com/repos/$GitHubRepo/releases/latest"

# Default installation directory
$DefaultBinDir = "$env:LOCALAPPDATA\marmite\bin"

# Colors for output
function Write-Info { Write-Host "[INFO]" -ForegroundColor Blue -NoNewline; Write-Host " $args" }
function Write-Success { Write-Host "[SUCCESS]" -ForegroundColor Green -NoNewline; Write-Host " $args" }
function Write-Warning { Write-Host "[WARN]" -ForegroundColor Yellow -NoNewline; Write-Host " $args" }
function Write-Error { Write-Host "[ERROR]" -ForegroundColor Red -NoNewline; Write-Host " $args" }

# Show help
if ($Help) {
    Write-Host @"
Marmite Install Script for Windows

Usage:
  iwr -useb https://marmite.blog/install.ps1 | iex
  
Parameters:
  -BinDir <path>   Install binary to specified directory
  -Force           Force installation (overwrite existing)
  -Help            Show this help message

Environment variables:
  MARMITE_BIN_DIR  Override the default binary directory
  MARMITE_VERSION  Install a specific version (default: latest)

Examples:
  # Install to default location
  iwr -useb https://marmite.blog/install.ps1 | iex
  
  # Install to custom location
  iwr -useb https://marmite.blog/install.ps1 | iex -BinDir "C:\tools\marmite"
  
  # Force reinstall
  iwr -useb https://marmite.blog/install.ps1 | iex -Force
"@
    exit 0
}

# Use environment variable if BinDir not set via parameter
if ([string]::IsNullOrEmpty($BinDir) -and $env:MARMITE_BIN_DIR) {
    $BinDir = $env:MARMITE_BIN_DIR
}

# Use default if still not set
if ([string]::IsNullOrEmpty($BinDir)) {
    $BinDir = $DefaultBinDir
}

# Ensure absolute path
$BinDir = [System.IO.Path]::GetFullPath($BinDir)

Write-Info "Starting Marmite installation..."

# Check if running as administrator (optional, not required)
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($IsAdmin) {
    Write-Info "Running with administrator privileges"
}

# Detect architecture
$Architecture = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$Platform = "pc-windows-msvc"
$TargetTriple = "$Architecture-$Platform"

Write-Info "Detected platform: $TargetTriple"

# Check existing installation
$MarmitePath = Join-Path $BinDir "marmite.exe"
if ((Test-Path $MarmitePath) -and -not $Force) {
    Write-Warning "Marmite is already installed at $MarmitePath"
    Write-Warning "Use -Force to overwrite or specify a different -BinDir"
    exit 1
}

try {
    # Get latest release information
    Write-Info "Fetching latest release information..."
    $ReleaseInfo = Invoke-RestMethod -Uri $ApiUrl -Headers @{
        "User-Agent" = "marmite-installer"
    }

    # Find matching asset
    $Asset = $ReleaseInfo.assets | Where-Object { 
        $_.name -like "*$TargetTriple*" -and $_.name -like "*.zip"
    } | Select-Object -First 1

    if (-not $Asset) {
        Write-Error "No matching release found for platform: $TargetTriple"
        Write-Error "Available assets:"
        $ReleaseInfo.assets | ForEach-Object { Write-Host "  - $($_.name)" }
        exit 1
    }

    Write-Info "Found asset: $($Asset.name)"
    $DownloadUrl = $Asset.browser_download_url

    # Create temporary directory
    $TempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    Write-Info "Using temporary directory: $TempDir"

    try {
        # Download archive
        $ArchivePath = Join-Path $TempDir $Asset.name
        Write-Info "Downloading Marmite..."
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchivePath -UseBasicParsing

        # Extract archive
        Write-Info "Extracting archive..."
        Expand-Archive -Path $ArchivePath -DestinationPath $TempDir -Force

        # Find marmite.exe
        $ExtractedExe = Get-ChildItem -Path $TempDir -Recurse -Filter "marmite.exe" | Select-Object -First 1
        if (-not $ExtractedExe) {
            Write-Error "Could not find marmite.exe in downloaded archive"
            exit 1
        }

        # Create installation directory
        if (-not (Test-Path $BinDir)) {
            Write-Info "Creating directory: $BinDir"
            New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        }

        # Move binary to installation directory
        Write-Info "Installing marmite.exe to $BinDir"
        Move-Item -Path $ExtractedExe.FullName -Destination $MarmitePath -Force

        Write-Success "Marmite installed successfully to $MarmitePath"

        # Test installation
        Write-Info "Testing installation..."
        $TestResult = & $MarmitePath --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Installation verified: $TestResult"
        } else {
            Write-Warning "Installation completed, but version check failed"
        }

        # Check if directory is in PATH
        $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $MachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        $InUserPath = $UserPath -split ";" | Where-Object { $_ -eq $BinDir }
        $InMachinePath = $MachinePath -split ";" | Where-Object { $_ -eq $BinDir }

        if (-not $InUserPath -and -not $InMachinePath) {
            Write-Warning "$BinDir is not in your PATH"
            Write-Host ""
            Write-Host "To add it to your PATH, run one of the following commands:"
            Write-Host ""
            Write-Host "  # Add to user PATH (recommended):"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$BinDir', 'User')"
            Write-Host ""
            Write-Host "  # Or add to system PATH (requires admin):"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$BinDir', 'Machine')"
            Write-Host ""
            Write-Host "Then restart your terminal or run:"
            Write-Host "  `$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User')"
        }

        Write-Host ""
        Write-Success "🎉 Marmite is ready to use!"
        Write-Host ""
        Write-Host "Quick start:"
        Write-Host "  marmite myblog --init-site --name 'My Blog' --tagline 'My thoughts'"
        Write-Host "  marmite myblog --new 'My First Post'"
        Write-Host "  marmite myblog --serve --watch"
        Write-Host ""
        Write-Host "For more information, visit: https://marmite.blog"

    } finally {
        # Clean up temporary directory
        if (Test-Path $TempDir) {
            Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

} catch {
    Write-Error "Installation failed: $_"
    exit 1
}