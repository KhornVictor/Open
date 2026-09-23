param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$repo = "KhornVictor/Open"
$installDir = "C:\Tool\Open"
$exePath = "$installDir\Open.exe"
$configPath = "$installDir\apps.toml"

Write-Host "Installing Open..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$running = Get-Process -Name "Open" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Closing active Open instance..." -ForegroundColor Yellow
    $running | Stop-Process -Force
    Start-Sleep -Milliseconds 500
}

$url = if ($Version -eq "latest") {
    "https://github.com/$repo/releases/latest/download/Open.exe"
} else {
    "https://github.com/$repo/releases/download/$Version/Open.exe"
}

Write-Host "Downloading Open.exe ($Version)..." -ForegroundColor Cyan

try {
    Invoke-WebRequest -Uri $url -OutFile $exePath -UseBasicParsing
} catch {
    Write-Host ""
    Write-Host "Error: Failed to download Open.exe from $url" -ForegroundColor Red
    Write-Host "Details: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $configPath)) {
    Write-Host "Downloading default apps.toml configuration..." -ForegroundColor Cyan
    $configUrl = "https://raw.githubusercontent.com/$repo/main/apps.toml"
    try {
        Invoke-WebRequest -Uri $configUrl -OutFile $configPath -UseBasicParsing
    } catch {
        Write-Host "Note: apps.toml could not be fetched; using built-in defaults." -ForegroundColor Yellow
    }
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$pathList = if ([string]::IsNullOrWhiteSpace($userPath)) { @() } else { $userPath -split ';' }

if ($pathList -notcontains $installDir) {
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
