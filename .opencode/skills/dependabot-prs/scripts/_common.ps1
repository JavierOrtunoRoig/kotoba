# Funciones compartidas por los scripts de la skill dependabot-prs.
# Uso: . (Join-Path $PSScriptRoot "_common.ps1")

function Get-RepoRoot {
    $root = (& git rev-parse --show-toplevel 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $root) {
        throw "No estás dentro de un repositorio git"
    }
    return ([string]($root -join "`n")).Trim()
}

function Get-RepoSlug {
    $url = [string](& git remote get-url origin 2>$null)
    if ($LASTEXITCODE -ne 0) { throw "No hay remote 'origin'" }
    if ($url -match 'github\.com[:/]([^/]+)/(.+?)(?:\.git)?$') {
        return "$($Matches[1])/$($Matches[2])"
    }
    throw "No se pudo deducir el slug de GitHub de: $url"
}

function Get-GitHubToken {
    $env:GCM_INTERACTIVE = "never"
    $payload = "protocol=https`nhost=github.com`n`n"
    $out = $payload | git credential fill 2>&1
    $line = ($out -split "`r?`n" | Where-Object { $_ -like "password=*" } | Select-Object -First 1)
    if (-not $line) { throw "No se obtuvo token de GitHub (git credential fill)" }
    return ($line -replace "^password=", "")
}

function Invoke-GhApi {
    param(
        [ValidateSet("GET", "POST", "PUT")]
        [string]$Method = "GET",
        [Parameter(Mandatory = $true)][string]$Path,
        $Body
    )
    $slug = Get-RepoSlug
    $token = Get-GitHubToken
    $headers = @{
        Authorization          = "Bearer $token"
        Accept                 = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
        "User-Agent"           = "kotoba-dependabot-skill"
    }
    $params = @{
        Method  = $Method
        Uri     = "https://api.github.com/repos/$slug$Path"
        Headers = $headers
    }
    if ($null -ne $Body) {
        $params.Body = $Body | ConvertTo-Json -Depth 10 -Compress
        $params.ContentType = "application/json"
    }
    return Invoke-RestMethod @params
}

function Get-CheckState {
    param($Checks)
    # Solo la ejecución más reciente de cada check (descarta re-runs antiguos).
    $runs = @(@($Checks.check_runs) | Group-Object name | ForEach-Object {
        $_.Group | Sort-Object created_at -Descending | Select-Object -First 1
    })
    if ($runs.Count -eq 0) { return "none" }
    $bad = @($runs | Where-Object {
        $c = $_.conclusion
        $c -and $c -notin @("success", "neutral", "skipped")
    })
    if ($bad.Count -gt 0) { return "failure" }
    $pending = @($runs | Where-Object {
        -not $_.conclusion -or $_.conclusion -in @("queued", "in_progress")
    })
    if ($pending.Count -gt 0) { return "pending" }
    return "success"
}

function Add-NodeToPath {
    $existing = Get-Command node -ErrorAction SilentlyContinue
    if ($existing) { return }
    $candidate = Join-Path $env:USERPROFILE "scoop\apps\nodejs-lts\current"
    if (Test-Path (Join-Path $candidate "node.exe")) {
        $env:Path = "$candidate;$env:Path"
    }
}

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Exe,
        [string[]]$Arguments = @(),
        [Parameter(Mandatory = $true)][string]$WorkDir,
        [Parameter(Mandatory = $true)][string]$Name
    )
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & $Exe @Arguments 2>&1 | Out-String
        $code = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previous
    }
    # PowerShell 5.1 envuelve el stderr nativo en bloques NativeCommandError: se recortan.
    $tail = @($output -split "`r?`n" | Where-Object {
        $_ -and $_ -notmatch 'NativeCommandError|CategoryInfo|FullyQualifiedErrorId' -and
        $_ -notmatch '^\s*[+~]' -and $_ -notmatch '^\s*(En|At)\s+.*(Car|char)'
    } | ForEach-Object { $_ -replace '^[\w.\-]+(\.exe|\.cmd)?\s*:\s*', '' } | Select-Object -Last 20)
    return [pscustomobject]@{
        name     = $Name
        exitCode = $code
        ok       = ($code -eq 0)
        tail     = $tail
    }
}

function Get-CargoVersionMap {
    param([Parameter(Mandatory = $true)][string]$Text)
    $map = @{}
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line -match '^\s*([A-Za-z0-9_-]+)\s*=\s*"([^"]+)"') {
            $map[$Matches[1]] = $Matches[2]
        }
        elseif ($line -match '^\s*([A-Za-z0-9_-]+)\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"') {
            $map[$Matches[1]] = $Matches[2]
        }
    }
    return $map
}

function Resolve-CargoTomlConflict {
    param(
        [Parameter(Mandatory = $true)][string]$BaseText,
        [Parameter(Mandatory = $true)][string]$PrText,
        [Parameter(Mandatory = $true)][string]$MainText,
        [Parameter(Mandatory = $true)][string]$Path
    )
    $base = Get-CargoVersionMap -Text $BaseText
    $pr = Get-CargoVersionMap -Text $PrText
    $main = Get-CargoVersionMap -Text $MainText

    # El PR manda en las claves que cambió; main manda en el resto.
    $changed = @{}
    foreach ($key in $pr.Keys) {
        if (-not $base.ContainsKey($key) -or $base[$key] -ne $pr[$key]) {
            $changed[$key] = $pr[$key]
        }
    }
    $unknown = @($changed.Keys | Where-Object { -not $main.ContainsKey($_) })
    if ($unknown.Count -gt 0) {
        throw "No se pueden resolver automáticamente estas claves: $($unknown -join ', ')"
    }

    $eol = if ($MainText.Contains("`r`n")) { "`r`n" } else { "`n" }
    $lines = $MainText -split "`r?`n"
    for ($i = 0; $i -lt $lines.Count; $i++) {
        foreach ($key in $changed.Keys) {
            if ($lines[$i] -match ('^\s*' + [regex]::Escape($key) + '\s*=')) {
                # Primera cadena entre comillas de la línea = versión (formato "x" o { version = "x" }).
                $lines[$i] = $lines[$i] -replace '"[^"]+"', ('"{0}"' -f $changed[$key])
            }
        }
    }
    $text = $lines -join $eol
    [System.IO.File]::WriteAllText($Path, $text, [System.Text.UTF8Encoding]::new($false))
    return @($changed.Keys | Sort-Object)
}

function Get-VerifyPath {
    param([Parameter(Mandatory = $true)][int]$Pr)
    $dir = Join-Path $env:TEMP "kotoba-dependabot"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
    return (Join-Path $dir "$Pr.verify.json")
}
