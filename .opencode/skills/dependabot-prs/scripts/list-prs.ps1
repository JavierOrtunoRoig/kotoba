<#
.SYNOPSIS
 Lista los PR abiertos, marcando cuáles son bumps de versión de Dependabot.
.PARAMETER All
 Incluir también PRs que no son bumps del bot.
#>
param([switch]$All)

. (Join-Path $PSScriptRoot "_common.ps1")
Set-Location (Get-RepoRoot)

$prs = Invoke-GhApi -Path "/pulls?state=open&per_page=100"
$rows = foreach ($p in $prs) {
    $isBot = $p.user.login -match '\[bot\]$'
    $isBump = $p.title -match '(?i)\bbump\b.+\bfrom\b.+\bto\b'
    if (-not $All -and -not ($isBot -and $isBump)) { continue }

    $checks = Invoke-GhApi -Path "/commits/$($p.head.sha)/check-runs?per_page=100"
    [pscustomobject]@{
        number         = $p.number
        title          = $p.title
        branch         = $p.head.ref
        headSha        = $p.head.sha
        author         = $p.user.login
        isBot          = [bool]$isBot
        isVersionBump  = [bool]$isBump
        mergeable      = $p.mergeable
        ci             = Get-CheckState -Checks $checks
        url            = $p.html_url
    }
}
$rows = @($rows | Where-Object { $_ -ne $null })
if ($rows.Count -eq 0) { "[]" } else { ConvertTo-Json -InputObject $rows -Depth 5 }
