$ErrorActionPreference = "Stop"

$repo = "KhornVictor/Open"
$version = "v1.0.0"

$installDir = "$env:LOCALAPPDATA\open-cli"
$exePath = "$installDir\open.exe"

Write-Host "Installing open-cli..."
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Download executable
$url = "https://github.com/$repo/releases/download/$version/open.exe"

Write-Host "Downloading open.exe..."

Invoke-WebRequest `
    -Uri $url `
    -OutFile $exePath

# Add installation directory to user PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($userPath -notlike "*$installDir*") {

    [Environment]::SetEnvironmentVariable(
        "Path",
        "$userPath;$installDir",
        "User"
    )

    Write-Host "Added open-cli to PATH."
}

Write-Host ""
Write-Host "Installation completed!"
Write-Host ""
Write-Host "Restart your terminal and run:"
Write-Host ""
Write-Host "    open"
Write-Host ""