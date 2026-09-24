# Microsoft Store listing

Text and settings to paste into Partner Center when submitting PerformanceTray. Keep this file in
step with the README and the winget manifests. The Store product page is what most people read, so
the wording here is a little friendlier than the README.

## Identity (Partner Center > Product management > Product identity)

The name **PerformanceTray** is reserved. These are the values Partner Center issued (2026-09-24):

| Field | Value |
| --- | --- |
| Package/Identity/Name | `9enki.PerformanceTray` |
| Package/Identity/Publisher | `CN=25E4528E-D650-45EA-990E-772D522814D6` |
| Package/Properties/PublisherDisplayName | `9enki` |
| Package Family Name | `9enki.PerformanceTray_3eayqh5a0wr90` |
| Store ID | `9N6HWM0261R9` |

The publisher values are the same for every app of the account, so they match PowerModeTray. The MSIX for a
submission is built locally with them (PowerModeTray was released the same way; the repository variables
that the Release workflow can use instead were never set):

```powershell
.\build-msix.ps1 -IdentityName 9enki.PerformanceTray -Publisher 'CN=25E4528E-D650-45EA-990E-772D522814D6' -PublisherDisplayName 9enki
```

Once the product is live it can be installed with `winget install --id 9N6HWM0261R9 --source msstore`, or from
`ms-windows-store://pdp/?productid=9N6HWM0261R9`.

## Upload material

`%USERPROFILE%\Downloads\PerformanceTray-store\` holds everything the submission form asks for. It is made
from the repository like this:

```powershell
.\build-msix.ps1 -IdentityName 9enki.PerformanceTray -Publisher 'CN=25E4528E-D650-45EA-990E-772D522814D6' -PublisherDisplayName 9enki
.\msix\make-assets.ps1 -OutDir obj\assets -StoreLogoDir <folder>      # store-logo-300.png and store-logo-1080.png
# plus docs\screenshot-icons.png and docs\screenshot-menu.png
```

## Submission, page by page

Partner Center > Apps and games > PerformanceTray > Start your submission. Every field not listed here
stays at its default.

### Pricing and availability

| Field | Value |
| --- | --- |
| Markets | Show in all markets (default) |
| Visibility / Discoverability | Make this product available and discoverable in the Store (default) |
| Schedule | As soon as possible (default) |
| Base price | Free |
| Free trial | No free trial |
| Sale pricing | none |
| Organizational licensing | Store-managed (online) volume licensing checked (default), disconnected licensing unchecked |

### Properties

| Field | Value |
| --- | --- |
| Category | Utilities & tools |
| Subcategory | leave empty if the list offers none that fits |
| Privacy policy URL | https://github.com/9enki/performance-tray/blob/main/PRIVACY.md |
| Website | https://github.com/9enki/performance-tray |
| Support contact info | https://github.com/9enki/performance-tray/issues |
| Game settings | not applicable |
| Display mode | nothing checked |
| Product declarations | keep the two defaults (alternate drives, OneDrive backup) checked; everything else unchecked: no in-app purchases, no accessibility claim, no non-Microsoft drivers or NT services, no pen and ink |
| System requirements | leave every row unspecified. The package already restricts the app to x64 |

### Age ratings

Start the IARC questionnaire. Pick the non-game app type ("All other app types"), answer **No** to every
content question (violence, fear, sexuality, nudity, language, controlled substances, gambling, and so on)
and **No** to every interactive element (users interact, shares info, shares location, digital purchases,
unrestricted internet). The result is the all-ages rating in every system (ESRB Everyone, PEGI 3, CERO A,
USK 0, IARC 3+).

### Packages

| Field | Value |
| --- | --- |
| Package | `PerformanceTray_1.0.0.msix` from the upload folder. Wait for the green check |
| Device family availability | Windows 10/11 Desktop checked. Anything else unchecked |
| Let Microsoft decide whether to make the app available to any future device families | unchecked |
| Gradual package rollout / Mandatory update | off |

### Store listings

The package declares 11 languages, so Partner Center lists 11 listing languages. Under *Manage additional
languages* (or *Add/remove languages*) keep **English (United States)** and **Japanese (Japan)** and remove
the rest. Default language: English (United States). If Partner Center refuses to remove a package
language, a description in that language is the only required text.

For each of the two languages, fill the listing with the text in the next two sections and:

| Field | Value |
| --- | --- |
| Product name | PerformanceTray (the reserved name) |
| Screenshots > Desktop | `screenshot-icons.png`, `screenshot-menu.png`, with the captions from the Screenshots table below |
| Store logos > 1:1 box art (300 x 300) | `store-logo-300.png` |
| Store logos > 1:1 app tile icon (1080 x 1080), when the form offers it | `store-logo-1080.png` |
| Trailers, promotional images, sort title, voice title, additional system requirements, additional license terms | leave empty |
| Developed by | 9enki |

### Submission options

| Field | Value |
| --- | --- |
| Publishing hold options | Publish this submission as soon as it passes certification (default) |
| Notes for certification | the text below |
| Restricted capabilities (`runFullTrust`), if a justification box appears | the text below |

Notes for certification:

```
PerformanceTray is a tray-only Win32 app (Desktop Bridge). It has no main window. After launch, two icons appear in the notification area; on a fresh install they sit in the overflow behind the ^ button. Hover an icon for its tooltip, left-click to open Task Manager, right-click for the menu (Open Task Manager / Start with Windows / Exit). The app only reads system usage through documented Win32 APIs. It writes nothing to disk or the registry and makes no network connections. "Start with Windows" toggles the declared StartupTask and is off by default.
```

Justification for `runFullTrust`:

```
Win32 desktop application packaged with the Desktop Bridge. It uses Win32 APIs (Shell_NotifyIcon, GetSystemTimes, GlobalMemoryStatusEx, NtQuerySystemInformation) to show the CPU and memory usage in the notification area. No elevation, no network access.
```

Then **Submit to the Store**. Certification usually takes one to three days and ends with an email.

## Store listing: English (United States)

**Short title** (up to 50 characters)

```
CPU and memory usage in the tray
```

**Short description** (up to 500 characters)

```
CPU and memory usage as numbers in the system tray, refreshed every second, with the programs behind them one hover away.
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

**Short description**

```
CPU とメモリの使用率をタスクトレイに数字で表示。1 秒ごとに更新され、原因のプログラムはマウスを載せればわかります。
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

## After the submission

1. Tag the release (`git tag v1.0.0 && git push --tags`). The Release workflow builds the exe, the GitHub
   Release and the winget manifests
2. Once the app is published, install it from the Store (`winget install --id 9N6HWM0261R9 --source msstore`)
   and remove the personal install with `install.ps1 -Uninstall`, because two copies cannot run at once
