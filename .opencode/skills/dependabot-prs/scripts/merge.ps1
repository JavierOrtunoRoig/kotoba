<#
.SYNOPSIS
 Sube la rama preparada, espera CI en verde y mergea el PR con squash.
.DESCRIPTION
 Precondición: verify.ps1 guardó un resultado passed=true para el HEAD local actual y el árbol está limpio.
 Con -Approve además envía un review APPROVE antes del merge.
#>
param(
    [Parameter(Mandatory = $true)][int]$Pr,
    [switch]$Approve,
    [int]$WaitSeconds = 900
)

. (Join-Path $PSScriptRoot "_common.ps1")
Set-Location (Get-RepoRoot)

$verifyPath = Get-VerifyPath -Pr $Pr
if (-not (Test-Path $verifyPath)) { throw "No hay verificación para #$Pr. Ejecuta verify.ps1 primero" }
$verify = Get-Content $verifyPath -Raw -Encoding UTF8 | ConvertFrom-Json
$head = [string]((& git rev-parse HEAD) -join "`n")
$dirty = @(& git status --porcelain)
if ($dirty.Count -gt 0) { throw "Working tree sucio; haz commit o descarta los cambios antes de mergear" }
if (-not $verify.passed) { throw "verify.ps1 no pasó para #$Pr" }
if ($verify.headSha -ne $head) { throw "El HEAD local cambió desde verify.ps1. Vuelve a ejecutarlo" }
$age = (Get-Date) - [datetime]$verify.checkedAt
if ($age.TotalHours -gt 2) { throw "La verificación tiene más de 2 horas. Vuelve a ejecutarla" }

$prData = Invoke-GhApi -Path "/pulls/$Pr"
$branch = $prData.head.ref
$current = [string]((& git rev-parse --abbrev-ref HEAD) -join "`n")
if ($current -ne $branch) { throw "Estás en '$current' pero el PR #$Pr usa '$branch'" }
if ($prData.mergeable -eq $false) { throw "GitHub dice que #$Pr tiene conflictos; prepara la rama de nuevo" }

& git push origin "HEAD:$branch" | Out-Null
if ($LASTEXITCODE -ne 0) { throw "git push falló. Puede que Dependabot haya rebaseado la rama; vuelve a prepare.ps1" }

$deadline = (Get-Date).AddSeconds($WaitSeconds)
$state = "none"
do {
    $prData = Invoke-GhApi -Path "/pulls/$Pr"
    $checks = Invoke-GhApi -Path "/commits/$($prData.head.sha)/check-runs?per_page=100"
    $state = Get-CheckState -Checks $checks
    if ($state -eq "success") { break }
    if ($state -eq "failure") { throw "CI en roja para #$Pr en $($prData.head.sha)" }
    if ((Get-Date) -ge $deadline) { throw "Timeout esperando CI de #$Pr (estado: $state)" }
    Start-Sleep -Seconds 20
} while ($true)

if ($Approve) {
    Invoke-GhApi -Method POST -Path "/pulls/$Pr/reviews" -Body @{ event = "APPROVE" } | Out-Null
}
$merge = Invoke-GhApi -Method PUT -Path "/pulls/$Pr/merge" -Body @{ merge_method = "squash" }

[pscustomobject]@{
    pr       = $Pr
    branch   = $branch
    headSha  = $prData.head.sha
    ci       = $state
    approved = [bool]$Approve
    merged   = [bool]$merge.merged
    message  = $merge.message
} | ConvertTo-Json -Depth 4
