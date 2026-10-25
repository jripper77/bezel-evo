param([Parameter(Mandatory=$true)][string]$OutputDirectory, [string]$MsiPath)
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$cache = Join-Path $repository 'target\scratch\game-fps-package'
$source = Join-Path $cache 'source-2.6.0'
$files = @('PresentMonService.exe','PresentMonAPI2.dll','ddETWExternal.xml')
$directory = Join-Path $source 'Intel\PresentMonSharedService'
$missing = @($files | Where-Object { -not (Test-Path -LiteralPath (Join-Path $directory $_) -PathType Leaf) })
if (-not (Test-Path -LiteralPath (Join-Path $source 'Intel\PresentMon\THIRD_PARTY.txt'))) { $missing += 'THIRD_PARTY.txt' }
if ($missing.Count -gt 0) {
    # A reusable build-source cache, not a per-run test/runtime directory.
    New-Item -ItemType Directory -Path $cache -Force | Out-Null
    if (-not $MsiPath) {
        $MsiPath = Join-Path $cache 'PresentMon-2.6.0.msi'
        if (-not (Test-Path -LiteralPath $MsiPath)) {
            Invoke-WebRequest 'https://github.com/GameTechDev/PresentMon/releases/download/v2.6.0/PresentMon-2.6.0.msi' -UseBasicParsing -OutFile $MsiPath
        }
    }
    $MsiPath = (Resolve-Path -LiteralPath $MsiPath).Path
    if ((Get-FileHash -LiteralPath $MsiPath).Hash -ine '0CB43D2622356C1E277A77F840887F9000CEA4E2D4236C0850922FC204C6F820') { throw 'PresentMon MSI checksum mismatch.' }
    $signature = Get-AuthenticodeSignature -LiteralPath $MsiPath
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Intel Corporation') { throw 'PresentMon MSI signature mismatch.' }
    New-Item -ItemType Directory -Path $source -Force | Out-Null
    $process = Start-Process -FilePath (Join-Path $env:WINDIR 'System32\msiexec.exe') -ArgumentList ('/a "{0}" /qn TARGETDIR="{1}" /l*v "{2}"' -f $MsiPath, $source, (Join-Path $cache 'extract.log')) -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -notin @(0,3010)) { throw "PresentMon source extraction failed: $($process.ExitCode)" }
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
foreach ($file in $files) { Copy-Item -LiteralPath (Join-Path $directory $file) -Destination (Join-Path $OutputDirectory $file) -Force }
Copy-Item -LiteralPath (Join-Path $repository 'packaging\windows\PresentMon-LICENSE.txt') -Destination (Join-Path $OutputDirectory 'LICENSE-PresentMon.txt') -Force
Copy-Item -LiteralPath (Join-Path $source 'Intel\PresentMon\THIRD_PARTY.txt') -Destination (Join-Path $OutputDirectory 'THIRD-PARTY-PresentMon.txt') -Force
& (Join-Path $PSScriptRoot 'setup-game-fps.ps1') -Mode Validate -SourceDirectory $OutputDirectory
if ($LASTEXITCODE -ne 0) { throw 'Game FPS payload validation failed.' }
