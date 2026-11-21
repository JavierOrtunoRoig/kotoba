<#
.SYNOPSIS
 Ejecuta cargo check, cargo test --lib y npm run typecheck; guarda un JSON con el resultado.
.DESCRIPTION
 El JSON se escribe en $env:TEMP\kotoba-dependabot\<PR>.verify.json y también se imprime por stdout.
#>
param([Parameter(Mandatory = $true)][int]$Pr)

. (Join-Path $PSScriptRoot "_common.ps1")
$root = Get-RepoRoot
Set-Location $root

$prData = Invoke-GhApi -Path "/pulls/$Pr"
$branch = $prData.head.ref
$current = [string]((& git rev-parse --abbrev-ref HEAD) -join "`n")
if ($current -ne $branch) { throw "Estás en '$current' pero el PR #$Pr usa '$branch'. Ejecuta prepare.ps1 primero" }
$unmerged = @(& git diff --name-only --diff-filter=U)
if ($unmerged.Count -gt 0) { throw "Conflictos sin resolver: $($unmerged -join ', ')" }

Add-NodeToPath
$cargo = (Get-Command cargo -ErrorAction Stop).Source
$npm = (Get-Command npm -ErrorAction Stop).Source

$steps = @()
$steps += Invoke-Step -Exe $cargo -Arguments @("check", "--manifest-path", "src-tauri/Cargo.toml") -WorkDir $root -Name "cargo check"
$steps += Invoke-Step -Exe $cargo -Arguments @("test", "--manifest-path", "src-tauri/Cargo.toml", "--lib") -WorkDir $root -Name "cargo test --lib"
$steps += Invoke-Step -Exe $npm -Arguments @("run", "typecheck") -WorkDir $root -Name "npm run typecheck"

$result = [pscustomobject]@{
    pr        = $Pr
    branch    = $branch
    headSha   = [string]((& git rev-parse HEAD) -join "`n")
    passed    = @($steps | Where-Object { -not $_.ok }).Count -eq 0
    checkedAt = (Get-Date).ToUniversalTime().ToString("o")
    steps     = $steps
}
$path = Get-VerifyPath -Pr $Pr
[System.IO.File]::WriteAllText($path, ($result | ConvertTo-Json -Depth 6), [System.Text.UTF8Encoding]::new($false))
$result | ConvertTo-Json -Depth 6
