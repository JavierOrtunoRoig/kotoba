param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

$changelog = Get-Content -Raw "CHANGELOG.md"
$pattern = "(?ms)^##\s+$([regex]::Escape($Version))\b.*?(?=^##\s|\z)"
$m = [regex]::Match($changelog, $pattern)
$section = if ($m.Success) { $m.Value.Trim() } else { "See CHANGELOG.md for details." }

$body = @"
Instaladores de kotoba v$Version para Windows 10 y 11 (x64).

- **NSIS (recomendado)**: ``kotoba_${Version}_x64-setup.exe``
- **MSI**: ``kotoba_${Version}_x64_en-US.msi``

El NSIS instala por usuario, sin permisos de administrador, y es el formato que usa el
updater automático. El MSI es el instalador clásico de Windows.

El código fuente de esta versión está en la etiqueta ``v$Version``.

WebView2 viene incluido en Windows 11 y en la mayoría de equipos con Windows 10; si
falta, el instalador puede descargarlo.

Los binarios **no están firmados digitalmente**, así que SmartScreen puede pedir
confirmación la primera vez.

---

$section
"@

Set-Content -Path "release-notes.md" -Value $body -Encoding utf8
gh release edit "v$Version" --notes-file "release-notes.md"
