#Requires -Version 5.1
<#
.SYNOPSIS
    Download a standalone Python build and install project dependencies
    into it. No system Python installation required.

.DESCRIPTION
    Windows-native PowerShell port of setup_vendor.sh. Downloads a portable
    CPython build from python-build-standalone, extracts it to vendor/python/,
    installs pip packages from requirements.txt, writes .cargo/config.toml so
    PyO3 links against the vendored interpreter, and copies Bokeh JS assets
    to vendor/bokeh/ for offline rendering.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\setup_vendor.ps1
#>

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

# -- Configuration ----------------------------------------------------------

$PythonVersion = '3.12.8'
$ReleaseTag    = '20250106'
$BokehVersion  = '3.9.0'
$BaseUrl       = "https://github.com/indygreg/python-build-standalone/releases/download/$ReleaseTag"

# -- Resolve paths ----------------------------------------------------------

$ScriptDir   = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir  = Resolve-Path (Join-Path $ScriptDir '..') | Select-Object -ExpandProperty Path
$VendorDir   = Join-Path $ProjectDir 'vendor\python'
$PythonExeRel = 'vendor/python/python.exe'
$PythonAbs   = Join-Path $ProjectDir 'vendor\python\python.exe'

# -- Detect architecture ----------------------------------------------------

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    'AMD64' { 'x86_64' }
    'ARM64' { 'aarch64' }
    default { throw "Unsupported architecture: $($env:PROCESSOR_ARCHITECTURE)" }
}

$Platform = "$arch-pc-windows-msvc"
$Archive  = "cpython-$PythonVersion+$ReleaseTag-$Platform-shared-install_only.tar.gz"
$Url      = "$BaseUrl/$Archive"

# -- Download & extract -----------------------------------------------------

if (Test-Path $VendorDir) {
    Write-Host 'vendor/python/ already exists - skipping download.'
    Write-Host '  (Delete vendor/python/ and re-run to force a fresh download.)'
} else {
    Write-Host "Downloading standalone Python $PythonVersion for $Platform..."
    Write-Host "  URL: $Url"

    $tmpFile = [System.IO.Path]::GetTempFileName()
    try {
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $Url -OutFile $tmpFile -UseBasicParsing
        $ProgressPreference = 'Continue'

        Write-Host 'Extracting to vendor/python/...'
        $vendorRoot = Join-Path $ProjectDir 'vendor'
        New-Item -ItemType Directory -Force -Path $vendorRoot | Out-Null

        # tar.exe ships with Windows 10 1803+ and handles .tar.gz natively.
        & tar.exe -xzf $tmpFile -C $vendorRoot
        if ($LASTEXITCODE -ne 0) {
            throw "tar.exe failed with exit code $LASTEXITCODE"
        }
    } finally {
        if (Test-Path $tmpFile) { Remove-Item -Force $tmpFile }
    }

    Write-Host 'Python extracted to vendor/python/'
}

if (-not (Test-Path $PythonAbs)) {
    throw "Expected Python at $PythonAbs but it does not exist. Check vendor/python/ contents."
}

Write-Host "Using Python: $PythonAbs"
& $PythonAbs --version

# -- Install pip packages ---------------------------------------------------

Write-Host 'Bootstrapping pip...'
& $PythonAbs -m ensurepip --upgrade 2>$null

Write-Host 'Installing dependencies from requirements.txt...'
& $PythonAbs -m pip install --upgrade pip setuptools wheel -q
if ($LASTEXITCODE -ne 0) { throw "pip upgrade failed ($LASTEXITCODE)" }

& $PythonAbs -m pip install -r (Join-Path $ProjectDir 'requirements.txt') -q
if ($LASTEXITCODE -ne 0) { throw "pip install failed ($LASTEXITCODE)" }

Write-Host 'Installed packages:'
& $PythonAbs -m pip list --format=columns

# -- Write .cargo/config.toml ----------------------------------------------

$cargoDir = Join-Path $ProjectDir '.cargo'
New-Item -ItemType Directory -Force -Path $cargoDir | Out-Null

$cargoConfig = @"
[env]
PYO3_PYTHON = { value = "$PythonExeRel", relative = true }
PYTHONHOME  = { value = "vendor/python", relative = true }
"@

$cargoConfigPath = Join-Path $cargoDir 'config.toml'
Set-Content -Path $cargoConfigPath -Value $cargoConfig -Encoding UTF8 -NoNewline

Write-Host ''
Write-Host "Wrote .cargo/config.toml with PYO3_PYTHON = $PythonExeRel"

# -- Copy Bokeh JS from installed package for offline rendering ------------

$BokehVendorDir = Join-Path $ProjectDir 'vendor\bokeh'
$BokehCopyMap = @{
    'bokeh.min.js'         = "bokeh-$BokehVersion.min.js"
    'bokeh-widgets.min.js' = "bokeh-widgets-$BokehVersion.min.js"
}

$needCopy = $false
foreach ($destName in $BokehCopyMap.Values) {
    if (-not (Test-Path (Join-Path $BokehVendorDir $destName))) {
        $needCopy = $true
        break
    }
}

if (-not $needCopy) {
    Write-Host 'vendor/bokeh/ already present - skipping Bokeh asset copy.'
} else {
    Write-Host 'Locating Bokeh static assets in installed package...'

    $bokehStatic = & $PythonAbs -c "import bokeh, os; print(os.path.join(os.path.dirname(bokeh.__file__), 'server', 'static'))"
    if ($LASTEXITCODE -ne 0) { throw 'Failed to locate Bokeh package.' }
    $bokehStatic = $bokehStatic.Trim()

    if (-not (Test-Path $bokehStatic)) {
        throw "Could not locate Bokeh static directory at '$bokehStatic'"
    }

    Write-Host "  Bokeh static dir: $bokehStatic"
    New-Item -ItemType Directory -Force -Path $BokehVendorDir | Out-Null

    foreach ($entry in $BokehCopyMap.GetEnumerator()) {
        $srcName  = $entry.Key
        $destName = $entry.Value
        $dest     = Join-Path $BokehVendorDir $destName

        if (Test-Path $dest) {
            Write-Host "  $destName already exists - skipping."
            continue
        }
        $src = Join-Path $bokehStatic "js\$srcName"
        if (-not (Test-Path $src)) {
            throw "Expected '$src' not found in Bokeh package."
        }
        Copy-Item -Path $src -Destination $dest
        Write-Host "  Copied $srcName -> $destName"
    }
    Write-Host 'Bokeh assets written to vendor/bokeh/'
}

Write-Host ''
Write-Host '========================================='
Write-Host '  Setup complete!'
Write-Host '  Build with:  cargo build --release'
Write-Host '  Run with:    cargo run --release'
Write-Host ''
Write-Host '  For offline HTML (no CDN required):'
Write-Host '  cargo build --release --features bokeh-inline'
Write-Host '========================================='
