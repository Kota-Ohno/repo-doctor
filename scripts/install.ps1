param(
    [string]$Version = $env:REPO_DOCTOR_VERSION,
    [string]$InstallDir = $env:REPO_DOCTOR_INSTALL_DIR,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    @"
Install repo-doctor.

Usage:
  install.ps1 [-Version <tag>] [-InstallDir <path>]

Options:
  -Version     Release tag to install, for example v0.1.0. Default: latest
  -InstallDir  Directory where repo-doctor.exe is installed. Default: ~/.repo-doctor/bin
  -Help        Show this help.
"@ | Write-Host
    exit 0
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = "latest"
}
if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = Join-Path $HOME ".repo-doctor\bin"
}

$target = "x86_64-pc-windows-msvc"
if ($Version -eq "latest") {
    $base = "https://github.com/Kota-Ohno/repo-doctor/releases/latest/download"
} else {
    $base = "https://github.com/Kota-Ohno/repo-doctor/releases/download/$Version"
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$zip = Join-Path $tmp "repo-doctor.zip"
$sum = Join-Path $tmp "repo-doctor.zip.sha256"
Write-Host "Installing repo-doctor $Version for $target"
Invoke-WebRequest "$base/repo-doctor-$target.zip" -OutFile $zip
Invoke-WebRequest "$base/repo-doctor-$target.zip.sha256" -OutFile $sum

$expected = (Get-Content $sum).Split(" ")[0].ToLowerInvariant()
$actual = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -ne $actual) {
    throw "checksum mismatch: expected $expected, got $actual"
}

Expand-Archive -Force $zip -DestinationPath $tmp
Copy-Item -Force (Join-Path $tmp "repo-doctor.exe") (Join-Path $InstallDir "repo-doctor.exe")
Remove-Item -Recurse -Force $tmp

Write-Host "Installed repo-doctor to $(Join-Path $InstallDir 'repo-doctor.exe')"
if (($env:PATH -split [System.IO.Path]::PathSeparator) -notcontains $InstallDir) {
    Write-Host "Add $InstallDir to PATH if repo-doctor is not found by your shell."
}
