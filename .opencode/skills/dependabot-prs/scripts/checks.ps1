<#
.SYNOPSIS
 Estado de los checks de CI del head de un PR. Con -Wait, espera a que terminen.
#>
param(
    [Parameter(Mandatory = $true)][int]$Pr,
    [switch]$Wait,
    [int]$TimeoutSeconds = 900
)

. (Join-Path $PSScriptRoot "_common.ps1")
Set-Location (Get-RepoRoot)

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
do {
    $prData = Invoke-GhApi -Path "/pulls/$Pr"
    $checks = Invoke-GhApi -Path "/commits/$($prData.head.sha)/check-runs?per_page=100"
    $state = Get-CheckState -Checks $checks
    if (-not $Wait -or $state -notin @("pending", "none") -or (Get-Date) -ge $deadline) { break }
    Start-Sleep -Seconds 20
} while ($true)

$result = [pscustomobject]@{
    number  = $Pr
    headSha = $prData.head.sha
    state   = $state
    checks  = @($checks.check_runs | ForEach-Object {
        [pscustomobject]@{ name = $_.name; status = $_.status; conclusion = $_.conclusion }
    })
}
$result | ConvertTo-Json -Depth 5
