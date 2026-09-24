# winget-pkgs へ提出するマニフェストを生成する。
#   .\winget\make-manifests.ps1                       # Cargo.toml の version と bin\PerformanceTray.exe の SHA256 から生成
#   .\winget\make-manifests.ps1 -Version 1.0.0 -Sha256 <hash>
# 生成先: winget\out\manifests\9\9enki\PerformanceTray\<version>\
# 提出: wingetcreate submit --token <PAT> winget\out\manifests\9\9enki\PerformanceTray\<version>
param(
    [string]$Version,
    [string]$Sha256,
    [string]$ReleaseDate = (Get-Date -Format 'yyyy-MM-dd')
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\scripts\common.ps1')

$ver = if ($Version) { $Version } else { Get-ProjectVersion }
if (-not $Sha256) {
    $exe = Join-Path $PSScriptRoot '..\bin\PerformanceTray.exe'
    if (-not (Test-Path $exe)) { throw 'bin\PerformanceTray.exe がありません。-Sha256 を指定するか先に build.ps1 を実行してください' }
    $Sha256 = (Get-FileHash $exe -Algorithm SHA256).Hash
}
$Sha256 = $Sha256.ToUpperInvariant()

$id = '9enki.PerformanceTray'
$repo = 'https://github.com/9enki/performance-tray'
$url = "$repo/releases/download/v$ver/PerformanceTray.exe"
$outDir = Join-Path $PSScriptRoot "out\manifests\9\9enki\PerformanceTray\$ver"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
InstallerType: portable
Commands:
- PerformanceTray
ReleaseDate: $ReleaseDate
Installers:
- Architecture: x64
  InstallerUrl: $url
  InstallerSha256: $Sha256
ManifestType: installer
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.installer.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
PackageLocale: en-US
Publisher: 9enki
PublisherUrl: https://github.com/9enki
PublisherSupportUrl: $repo/issues
PackageName: PerformanceTray
PackageUrl: $repo
License: MIT
LicenseUrl: $repo/blob/main/LICENSE
Copyright: Copyright (c) 2026 9enki
ShortDescription: Show CPU and memory usage as numbers in the Windows system tray.
Description: |-
  A tiny tray app for Windows 10 and 11. Two tray icons show the CPU usage and the memory usage as
  percentages, updated every second; the tooltip adds the memory in use and the total. A left click
  opens Task Manager. It reads GetSystemTimes and GlobalMemoryStatusEx only, writes nothing to the
  registry or disk, and needs no admin rights and no extra runtime.
Moniker: performancetray
Tags:
- cpu
- memory
- monitor
- performance
- ram
- system-tray
- tray
- windows
ReleaseNotesUrl: $repo/releases/tag/v$ver
ManifestType: defaultLocale
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.locale.en-US.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.locale.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
PackageLocale: ja-JP
Publisher: 9enki
PackageName: PerformanceTray
License: MIT
ShortDescription: CPU とメモリの使用率をタスクトレイに数字で表示する最小アプリ。
Description: |-
  トレイアイコン 2 つで CPU 使用率とメモリ使用率を 1 秒ごとに数字で表示します。ツールチップには
  メモリの使用量と搭載量も出ます。左クリックでタスク マネージャーが開きます。読み取り専用の API
  （GetSystemTimes / GlobalMemoryStatusEx）だけを使い、レジストリやファイルへの書き込みはなく、
  管理者権限も追加ランタイムも不要です。
ManifestType: locale
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.locale.ja-JP.yaml") -Encoding UTF8

"生成: $outDir"
Get-ChildItem $outDir | Select-Object -ExpandProperty Name
