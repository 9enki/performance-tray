# ユニットテスト（引数の解釈、使用率の計算、アイコンの文字と色の選択、表示文字列）を実行する。
#   WSL から : powershell.exe -NoProfile -ExecutionPolicy Bypass -File test.ps1
#   Windows  : .\test.ps1
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. .\scripts\common.ps1

$cargo = Get-Cargo
$env:CARGO_TARGET_DIR = Get-CargoTargetDir
$ErrorActionPreference = 'Continue'   # cargo の stderr 出力をエラー扱いにしない
& $cargo test 2>&1 | ForEach-Object { "$_" }
exit $LASTEXITCODE
