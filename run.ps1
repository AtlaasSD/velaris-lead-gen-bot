# ╔══════════════════════════════════════════════════════════════╗
# ║          VELARIS LEAD-GEN BOT  —  Launcher Script           ║
# ║          Instala dependencias si hace falta y ejecuta        ║
# ╚══════════════════════════════════════════════════════════════╝

$Host.UI.RawUI.WindowTitle = "Velaris Lead-Gen Bot"

# ── Colores ANSI compatibles con PowerShell 5.1 de Windows ─────────
$ESC = [char]27
$CYAN   = "$ESC[96m"
$GREEN  = "$ESC[92m"
$YELLOW = "$ESC[93m"
$RED    = "$ESC[91m"
$DIM    = "$ESC[2m"
$BOLD   = "$ESC[1m"
$RESET  = "$ESC[0m"

function Write-Banner {
    Clear-Host
    Write-Host ""
    Write-Host "$CYAN$BOLD  ██╗   ██╗███████╗██╗      █████╗ ██████╗ ██╗███████╗$RESET"
    Write-Host "$CYAN$BOLD  ██║   ██║██╔════╝██║     ██╔══██╗██╔══██╗██║██╔════╝$RESET"
    Write-Host "$CYAN$BOLD  ██║   ██║█████╗  ██║     ███████║██████╔╝██║███████╗$RESET"
    Write-Host "$CYAN$BOLD  ╚██╗ ██╔╝██╔══╝  ██║     ██╔══██║██╔══██╗██║╚════██║$RESET"
    Write-Host "$CYAN$BOLD   ╚████╔╝ ███████╗███████╗██║  ██║██║  ██║██║███████║$RESET"
    Write-Host "$CYAN$BOLD    ╚═══╝  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚══════╝$RESET"
    Write-Host ""
    Write-Host "$CYAN                   L E A D - G E N   B O T$RESET"
    Write-Host "$DIM  ──────────────────────────────────────────────────────────$RESET"
    Write-Host "$DIM  Velaris Agency  |  v1.0.0  |  Launcher de entorno$RESET"
    Write-Host "$DIM  ──────────────────────────────────────────────────────────$RESET"
    Write-Host ""
}

function Write-Step($num, $total, $text) {
    Write-Host ""
    Write-Host "$CYAN$BOLD  [$num/$total] ▶ $text$RESET"
    Write-Host "$DIM  $(('─' * 54))$RESET"
}

function Write-OK($text)   { Write-Host "$GREEN$BOLD  [OK]$RESET $text" }
function Write-Info($text) { Write-Host "$CYAN  [i]$RESET $text" }
function Write-Warn($text) { Write-Host "$YELLOW  [!]$RESET $text" }
function Write-Err($text)  { Write-Host "$RED$BOLD  [X]$RESET $RED$text$RESET" }

function Show-Spinner($msg, $seconds) {
    $frames = @("[|]", "[/]", "[-]", "[\]")
    $end = (Get-Date).AddSeconds($seconds)
    $i = 0
    while ((Get-Date) -lt $end) {
        Write-Host -NoNewline "`r$CYAN  $($frames[$i % 4])$RESET $msg   "
        Start-Sleep -Milliseconds 80
        $i++
    }
    Write-Host -NoNewline "`r$(' ' * 60)`r"
}

# Obtener directorio del script de forma segura
$scriptDir = $PSScriptRoot
if (-not $scriptDir) {
    # Fallback seguro para invocaciones extrañas
    $scriptDir = Split-Path -Parent $scriptPath
    if (-not $scriptDir) { $scriptDir = (Get-Location).Path }
}

# ══════════════════════════════════════════════════════════════
# PASO 1 — VERIFICAR RUST / CARGO
# ══════════════════════════════════════════════════════════════
Write-Banner
Write-Step 1 3 "VERIFICANDO ENTORNO RUST"

$cargoFound = $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)

if ($cargoFound) {
    $cargoVersion = cargo --version 2>&1
    Write-OK "Rust/Cargo detectado: $cargoVersion"
} else {
    Write-Warn "Rust no esta instalado. Iniciando instalacion automatica..."
    Write-Host ""
    Write-Info "Descargando rustup-init.exe..."

    $rustupUrl  = "https://win.rustup.rs/x86_64"
    $rustupExe  = "$env:TEMP\rustup-init.exe"

    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing
        Write-OK "Descarga completa. Instalando (esto puede tardar 2-5 minutos)..."
        Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
        # Recargar PATH
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" +
                    [System.Environment]::GetEnvironmentVariable("Path","User")
        Write-OK "Rust instalado correctamente."
    } catch {
        Write-Err "No se pudo instalar Rust automaticamente."
        Write-Err "Instalalo manualmente desde: https://rustup.rs"
        Read-Host "`n  Presiona Enter para salir"
        exit 1
    }
}

Write-Host ""

# ══════════════════════════════════════════════════════════════
# PASO 2 — COMPILAR (solo si no existe el binario o es viejo)
# ══════════════════════════════════════════════════════════════
Write-Step 2 3 "COMPILACION DEL PROYECTO"

$binaryPath = Join-Path $scriptDir "target\release\velaris_lead_gen.exe"
$needsBuild = $true

if (Test-Path $binaryPath) {
    $binTime = (Get-Item $binaryPath).LastWriteTime
    $srcDir = Join-Path $scriptDir "src"
    $cargoToml = Join-Path $scriptDir "Cargo.toml"
    
    $anyNewer = $false
    if (Test-Path $srcDir) {
        $srcFiles = Get-ChildItem -Path $srcDir -Recurse -Filter "*.rs"
        $anyNewer = ($srcFiles | Where-Object { $_.LastWriteTime -gt $binTime }).Count -gt 0
    }
    
    $cargoNewer = $false
    if (Test-Path $cargoToml) {
        $cargoNewer = (Get-Item $cargoToml).LastWriteTime -gt $binTime
    }

    if (-not $anyNewer -and -not $cargoNewer) {
        Write-OK "Binario actualizado encontrado. Omitiendo compilacion."
        $needsBuild = $false
    } else {
        Write-Info "Se detectaron cambios en el codigo fuente. Recompilando..."
    }
}

if ($needsBuild) {
    Write-Info "Compilando en modo release (optimizado). Esto tarda..."
    Write-Host ""

    Push-Location $scriptDir
    cargo build --release
    $exit_code = $LASTEXITCODE
    Pop-Location

    if ($exit_code -ne 0) {
        Write-Err "La compilacion fallo. Revisa los errores en la pantalla."
        Read-Host "`n  Presiona Enter para salir"
        exit 1
    }
    Write-OK "Compilacion exitosa."
}

Write-Host ""

# ══════════════════════════════════════════════════════════════
# PASO 3 — EJECUTAR
# ══════════════════════════════════════════════════════════════
Write-Step 3 3 "LANZANDO EL BOT"
Write-Info "Transfiriendo control al bot..."
Show-Spinner "Iniciando..." 1
Write-Host ""

Set-Location $scriptDir
& $binaryPath

Write-Host ""
Write-Host "$DIM  ──────────────────────────────────────────────────────────$RESET"
Write-Host "$GREEN$BOLD  *** Sesion finalizada.$RESET"
Write-Host "$DIM  ──────────────────────────────────────────────────────────$RESET"
Write-Host ""
Read-Host "  Presiona Enter para cerrar"
