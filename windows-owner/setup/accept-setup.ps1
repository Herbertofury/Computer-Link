$ErrorActionPreference = 'Stop'
$compiler = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'
if (-not (Test-Path -LiteralPath $compiler -PathType Leaf)) { throw 'Official runner Inno compiler unavailable; no fallback download' }
$compilerVersion = (Get-Item -LiteralPath $compiler).VersionInfo.FileVersion
Write-Output ('Installed ISCC file version: ' + $compilerVersion)
if ($compilerVersion -notmatch '^6\.7\.1(?:\.|$)') { throw 'ISCC version differs from reviewed 6.7.1; review required' }
$root = Join-Path $env:RUNNER_TEMP ('computer-link-setup-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$stage = Join-Path $root 'payload'
New-Item -ItemType Directory -Path $stage | Out-Null
foreach ($name in @('computer-link-owner-proof.exe','local-client.exe')) {
  Copy-Item -LiteralPath (Join-Path 'windows-owner/target/debug' $name) -Destination $stage
}
$recipe = Join-Path $PSScriptRoot 'computer-link.iss'
& $compiler "/DPayloadDir=$stage" "/O$root" $recipe
if ($LASTEXITCODE -ne 0) { throw 'Setup compilation failed' }
$setup = Join-Path $root 'Computer-Link-Local-Preview-Setup.exe'
$destination = Join-Path $root 'installed-preview'
function Run-Owned([string]$exe, [string[]]$arguments) {
  $info = [Diagnostics.ProcessStartInfo]::new($exe)
  $info.UseShellExecute = $false
  foreach ($argument in $arguments) { $info.ArgumentList.Add($argument) }
  $process = [Diagnostics.Process]::Start($info)
  if (-not $process.WaitForExit(120000)) { $process.Kill(); $null = $process.WaitForExit(5000); throw 'Owned setup process timed out; no completion claimed' }
  return $process.ExitCode
}
$arguments = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',"/DIR=$destination")
if ((Run-Owned $setup $arguments) -ne 0) { throw 'Disposable setup failed' }
foreach ($name in @('computer-link-owner-proof.exe','local-client.exe')) {
  if ((Get-FileHash -LiteralPath (Join-Path $stage $name)).Hash -ne (Get-FileHash -LiteralPath (Join-Path $destination $name)).Hash) { throw 'Installed bytes differ from tested binaries' }
}
$marker = Join-Path $destination 'unrelated-synthetic-user-data.txt'
[IO.File]::WriteAllText($marker,'preserve-this-fixture')
$refusalLog = Join-Path $root 'reinstall.log'
if ((Run-Owned $setup ($arguments + @('/LOG=' + $refusalLog))) -eq 0) { throw 'Existing-folder reinstall incorrectly accepted' }
if ((Get-Item -LiteralPath $refusalLog).Length -gt 1048576) { throw 'Refusal log exceeds test bound' }
if (-not ([IO.File]::ReadAllText($refusalLog).Contains('Choose a new destination folder. Existing folders are refused'))) { throw 'Reinstall failed without expected preservation refusal' }
if ([IO.File]::ReadAllText($marker) -ne 'preserve-this-fixture') { throw 'Reinstall changed unrelated fixture' }
foreach ($name in @('computer-link-owner-proof.exe','local-client.exe')) {
  if ((Get-FileHash -LiteralPath (Join-Path $stage $name)).Hash -ne (Get-FileHash -LiteralPath (Join-Path $destination $name)).Hash) { throw 'Rejected reinstall changed installed program bytes' }
}
$uninstaller = Join-Path $destination 'unins000.exe'
if ((Run-Owned $uninstaller @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART')) -ne 0) { throw 'Disposable uninstall failed' }
if ([IO.File]::ReadAllText($marker) -ne 'preserve-this-fixture') { throw 'Uninstall changed unrelated fixture' }
foreach ($name in @('computer-link-owner-proof.exe','local-client.exe')) {
  if (Test-Path -LiteralPath (Join-Path $destination $name)) { throw 'Program file still present after uninstall' }
}
Write-Output 'Disposable setup, exact installed bytes, existing-folder refusal and unrelated-data preservation passed. No artifact uploaded.'
