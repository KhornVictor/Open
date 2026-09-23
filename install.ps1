$ErrorActionPreference = "Stop"

$repo = "KhornVictor/Open"
$version = "v1.0.0"

$installDir = "C:\Tool\Open"
$exePath = "$installDir\Open.exe"
$configPath = "$installDir\apps.toml"

Write-Host "Installing Open..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Close any running instances of Open to prevent file lock
$running = Get-Process -Name "Open" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Closing active Open instance..." -ForegroundColor Yellow
    $running | Stop-Process -Force
    Start-Sleep -Milliseconds 500
}

# Download executable
$url = "https://github.com/$repo/releases/download/$version/Open.exe"
Write-Host "Downloading Open.exe..." -ForegroundColor Cyan

Invoke-WebRequest `
    -Uri $url `
    -OutFile $exePath

# Download default apps.toml if it does not already exist
if (-not (Test-Path $configPath)) {
    Write-Host "Downloading default apps.toml configuration..." -ForegroundColor Cyan
    $configUrl = "https://raw.githubusercontent.com/$repo/main/apps.toml"
    try {
        Invoke-WebRequest -Uri $configUrl -OutFile $configPath
    } catch {
        Write-Host "Note: apps.toml could not be fetched; using built-in defaults." -ForegroundColor Yellow
    }
}

# Add installation directory to user PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($userPath -notlike "*$installDir*") {
    $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $installDir } else { "$userPath;$installDir" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path += ";$installDir"
    Write-Host "Added $installDir to user PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "Installation completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Run 'open' in your terminal to start:"
Write-Host "    open" -ForegroundColor Yellow
Write-Host ""