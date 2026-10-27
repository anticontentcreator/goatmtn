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

$rev = git rev-parse --short HEAD
$tag = $null

if ($Target -eq "prod") {
  if (git status --porcelain) { Write-Error "Uncommitted changes. Commit first."; exit 1 }
  git fetch origin --quiet
  if ($LASTEXITCODE -ne 0) { Write-Error "git fetch failed. Check your GitHub SSH access."; exit 1 }
  $branch = git rev-parse --abbrev-ref HEAD
  $tag    = git describe --exact-match --tags HEAD 2>$null
  if ($branch -ne "main" -and -not $tag) {
    Write-Error "Prod deploys must come from main or an already-tagged commit."; exit 1
  }
  if ($branch -eq "main" -and (git rev-parse HEAD) -ne (git rev-parse origin/main)) {
    Write-Error "main is not in sync with origin/main. Push or pull first."; exit 1
  }
  if ((Read-Host "Deploy $rev ($branch $tag) to PRODUCTION? Type 'prod' to confirm") -ne "prod") { exit }
}

$stamp = Get-Date -Format yyyyMMdd-HHmmss

ssh $s "mkdir -p ${base}/staging"
if ($LASTEXITCODE -ne 0) { Write-Error "Cannot reach $s over SSH."; exit 1 }

scp @($files.FullName) "${s}:${base}/staging/"
if ($LASTEXITCODE -ne 0) { Write-Error "Upload failed."; exit 1 }

ssh $s "mkdir -p ${base}/backups/$stamp && cd ${base}/staging && for f in *.cs; do [ -f ../plugins/`$f ] && cp ../plugins/`$f ../backups/$stamp/; done; mv *.cs ../plugins/ && journalctl -u rust -n 25 --no-pager"
if ($LASTEXITCODE -ne 0) { Write-Error "Remote step failed. Not tagging."; exit 1 }

if ($Target -eq "prod") {
  ssh $s "echo '$rev $stamp' >> ${base}/backups/DEPLOYS.log"
  if (-not $tag) {
    git tag -a "prod-$stamp" -m "Prod deploy $stamp"
    git push origin "prod-$stamp"
  }
}