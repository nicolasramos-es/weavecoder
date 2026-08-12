# Weavecoder installer — https://weavecoder.sh/install.ps1
# Downloads the latest wvc binary from GitHub Releases.
# Usage (PowerShell 5.1+):  irm https://weavecoder.sh/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "nicolasramos/weavecoder"
$Version = if ($env:WVC_VERSION) { $env:WVC_VERSION } else { "latest" }

function Get-Arch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") { return "x86_64" }
    if ($arch -eq "ARM64") { return "arm64" }
    throw "Unsupported architecture: $arch"
}

$osName = "windows"
$archName = Get-Arch
$asset = "wvc-$osName-$archName.exe"

if ($Version -eq "latest") {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $release.tag_name
}

$url = "https://github.com/$Repo/releases/download/$Version/$asset"
$installDir = if ($env:WVC_INSTALL_DIR) { $env:WVC_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$out = Join-Path $installDir "wvc.exe"
Write-Host "Weavecoder installer"
Write-Host "  OS:      windows ($archName)"
Write-Host "  Version: $Version"
Write-Host "  URL:     $url"

Write-Host "Downloading..."
Invoke-RestMethod -Uri $url -OutFile $out -UseBasicParsing

# --- SHA-256 checksum verification ---
# Fetch the release JSON and extract the digest for this asset.
$releaseJson = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/tags/$Version"
$digests = $releaseJson.assets | Where-Object { $_.digest } | ForEach-Object { $_.digest }
$expectedDigest = $null
foreach ($d in $digests) {
    if ($d -match '^sha256:([0-9a-f]{64})$') {
        $expectedDigest = $Matches[1]
        break
    }
}

if (-not $expectedDigest) {
    Write-Host "error: Could not obtain SHA-256 digest for $asset from release $Version; aborting for safety" -ForegroundColor Red
    exit 1
}

# Compute local hash
$actualHash = (Get-FileHash -Path $out -Algorithm SHA256).Hash.ToLower()

if ($actualHash -ne $expectedDigest) {
    Write-Host "error: Checksum mismatch for $asset — expected $expectedDigest, got $actualHash" -ForegroundColor Red
    Remove-Item $out -Force
    exit 1
}

Write-Host "SHA-256 verified: $asset" -ForegroundColor Green

# Verify it is the real binary before finishing
$versionOut = & $out --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Remove-Item $out -Force
    throw "Downloaded file is not a valid wvc binary"
}

Write-Host "Installed wvc to $out"
Write-Host ""
Write-Host "Verify with:  wvc --version"
& $out --version
