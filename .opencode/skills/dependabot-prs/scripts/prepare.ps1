<#
.SYNOPSIS
 Prepara la rama de un PR: trae main, resuelve conflictos de Cargo.toml/Cargo.lock y registra el estado.
.DESCRIPTION
 Deja el working tree listo para que la IA arregle posibles cambios de API. Nunca hace push.
#>
param([Parameter(Mandatory = $true)][int]$Pr)

. (Join-Path $PSScriptRoot "_common.ps1")
$root = Get-RepoRoot
Set-Location $root

$prData = Invoke-GhApi -Path "/pulls/$Pr"
$slug = Get-RepoSlug
if ($prData.head.repo.full_name -ne $slug) {
    throw "El PR #$Pr viene de un fork ($($prData.head.repo.full_name)); este script solo maneja ramas del repo"
}
$branch = $prData.head.ref

$dirty = @(& git status --porcelain)
if ($dirty.Count -gt 0) { throw "Working tree sucio. Confirma o descarta tus cambios antes de preparar el PR #$Pr" }

& git fetch origin main | Out-Null
if ($LASTEXITCODE -ne 0) { throw "git fetch origin main falló" }
& git fetch origin $branch | Out-Null
if ($LASTEXITCODE -ne 0) { throw "git fetch origin $branch falló" }
& git checkout -B $branch "origin/$branch" | Out-Null
if ($LASTEXITCODE -ne 0) { throw "No se pudo checkout $branch" }

$baseSha = [string]((& git merge-base HEAD origin/main) -join "`n")
$mainToml = [string]((& git show "origin/main`:src-tauri/Cargo.toml" 2>$null) -join "`n")
$baseToml = [string]((& git show "$baseSha`:src-tauri/Cargo.toml" 2>$null) -join "`n")
$prToml = [string]((& git show "HEAD`:src-tauri/Cargo.toml" 2>$null) -join "`n")

& git merge origin/main --no-edit | Out-Null
$mergeCode = $LASTEXITCODE
$conflicts = @(& git diff --name-only --diff-filter=U)
$resolved = @()
$unresolved = @()

if ($conflicts -contains "src-tauri/Cargo.lock") {
    & git checkout --theirs -- src-tauri/Cargo.lock
    if ($LASTEXITCODE -ne 0) { throw "No se pudo tomar Cargo.lock de main" }
    & git add src-tauri/Cargo.lock
    $resolved += "src-tauri/Cargo.lock"
}
if ($conflicts -contains "src-tauri/Cargo.toml") {
    $resolved += Resolve-CargoTomlConflict -BaseText $baseToml -PrText $prToml -MainText $mainToml -Path (Join-Path $root "src-tauri\Cargo.toml")
    & git add src-tauri/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw "No se pudo añadir Cargo.toml" }
}
$unresolved = @($conflicts | Where-Object { $_ -notin @("src-tauri/Cargo.lock", "src-tauri/Cargo.toml") })
if ($unresolved.Count -gt 0) {
    throw "Conflictos pendientes de revisión humana/IA: $($unresolved -join ', ')"
}

if ($mergeCode -ne 0) {
    & git commit --no-edit | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "No se pudo cerrar el merge de main" }
}

# Regenerar el lock con las versiones del PR (cargo check lo actualiza aunque falle la compilación).
$cargo = (Get-Command cargo -ErrorAction Stop).Source
$check = Invoke-Step -Exe $cargo -Arguments @("check", "--manifest-path", "src-tauri/Cargo.toml") -WorkDir $root -Name "cargo check"
$lockChanged = @(& git status --porcelain -- src-tauri/Cargo.lock).Count -gt 0
if ($lockChanged) {
    & git add src-tauri/Cargo.lock
    & git commit -m "chore(deps): regenerate Cargo.lock for PR #$Pr" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "No se pudo commitear Cargo.lock" }
}

$result = [pscustomobject]@{
    pr              = $Pr
    branch          = $branch
    headSha         = [string]((& git rev-parse HEAD) -join "`n")
    mergeCode       = $mergeCode
    conflictsFixed  = $resolved
    cargoCheck      = [pscustomobject]@{ ok = $check.ok; exitCode = $check.exitCode; tail = $check.tail }
    lockRegenerated = $lockChanged
    next            = if ($check.ok) { "verify.ps1" } else { "Corregir código (API cambiada) y luego verify.ps1" }
}
$result | ConvertTo-Json -Depth 6
