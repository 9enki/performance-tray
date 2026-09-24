# Privacy Policy

**PerformanceTray does not collect, store, or transmit any personal data.**

Last updated: 2026-09-24

## What the app does

PerformanceTray reads the CPU usage and the physical memory usage of the device it runs on and draws
them as numbers in the system tray. To do so it calls two read-only Windows APIs
(`GetSystemTimes` and `GlobalMemoryStatusEx`) once a second, and reads one registry value to follow
the light/dark theme.

Every ten seconds it also reads the list of running processes (`NtQuerySystemInformation`) to learn
each process's name, CPU time and memory use, so that the tooltips can name the five programs using
the most CPU and the most memory. These names and figures are shown on your screen only. They are
kept in memory just long enough to compute the next tooltip and are never written down or sent anywhere.

A left click on an icon starts the Windows Task Manager; nothing else is launched.

If you turn on "Start with Windows" from the tray menu, the app writes a single value named
`PerformanceTray` under the per-user key
`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`, holding the path to the
executable. Turning the option off deletes that value. Nothing else is ever written.

The Microsoft Store (MSIX) version does not write to the `Run` key. There, the same option turns the
package's startup task on or off through the Windows `StartupTask` API, and Windows keeps that state
itself. It is the same switch as Settings > Apps > Startup.

## Data collection

None. Specifically, the app:

- Collects no personal information, telemetry, analytics, crash reports, or usage statistics
- Makes no network connections of any kind
- Writes nothing to disk and creates no configuration files, logs, or caches
- Writes nothing to the registry, apart from the opt-in "Start with Windows" value described above
- Contains no advertising and no third-party SDKs

The usage figures and the process names exist only in memory while the app runs and are shown on
your screen. They are never recorded or sent anywhere.

## Third parties

No data is shared with anyone, because no data is collected.

## Children

The app collects no data from anyone, including children.

## Changes

Any change to this policy will be published in this file in the project repository.

## Contact

Questions about this policy can be raised as an issue at
https://github.com/9enki/performance-tray/issues
