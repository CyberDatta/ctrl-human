# Builds standalone PyInstaller binaries for all Python scripts that ctrl-human
# spawns at runtime. Run this once per machine before `pnpm tauri build`.
#
# Output: src-tauri/binaries/{bridge,camera_server,extract_pose}-<target-triple>.exe
#
# Requirements: Rust toolchain on PATH, pygmy venv in place.

$ErrorActionPreference = "Stop"

$ScriptDir   = $PSScriptRoot
$BinariesDir = Join-Path $ScriptDir "..\src-tauri\binaries"
$BuildTmp    = Join-Path $ScriptDir ".pyinstaller_build"
$Python      = Join-Path $ScriptDir "pygmy\Scripts\python.exe"
$Pip         = Join-Path $ScriptDir "pygmy\Scripts\pip.exe"

$rustInfo = rustc -vV
$Target = ($rustInfo | Select-String "^host: ").ToString().Replace("host: ", "").Trim()
Write-Host "Building sidecars for target: $Target"

New-Item -ItemType Directory -Force -Path $BinariesDir | Out-Null
New-Item -ItemType Directory -Force -Path $BuildTmp    | Out-Null

& $Pip install pyinstaller --quiet

function Build-Binary {
    param(
        [string]$Script,
        [string]$Name,
        [string[]]$ExtraArgs = @()
    )

    Write-Host "-> Building $Name..."

    $pyArgs = @(
        "-m", "PyInstaller",
        "--onefile",
        "--name", $Name,
        "--distpath", $BuildTmp,
        "--workpath", (Join-Path $BuildTmp "work"),
        "--specpath", (Join-Path $BuildTmp "specs"),
        "--noconfirm"
    ) + $ExtraArgs + @((Join-Path $ScriptDir $Script))

    & $Python @pyArgs
    if ($LASTEXITCODE -ne 0) { throw "PyInstaller failed for $Name" }

    $src  = Join-Path $BuildTmp "$Name.exe"
    $dest = Join-Path $BinariesDir "$Name-$Target.exe"
    Move-Item -Force $src $dest
    Write-Host "   $dest"
}

# bridge.py uses only stdlib - fast build, tiny binary
Build-Binary -Script "bridge.py" -Name "bridge"

# camera_server.py uses cv2 + mediapipe
Build-Binary -Script "camera_server.py" -Name "camera_server" -ExtraArgs @(
    "--collect-all", "mediapipe",
    "--collect-all", "cv2"
)

# extract_pose.py uses mediapipe
Build-Binary -Script "extract_pose.py" -Name "extract_pose" -ExtraArgs @(
    "--collect-all", "mediapipe"
)

Write-Host ""
Write-Host "Done. Sidecars written to: $BinariesDir"
Write-Host "You can now run: pnpm tauri build"
