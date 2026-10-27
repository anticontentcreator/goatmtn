# deploy.ps1
param(
  [ValidateSet("test","prod")][string]$Target = "test",
  [string]$Plugin
)

$hosts = @{ test = "rtest"; prod = "rprod" }
$s     = $hosts[$Target]
$src   = Join-Path $PSScriptRoot "plugin\scripts"
$base  = "/home/rust/server/carbon"

$files = Get-ChildItem $src -Filter "*.cs" -Recurse -ErrorAction SilentlyContinue |
  Where-Object { $_.FullName -notmatch '\\(bin|obj)\\' }
if ($Plugin) { $files = $files | Where-Object { $_.BaseName -eq $Plugin } }
if (-not $files) { Write-Error "No .cs files found in $src"; exit 1 }

if ($Target -eq "prod") {
  if (git status --porcelain) { Write-Error "Uncommitted changes. Commit first."; exit 1 }
  $rev = git rev-parse --short HEAD
  if ((Read-Host "Deploy $rev to PRODUCTION? Type 'prod' to confirm") -ne "prod") { exit }
}

$stamp = Get-Date -Format yyyyMMdd-HHmmss
ssh $s "mkdir -p ${base}/staging"
scp @($files.FullName) "${s}:${base}/staging/"
ssh $s "mkdir -p ${base}/backups/$stamp && cd ${base}/staging && for f in *.cs; do [ -f ../plugins/`$f ] && cp ../plugins/`$f ../backups/$stamp/; done; mv *.cs ../plugins/ && journalctl -u rust -n 25 --no-pager"