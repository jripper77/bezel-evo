param(
    [string]$Repository = 'F:\Dev\bezel',
    [string]$Python = 'C:\Users\danie\miniconda3\python.exe',
    [switch]$SmokeOnly,
    [switch]$NormalClientProbe,
    [int]$GameProcessId = 0,
    [ValidateRange(10,120)][int]$GameSeconds = 45,
    [switch]$SkipGameBaseline,
    [ValidateRange(0,30)][int]$StartDelaySeconds = 0
)
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'This ETW capture probe needs elevation; it does not install a service.'
}
$root = (Resolve-Path -LiteralPath $Repository).Path
$scratch = Join-Path $root 'target\scratch\fps-evaluation'
$logs = Join-Path $root 'target\logs\fps-evaluation'
$runId = Get-Date -Format 'yyyyMMdd-HHmmss'
$runLogs = Join-Path $logs $runId
New-Item -ItemType Directory -Path $runLogs | Out-Null
$statusFile = Join-Path $runLogs 'status.json'
$registryPaths = @(
    'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Intel\PresentMon\Service',
    'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Intel\PresentMon'
)
$registryExisted = @{}
foreach ($path in $registryPaths) { $registryExisted[$path] = Test-Path -LiteralPath $path }
$result = [ordered]@{ state = 'running'; output = $runLogs; serviceInstalled = $false }
$result | ConvertTo-Json | Set-Content -LiteralPath $statusFile -Encoding UTF8
try {
    # Download hashes are pinned to the official v2.6.0 release metadata.
    foreach ($asset in @(
        @('PresentMon-2.6.0-x64.exe','B2A706BC6AD475749E3B7E3409263AA1E6906D45BDCF993F6DBC0F660188F1AF'),
        @('PresentMon-2.6.0.msi','0CB43D2622356C1E277A77F840887F9000CEA4E2D4236C0850922FC204C6F820')
    )) {
        $file = Join-Path (Join-Path $scratch 'downloads') $asset[0]
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $asset[1]) {
            throw 'Release download hash mismatch.'
        }
        if ((Get-AuthenticodeSignature -LiteralPath $file).Status -ne 'Valid') {
            throw 'Release download signature is invalid.'
        }
    }
    $extracted = Join-Path $scratch 'extracted'
    foreach ($name in @('PresentMonService.exe','PresentMonAPI2.dll')) {
        $file = Join-Path (Join-Path $extracted 'Intel\PresentMonSharedService') $name
        if ((Get-AuthenticodeSignature -LiteralPath $file).Status -ne 'Valid') {
            throw "Invalid extracted component signature: $name"
        }
    }
    $probe = Join-Path $root 'scripts\windows\fps-evaluation\evaluate.py'
    $probeArgs = @('-I', $probe, '--extracted', $extracted,
        '--console', (Join-Path $scratch 'downloads\PresentMon-2.6.0-x64.exe'),
        '--workload', (Join-Path $scratch 'fps-workload-v2.exe'))
    if ($GameProcessId -gt 0) {
        $game = Get-CimInstance Win32_Process -Filter "ProcessId = $GameProcessId"
        if (-not $game -or $game.ExecutablePath -notmatch 'Windrose') {
            throw 'The selected PID is not the expected running Windrose executable.'
        }
        $result.game = $game.Name
        $result.gamePid = $GameProcessId
        if ($StartDelaySeconds) { Start-Sleep -Seconds $StartDelaySeconds }
        $gameModes = if ($SkipGameBaseline) { 'console' } else { 'baseline,console' }
        & $Python @probeArgs --pid $GameProcessId --output (Join-Path $runLogs 'game') --seconds $GameSeconds --modes $gameModes *> (Join-Path $runLogs 'game.log')
        if ($LASTEXITCODE -ne 0) { throw 'Game console test failed; inspect game.log.' }
    } elseif (-not $NormalClientProbe) {
        # First smoke test; proceed to repeated runs only if both readers work.
        & $Python @probeArgs --output (Join-Path $runLogs 'smoke') --seconds 12 --modes console,api *> (Join-Path $runLogs 'smoke.log')
        if ($LASTEXITCODE -ne 0) { throw 'Smoke test failed; inspect smoke.log and capture.stderr.log.' }
        if (-not $SmokeOnly) {
            & $Python @probeArgs --output (Join-Path $runLogs 'comparison') --seconds 24 --repetitions 3 *> (Join-Path $runLogs 'comparison.log')
            if ($LASTEXITCODE -ne 0) { throw 'Comparison failed; inspect comparison.log.' }
        }
    }
    if ($NormalClientProbe -or $GameProcessId -gt 0) {
        $result.state = 'awaiting-normal-client'
        $result | ConvertTo-Json | Set-Content -LiteralPath $statusFile -Encoding UTF8
        & $Python @probeArgs --serve --output (Join-Path $runLogs 'api-server') *> (Join-Path $runLogs 'api-server.log')
        if ($LASTEXITCODE -ne 0) { throw 'External service probe failed; inspect api-server.log.' }
    }
    $result.state = 'complete'
} catch {
    $result.state = 'failed'
    $result.error = $_.Exception.Message
} finally {
    # The portable service opens its configuration key even when it only
    # reads defaults. Remove only newly-created, still-empty exact keys.
    # Keep pre-existing keys and any nonempty key for investigation.
    foreach ($path in $registryPaths) {
        if (-not $registryExisted[$path] -and (Test-Path -LiteralPath $path)) {
            $key = Get-Item -LiteralPath $path
            if ($key.GetValueNames().Count -eq 0 -and $key.GetSubKeyNames().Count -eq 0) {
                Remove-Item -LiteralPath $path
            }
        }
    }
    $result | ConvertTo-Json | Set-Content -LiteralPath $statusFile -Encoding UTF8
}
