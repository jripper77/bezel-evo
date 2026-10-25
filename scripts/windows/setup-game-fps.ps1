param(
    [ValidateSet('Install','Validate','Uninstall')][string]$Mode = 'Install',
    [string]$SourceDirectory = (Join-Path $PSScriptRoot 'fps'),
    [switch]$Elevated
)
# Optional, machine-wide component. No user groups, application settings,
# themes or other installations are changed by this script.
$ErrorActionPreference = 'Stop'
$serviceName = 'PresentMonSharedService'
$component = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'Bezel Evo FPS'
$executable = Join-Path $component 'PresentMonService.exe'
$uninstallKey = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BezelEvoFPS'
$payload = [ordered]@{
    'PresentMonService.exe' = 'C3489174D543A59D345CAFADA6116AE1CB680F8D5BF04E70493835DA444ECAA0'
    'PresentMonAPI2.dll' = '3164E9FB99E65B120D3C7B2A973239670F3134CF55034BDC89A56278362723DA'
    'ddETWExternal.xml' = '7AA7F8194A0FE5B2A713A610F7C3A22C74E82BFFDB7B13582BC97A8ED23389B7'
}
function Service-Executable([string]$command) {
    if ($command -match '^"([^"]+)"') { return [IO.Path]::GetFullPath($Matches[1]) }
    if ($command -match '^(.*?\.exe)(\s|$)') { return [IO.Path]::GetFullPath($Matches[1]) }
    throw 'Unrecognized PresentMon service executable.'
}
function Validate-Payload {
    foreach ($entry in $payload.GetEnumerator()) {
        $path = Join-Path $SourceDirectory $entry.Key
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ine $entry.Value) {
            throw "FPS component file failed verification: $($entry.Key)"
        }
        if ([IO.Path]::GetExtension($path) -in @('.exe','.dll')) {
            $signature = Get-AuthenticodeSignature -LiteralPath $path
            if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Intel Corporation') {
                throw "FPS component signature failed verification: $($entry.Key)"
            }
        }
    }
}
function Owned-Service {
    $service = Get-CimInstance Win32_Service -Filter "Name='$serviceName'" -ErrorAction SilentlyContinue
    if ($service -and (Service-Executable $service.PathName) -ieq $executable) { return $service }
    return $null
}
try {
    if ($Mode -ne 'Uninstall') {
        $SourceDirectory = [IO.Path]::GetFullPath($SourceDirectory)
        Validate-Payload
    }
    if ($Mode -eq 'Validate') {
        Write-Output 'PresentMon 2.6.0 service payload verified (no system changes).'
        exit 0
    }
    $admin = ([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $admin) {
        if ($Elevated) { throw 'FPS component setup requires administrator approval.' }
        $shell = Join-Path $env:WINDIR 'System32\WindowsPowerShell\v1.0\powershell.exe'
        $arguments = '-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{0}" -Mode {1} -SourceDirectory "{2}" -Elevated' -f $PSCommandPath, $Mode, $SourceDirectory
        $process = Start-Process -FilePath $shell -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        if ($process.ExitCode -eq 0) { Write-Output 'FPS component setup completed. Studio and Light can run as normal users.' }
        else { Write-Output 'FPS component setup failed. Close its clients and retry Setup game FPS.' }
        exit $process.ExitCode
    }
    if ($Mode -eq 'Uninstall') {
        # A later Intel/third-party install may have taken over the shared name.
        # Never stop or delete a service whose executable is outside our component.
        if (Owned-Service) {
            Stop-Service -Name $serviceName
            (Get-Service -Name $serviceName).WaitForStatus('Stopped', [TimeSpan]::FromSeconds(20))
            & (Join-Path $env:WINDIR 'System32\sc.exe') delete $serviceName | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'Could not remove the owned FPS service.' }
        }
        # Explicit files only, after verifying their hashes. Preserve modified or
        # unknown files and the directory if it contains anything else.
        foreach ($entry in $payload.GetEnumerator()) {
            $path = Join-Path $component $entry.Key
            if ((Test-Path -LiteralPath $path -PathType Leaf) -and (Get-FileHash -LiteralPath $path).Hash -ieq $entry.Value) {
                Remove-Item -LiteralPath $path
            }
        }
        # Retain the retry entry if a loaded DLL prevented file removal.
        $registration = Get-ItemProperty -LiteralPath $uninstallKey -ErrorAction SilentlyContinue
        if ($registration -and $registration.InstallLocation -ieq $component) {
            Remove-Item -LiteralPath $uninstallKey
        }
        Write-Output 'Owned FPS service removed. Modified files, licences and the maintenance script are retained.'
        exit 0
    }
    $existing = Get-CimInstance Win32_Service -Filter "Name='$serviceName'" -ErrorAction SilentlyContinue
    if ($existing -and -not (Owned-Service)) {
        # Reuse a valid Intel 2.6 installation; never downgrade or overwrite it.
        $installed = Service-Executable $existing.PathName
        $api = Join-Path ([IO.Path]::GetDirectoryName($installed)) 'PresentMonAPI2.dll'
        foreach ($path in @($installed,$api)) {
            $signature = Get-AuthenticodeSignature -LiteralPath $path
            if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Intel Corporation') { throw 'Existing PresentMon service is not a verified Intel installation.' }
        }
        if ((Get-Item -LiteralPath $api).VersionInfo.ProductVersion -notmatch '^2\.6\.') {
            throw 'The existing PresentMon service is a different version. It has been preserved; install a compatible PresentMon 2.6 service or update Bezel.'
        }
        Start-Service -Name $serviceName
        Write-Output 'Using the existing Intel PresentMon service; no files or ownership changed.'
        exit 0
    }
    New-Item -ItemType Directory -Path $component -Force | Out-Null
    $changed = @($payload.GetEnumerator() | Where-Object {
        $path = Join-Path $component $_.Key
        -not (Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path).Hash -ine $_.Value
    }).Count -gt 0
    # Preflight DLL locks before stopping any service. Never terminate Bezel.
    if ($existing -and $changed) {
        $api = Join-Path $component 'PresentMonAPI2.dll'
        $probe = $null
        try { $probe = [IO.File]::Open($api, 'Open', 'ReadWrite', 'None') }
        catch { throw 'Save and close Studio and Light before updating the FPS component.' }
        finally { if ($probe) { $probe.Dispose() } }
        Stop-Service -Name $serviceName
        (Get-Service -Name $serviceName).WaitForStatus('Stopped', [TimeSpan]::FromSeconds(20))
    }
    if ($changed) {
        foreach ($entry in $payload.GetEnumerator()) {
            Copy-Item -LiteralPath (Join-Path $SourceDirectory $entry.Key) -Destination (Join-Path $component $entry.Key) -Force
            if ((Get-FileHash -LiteralPath (Join-Path $component $entry.Key)).Hash -ine $entry.Value) { throw 'Installed FPS component hash mismatch.' }
        }
    }
    if ([IO.Path]::GetFullPath($PSCommandPath) -ine (Join-Path $component 'setup-game-fps.ps1')) {
        Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $component 'setup-game-fps.ps1') -Force
    }
    foreach ($name in @('LICENSE-PresentMon.txt','THIRD-PARTY-PresentMon.txt')) {
        $licence = Join-Path $SourceDirectory $name
        if (Test-Path -LiteralPath $licence) { Copy-Item -LiteralPath $licence -Destination (Join-Path $component $name) -Force }
    }
    if (-not $existing) {
        # SCM does not pass ImagePath arguments to Intel's ServiceMain callback.
        # Use the official service defaults, including its PMService ETW name.
        New-Service -Name $serviceName -BinaryPathName ('"{0}"' -f $executable) -DisplayName 'Intel PresentMon Shared Service (Bezel Evo FPS)' -StartupType Automatic -Description 'Optional game FPS collector. Bezel Studio and Light run without administrator privileges.' | Out-Null
    }
    Start-Service -Name $serviceName
    (Get-Service -Name $serviceName).WaitForStatus('Running', [TimeSpan]::FromSeconds(20))
    New-Item -Path $uninstallKey -Force | Out-Null
    $shell = Join-Path $env:WINDIR 'System32\WindowsPowerShell\v1.0\powershell.exe'
    $values = @{
        DisplayName = 'Bezel Evo - Game FPS support (Intel PresentMon)'; DisplayVersion = '2.6.0'; Publisher = 'Bezel Evo / Intel Corporation'
        InstallLocation = $component
        UninstallString = '"{0}" -NoProfile -ExecutionPolicy Bypass -File "{1}" -Mode Uninstall' -f $shell, (Join-Path $component 'setup-game-fps.ps1')
    }
    foreach ($name in $values.Keys) { New-ItemProperty -LiteralPath $uninstallKey -Name $name -Value $values[$name] -PropertyType String -Force | Out-Null }
    foreach ($name in @('NoModify','NoRepair')) { New-ItemProperty -LiteralPath $uninstallKey -Name $name -Value 1 -PropertyType DWord -Force | Out-Null }
    New-ItemProperty -LiteralPath $uninstallKey -Name EstimatedSize -Value 6000 -PropertyType DWord -Force | Out-Null
    Write-Output 'Game FPS support installed. Studio and Light can run normally; account groups were not changed.'
    exit 0
} catch {
    Write-Output $_.Exception.Message
    exit 1
}
