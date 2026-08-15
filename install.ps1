# Weavecoder installer — https://weavecoder.sh/install.ps1
# Downloads the latest wvc binary from GitHub Releases and verifies its SHA-256 checksum.
# Usage (PowerShell 5.1+):  irm https://weavecoder.sh/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "nicolasramos-es/weavecoder"
$Version = if ($env:WVC_VERSION) { $env:WVC_VERSION } else { "latest" }
# Optional auth for private repositories (the product repo is private until launch).
$GithubToken = if ($env:WVC_GITHUB_TOKEN) { $env:WVC_GITHUB_TOKEN } else { $env:GITHUB_TOKEN }

function Get-Arch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") { return "x86_64" }
    if ($arch -eq "ARM64") { return "arm64" }
    throw "Unsupported architecture: $arch"
}

$osName = "windows"
$archName = Get-Arch
$asset = "wvc-$osName-$archName.exe"

# API request headers — add Authorization when a token is provided (private repo)
$headers = @{}
if ($GithubToken) { $headers["Authorization"] = "Bearer $GithubToken" }

# Fetch release metadata (latest or pinned tag) — the same response carries
# the tag name, the asset id, and the published SHA-256 digest.
if ($Version -eq "latest") {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -Headers $headers
    $Version = $release.tag_name
} else {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/tags/$Version" -Headers $headers
}

# Find our asset and its published SHA-256 digest (assets[].digest, "sha256:<hex>")
$assetInfo = $release.assets | Where-Object { $_.name -eq $asset }
if (-not $assetInfo) { throw "Asset '$asset' not found in release $Version" }
$expectedSha = $null
if ($assetInfo.digest -match "^sha256:([0-9a-f]{64})$") { $expectedSha = $Matches[1] }
if (-not $expectedSha) { throw "Could not find a SHA-256 digest for $asset in release $Version; cannot verify the download." }

# Private repos require the API asset endpoint; public repos use the direct URL
if ($GithubToken) {
    $url = "https://api.github.com/repos/$Repo/releases/assets/$($assetInfo.id)"
    $downloadHeaders = @{ Authorization = "Bearer $GithubToken"; Accept = "application/octet-stream" }
} else {
    $url = "https://github.com/$Repo/releases/download/$Version/$asset"
    $downloadHeaders = @{}
}

$installDir = if ($env:WVC_INSTALL_DIR) { $env:WVC_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$out = Join-Path $installDir "wvc.exe"
Write-Host "Weavecoder installer"
Write-Host "  OS:      windows ($archName)"
Write-Host "  Version: $Version"
Write-Host "  URL:     $url"

Write-Host "Downloading..."
Invoke-WebRequest -Uri $url -Headers $downloadHeaders -OutFile $out -UseBasicParsing

# Verify SHA-256 checksum against the digest published with the release
Write-Host "Verifying SHA-256 checksum..."
$actualSha = (Get-FileHash -Algorithm SHA256 -Path $out).Hash.ToLower()
if ($actualSha -ne $expectedSha) {
    Remove-Item $out -Force
    throw "Checksum mismatch for $asset ($Version): expected $expectedSha, got $actualSha. The downloaded file may be corrupted or tampered with; aborting."
}
Write-Host "Checksum OK ($expectedSha)"

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
