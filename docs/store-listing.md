# Microsoft Store listing

Text and settings to paste into Partner Center when submitting PerformanceTray. Keep this file in
step with the README and the winget manifests. The Store product page is what most people read, so
the wording here is a little friendlier than the README.

## Identity (Partner Center > Product management > Product identity)

Reserve the name **PerformanceTray** in Partner Center, then copy the three values shown there into
the repository variables `STORE_IDENTITY_NAME`, `STORE_PUBLISHER` and `STORE_PUBLISHER_DISPLAY_NAME`,
or pass them to `build-msix.ps1` by hand:

```powershell
.\build-msix.ps1 -IdentityName <Package/Identity/Name> -Publisher '<Package/Identity/Publisher>' -PublisherDisplayName 9enki
```

The publisher values are the same for every app of the account, so they match PowerModeTray.
Only the identity name is new.

## Properties

| Field | Value |
| --- | --- |
| Category | Utilities & tools |
| Subcategory | (none) |
| Pricing | Free |
| Markets | All |
| Age rating | Fill in the IARC questionnaire. The app has no user content, no purchases, no online features |
| Privacy policy URL | https://github.com/9enki/performance-tray/blob/main/PRIVACY.md |
| Website | https://github.com/9enki/performance-tray |
| Support contact | https://github.com/9enki/performance-tray/issues |
| System requirements | Windows 10 version 1809 or later, x64 |

## Store listing: English (United States)

**Short title** (up to 50 characters)

```
CPU and memory usage in the tray
```

**Description**

```
PerformanceTray shows the CPU usage and the memory usage of your PC as numbers in the system tray, refreshed every second, and tells you which programs are behind them.

Two icons sit in the tray: a chip outline for the CPU and a memory module for the memory, each with the usage percentage inside. Either one turns red at 90 % and above. Hover an icon and the tooltip lists the five programs that used the most CPU over the last ten seconds, or the five that hold the most memory, each with its share. A left click opens Task Manager.

The app is a single small executable written in Rust against the Win32 API. It needs no administrator rights, no extra runtime and about 2.5 MB of memory while resident. It only reads: nothing is written to disk or to the registry, no data leaves your PC, and there is no telemetry.

Start with Windows is off by default and can be turned on from the right-click menu at any time. The interface follows the Windows display language (English, Japanese, Chinese, Korean, German, French, Spanish, Portuguese, Italian and Russian).
```

**What's new in this version**

```
First release.
```

**Product features** (one per line, up to 20)

```
CPU and memory usage as numbers in the system tray, refreshed every second
Tooltips name the five programs using the most CPU or the most memory
Red at 90 % and above
Left click opens Task Manager
Optional start with Windows, off by default
Follows the light and dark taskbar theme and the display scaling
No administrator rights, no extra runtime, about 2.5 MB of memory
Reads only: no disk or registry writes, no network, no telemetry
Speaks 11 languages
```

**Search terms** (up to 7, each up to 30 characters)

```
cpu
memory
ram
system tray
task manager
performance monitor
resource monitor
```

**Copyright and trademark info**

```
© 2026 9enki. MIT License.
```

## Store listing: Japanese (Japan)

**Short title**

```
CPU とメモリの使用率をトレイに
```

**Description**

```
PerformanceTray は、CPU 使用率とメモリ使用率をタスクトレイに数字で表示し、その数字の原因になっているプログラムを教えてくれる小さなアプリです。

トレイには 2 つのアイコンが並びます。チップの枠が CPU、メモリモジュールの形がメモリで、それぞれの中に使用率の数字が入ります。1 秒ごとに更新され、90 % 以上になると赤くなります。アイコンにマウスを載せると、直近 10 秒で CPU を最も使ったプログラム 5 件、またはメモリを最も多く使っているプログラム 5 件が割合付きで表示されます。左クリックでタスク マネージャーが開きます。

Rust と Win32 API だけで書かれた 1 つの小さな実行ファイルで、管理者権限も追加ランタイムも不要、常駐中のメモリは約 2.5 MB です。読み取りしかしません。ディスクやレジストリへの書き込みはなく、データが PC の外に出ることも、テレメトリもありません。

Windows 起動時の自動起動は既定でオフで、右クリックメニューからいつでも切り替えられます。表示言語は Windows の設定に従います（日本語、英語、中国語、韓国語、ドイツ語、フランス語、スペイン語、ポルトガル語、イタリア語、ロシア語）。
```

**What's new in this version**

```
初回リリース。
```

**Product features**

```
CPU とメモリの使用率をタスクトレイに数字で表示（1 秒ごとに更新）
ツールチップに CPU / メモリを最も使っているプログラム 5 件を表示
90 % 以上で赤色
左クリックでタスク マネージャー
Windows 起動時の自動起動（既定はオフ）
タスクバーのライト / ダークテーマと表示倍率に追従
管理者権限・追加ランタイム不要、常駐メモリ約 2.5 MB
読み取りのみ。ディスク・レジストリ書き込みなし、通信なし、テレメトリなし
11 言語対応
```

**Search terms**

```
CPU
メモリ
使用率
タスクトレイ
タスク マネージャー
リソース モニター
パフォーマンス
```

## Screenshots

Desktop screenshots, PNG, 1920 × 1080. Both files live in this folder and are the same images the
README shows.

| File | Caption (English) | Caption (Japanese) |
| --- | --- | --- |
| `screenshot-icons.png` | Two tray icons show the CPU and the memory usage, red from 90 % | CPU とメモリの使用率を 2 つのトレイアイコンで表示。90 % 以上は赤 |
| `screenshot-menu.png` | Tooltips name the programs behind the numbers; the menu toggles start with Windows | ツールチップで原因のプログラムがわかる。メニューで自動起動を切り替え |

## Submission checklist

1. Tag the release (`git tag v1.0.0 && git push --tags`). The Release workflow builds the exe, the
   GitHub Release, the winget manifests and, when the three `STORE_*` repository variables are set,
   the MSIX as the `msix-for-store` artifact
2. Download the `.msix` from the workflow run, or build it locally with `build-msix.ps1` and the
   identity values
3. Partner Center > the app > Start your submission > Packages: upload the `.msix`. The Store signs it
4. Properties, Age ratings, Store listings (en-US and ja-JP): paste the text above and upload the two
   screenshots
5. Submit. Certification usually takes a day or two. Once the app is published, install it from the Store
   and remove the personal install with `install.ps1 -Uninstall`, because two copies cannot run at once
