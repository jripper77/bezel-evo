$ErrorActionPreference = 'Stop'
$maintenance = Join-Path $PSScriptRoot 'installer-maintenance.ps1'
$fixture = Join-Path ([IO.Path]::GetTempPath()) ('bezel-maintenance-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path (Join-Path $fixture 'sensors') -Force | Out-Null
$wrapper = Join-Path $fixture 'check.ps1'
# Run the real script in a child: its exit must not terminate the test runner.
# Only process enumeration is replaced; lock checks use real Windows file handles.
@'
param($Maintenance, $Directory, $Processes, [int]$WaitSeconds, [int]$DisappearAfterMs)
$ErrorActionPreference = 'Stop'
$started = [Diagnostics.Stopwatch]::StartNew()
function Get-CimInstance {
    param($ClassName, $Filter)
    if ($DisappearAfterMs -lt 0 -or $started.ElapsedMilliseconds -lt $DisappearAfterMs) {
        Get-Content -LiteralPath $Processes -Raw | ConvertFrom-Json
    }
}
& $Maintenance -Mode Check -AppDirectory $Directory -HelperWaitSeconds $WaitSeconds
exit $LASTEXITCODE
'@ | Set-Content -LiteralPath $wrapper -Encoding UTF8
$shell = Join-Path $env:WINDIR 'System32\WindowsPowerShell\v1.0\powershell.exe'
$dll = Join-Path $fixture 'sensors\BlackSharp.Core.dll'
[IO.File]::WriteAllText($dll, 'Lock fixture; no hardware reader is started.')
$script:caseNumber = 0
function Start-Check($Processes = @(), [int]$WaitSeconds = 0, [int]$DisappearAfterMs = -1) {
    $script:caseNumber++
    $prefix = Join-Path $fixture "case-$script:caseNumber"
    ConvertTo-Json -InputObject @($Processes) | Set-Content -LiteralPath "$prefix.json" -Encoding UTF8
    $arguments = '-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{0}" -Maintenance "{1}" -Directory "{2}" -Processes "{3}.json" -WaitSeconds {4} -DisappearAfterMs {5}' -f $wrapper, $maintenance, $fixture, $prefix, $WaitSeconds, $DisappearAfterMs
    $process = Start-Process -FilePath $shell -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput "$prefix.out" -RedirectStandardError "$prefix.err"
    $null = $process.Handle
    return [pscustomobject]@{ Process = $process; Prefix = $prefix }
}
function Assert-Check($Check, [int]$ExitCode, [string]$Message = '') {
    if (-not $Check.Process.WaitForExit(15000)) { throw 'Maintenance check exceeded its test deadline.' }
    $Check.Process.Refresh()
    $output = [IO.File]::ReadAllText($Check.Prefix + '.out') + [IO.File]::ReadAllText($Check.Prefix + '.err')
    if ($Check.Process.ExitCode -ne $ExitCode -or ($Message -and -not $output.Contains($Message))) {
        throw "Unexpected maintenance result ($($Check.Process.ExitCode)): $output"
    }
}
try {
    Assert-Check (Start-Check) 0
    foreach ($name in @('bezel-studio.exe','bezel.exe')) {
        Assert-Check (Start-Check @([pscustomobject]@{ Name = $name; ExecutablePath = Join-Path $fixture $name })) 1 'Save and exit'
    }
    # A separate installation must not block this one.
    Assert-Check (Start-Check @([pscustomobject]@{ Name = 'bezel.exe'; ExecutablePath = Join-Path $fixture 'other\bezel.exe' })) 0
    $reader = [pscustomobject]@{ Name = 'bezel-sensors-helper.exe'; ExecutablePath = Join-Path $fixture 'sensors\bezel-sensors-helper.exe' }
    Assert-Check (Start-Check @($reader)) 1 'The sensor helper stops'
    Assert-Check (Start-Check @($reader) -WaitSeconds 3 -DisappearAfterMs 750) 0
    $invisibleReader = [pscustomobject]@{ Name = 'bezel-sensors-helper.exe'; ExecutablePath = $null }
    $lock = [IO.File]::Open($dll, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        Assert-Check (Start-Check @($invisibleReader)) 1 'The sensor helper stops'
        $pending = Start-Check @($invisibleReader) -WaitSeconds 5
        Start-Sleep -Milliseconds 1500
        if ($pending.Process.HasExited) { throw 'Check accepted a locked sensor DLL.' }
    } finally { $lock.Dispose() }
    Assert-Check $pending 0
    # Lock probing must preserve the payload bytes.
    if ([IO.File]::ReadAllText($dll) -ne 'Lock fixture; no hardware reader is started.') { throw 'Check modified a sensor file.' }

    # Sensor task ownership. Task Scheduler reports the principal as "user"
    # even when it was registered as DOMAIN\user: both are this installation's.
    # The task and the elevation are replaced, so no real task is touched; an
    # elevated runner would skip elevation and register a real task, so skip.
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $isAdmin = (New-Object Security.Principal.WindowsPrincipal($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    $ownership = 0
    if (-not $isAdmin) {
        $helper = Join-Path $fixture 'sensors\bezel-sensors-helper.exe'
        [IO.File]::WriteAllText($helper, 'Helper fixture; never started.')
        $sensorsWrapper = Join-Path $fixture 'sensors-task.ps1'
        @'
param($Maintenance, $Directory, $UserId, $Execute)
$ErrorActionPreference = 'Stop'
function Get-ScheduledTask {
    param($TaskName, $TaskPath, $ErrorAction)
    [pscustomobject]@{ Actions = @([pscustomobject]@{ Execute = $Execute }); Principal = [pscustomobject]@{ UserId = $UserId } }
}
function Start-Process { param($FilePath, $ArgumentList, $Verb, $WindowStyle, [switch]$Wait, [switch]$PassThru) [pscustomobject]@{ ExitCode = 0 } }
& $Maintenance -Mode Sensors -AppDirectory $Directory
exit $LASTEXITCODE
'@ | Set-Content -LiteralPath $sensorsWrapper -Encoding UTF8
        function Assert-Owner([string]$UserId, [string]$Execute, [int]$ExitCode) {
            $script:caseNumber++
            $prefix = Join-Path $fixture "case-$script:caseNumber"
            $arguments = '-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{0}" -Maintenance "{1}" -Directory "{2}" -UserId "{3}" -Execute "{4}"' -f $sensorsWrapper, $maintenance, $fixture, $UserId, $Execute
            $process = Start-Process -FilePath $shell -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput "$prefix.out" -RedirectStandardError "$prefix.err"
            $null = $process.Handle
            if (-not $process.WaitForExit(15000)) { throw 'Ownership check exceeded its test deadline.' }
            $output = [IO.File]::ReadAllText("$prefix.out") + [IO.File]::ReadAllText("$prefix.err")
            if ($process.ExitCode -ne $ExitCode) { throw "Unexpected ownership result for '$UserId' ($($process.ExitCode)): $output" }
            if ($ExitCode -eq 1 -and -not $output.Contains('Another Bezel installation')) { throw "Unexpected ownership message: $output" }
        }
        $short = $identity.Name.Split('\')[-1]
        Assert-Owner $short $helper 0
        Assert-Owner $identity.Name $helper 0
        Assert-Owner $identity.User.Value $helper 0
        Assert-Owner 'NoSuchBezelUser-7f3a' $helper 1
        Assert-Owner $short (Join-Path $fixture 'other\sensors\bezel-sensors-helper.exe') 1
        Remove-Item -LiteralPath $helper
        $ownership = 5
    }
    $summary = if ($isAdmin) { 'ownership skipped (elevated runner)' } else { "$ownership ownership (short name, DOMAIN\user, SID, other user, other path)" }
    Write-Output "PASS: 8 checks (idle, Studio, Light, other installation, helper timeout/exit, hidden helper lock timeout/release); payload preserved; $summary."
} finally {
    # Delete only this test's known temporary files, without recursive removal.
    Get-ChildItem -LiteralPath $fixture -File | Remove-Item
    $leftHelper = Join-Path $fixture 'sensors\bezel-sensors-helper.exe'
    if (Test-Path -LiteralPath $leftHelper) { Remove-Item -LiteralPath $leftHelper }
    Remove-Item -LiteralPath $dll
    Remove-Item -LiteralPath (Join-Path $fixture 'sensors')
    Remove-Item -LiteralPath $fixture
}
