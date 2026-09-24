//! PerformanceTray - CPU とメモリの使用率をタスクトレイに数字で表示する最小アプリ。
//!
//! GetSystemTimes と GlobalMemoryStatusEx を 1 秒ごとに読み、使用率をアイコンの数字として描く。
//! 10 秒ごとに NtQuerySystemInformation で全プロセスの CPU 時間とメモリ使用量を読み、
//! 使用率の高いプログラム上位をツールチップに出す。いずれも読み取りだけで、レジストリや
//! ファイルへの書き込み、管理者権限、追加ランタイムは不要。GUI フレームワークを使わず Win32 API だけで動く。
//!
//!   アイコン    : CPU はチップの枠、メモリはメモリモジュールの形で、中に使用率の数字。90% 以上は赤
//!   ツールチップ : CPU アイコンは直近 10 秒で CPU 使用率の高いプログラム上位 5 件、
//!                 メモリアイコンはメモリ使用量の多いプログラム上位 5 件（いずれも割合付き）。
//!                 開いている間はそのアイコンの更新を保留する（更新するとツールチップが閉じるため）
//!   左クリック  : タスク マネージャーを開く
//!   右クリック  : タスク マネージャー / Windows 起動時に実行 / 終了
//!   --show      : cpu / memory / both（既定）でアイコンを絞れる
//!   自動起動    : 右クリックメニューでいつでも切り替えられる（既定は無効）。通常の exe はユーザー単位の
//!                 Run キーに値を 1 つ書き、MSIX（Store 版）はパッケージの StartupTask を切り替える
//!   表示言語    : Windows の表示言語に合わせる（11 言語、未対応なら英語）。--lang で固定できる

#![windows_subsystem = "windows"]

mod cli;
mod i18n;
mod procs;
mod startup;
mod stats;
mod tray;
mod win;

use std::cell::RefCell;
use std::mem::zeroed;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, ERROR_SUCCESS, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows_sys::Win32::UI::Shell::{ShellExecuteW, NIN_SELECT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use cli::{Command, Show};
use procs::Snapshot;
use stats::CpuTimes;
use tray::Style;
use win::{hiword, loword, wide};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 1000; // タスク マネージャーの既定と同じ更新間隔
/// プロセス一覧を採り直す間隔（TIMER_INTERVAL_MS 単位）
const PROCESS_TICKS: u32 = 10;
/// ツールチップに載せるプログラムの数
const TOP_N: usize = 5;
const CMD_TASK_MANAGER: u32 = 1;
const CMD_STARTUP: u32 = 9;
const CMD_EXIT: u32 = 10;
const NIN_KEYSELECT: u32 = NIN_SELECT | 0x1; // キーボードでの選択（NIN_SELECT | NINF_KEY）
const NIN_POPUPOPEN: u32 = 0x406; // ツールチップが開いた（NOTIFYICON_VERSION_4）
const NIN_POPUPCLOSE: u32 = 0x407; // ツールチップが閉じた
/// NIN_POPUPCLOSE を取りこぼしたときの保険。この tick 数で更新の保留を解く
const HOVER_TIMEOUT_TICKS: u32 = 30;
const UNKNOWN_ALPHA: u8 = 110; // 値が取れていないときは薄く描く
const UNKNOWN_TIP: &str = "--";

#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *const core::ffi::c_void, initial_owner: i32, name: *const u16) -> *mut core::ffi::c_void;
}

/// ツールチップの開閉状態。開いている間に NIM_MODIFY するとツールチップが閉じてしまうので、
/// その間はアイコンの更新を保留する。閉じた通知を取りこぼしたときの保険として、時間がたてば自然に解ける。
#[derive(Default)]
struct Hover {
    open: bool,
    ticks: u32,
}

impl Hover {
    fn set(&mut self, open: bool) {
        self.open = open;
        self.ticks = 0;
    }

    /// 1 秒ごとに呼ぶ。開いたまま HOVER_TIMEOUT_TICKS を超えたら閉じたものとみなす
    fn tick(&mut self) {
        if self.open {
            self.ticks += 1;
            if self.ticks > HOVER_TIMEOUT_TICKS {
                self.set(false);
            }
        }
    }

    /// 今アイコンを更新してよいか。まだ登録していないときと強制更新は保留しない
    fn blocks(&self, added: bool, force: bool) -> bool {
        self.open && added && !force
    }
}

/// トレイアイコン 1 つ分の状態。描き直しとツールチップの更新を別々に判断する。
struct TrayIcon {
    id: u32,
    style: Style,
    hicon: HICON,
    added: bool,
    /// 最後に描いた (使用率, ライトテーマか)
    shown: Option<(Option<u32>, bool)>,
    /// 最後に送ったツールチップ
    tip: String,
    hover: Hover,
}

impl TrayIcon {
    fn new(id: u32, style: Style) -> TrayIcon {
        TrayIcon { id, style, hicon: null_mut(), added: false, shown: None, tip: String::new(), hover: Hover::default() }
    }

    /// 表示を今の値に合わせる。変わっていない部分は触らず、ツールチップが開いている間は何も送らない。
    fn sync(&mut self, hwnd: HWND, percent: Option<u32>, light: bool, tip: &str, size: i32, force: bool) {
        if self.hover.blocks(self.added, force) {
            return; // 閉じたら NIN_POPUPCLOSE で追いつく
        }
        let redraw = force || !self.added || self.shown != Some((percent, light));
        let retip = force || !self.added || self.tip != tip;
        if !redraw && !retip {
            return;
        }
        let alpha = if percent.is_some() { 255 } else { UNKNOWN_ALPHA };
        let icon = redraw.then(|| tray::render_icon(percent, size, self.style, tray::color_for(percent, light), alpha));
        if self.added {
            tray::update(hwnd, self.id, icon, retip.then_some(tip));
        } else if let Some(icon) = icon {
            self.added = tray::add(hwnd, self.id, icon, tip);
        }
        if let Some(icon) = icon {
            // Shell 側にコピーされるので、渡した直後に古いアイコンを破棄してよい
            self.destroy_icon();
            self.hicon = icon;
            self.shown = Some((percent, light));
        }
        if retip {
            self.tip = tip.to_string();
        }
    }

    fn destroy_icon(&mut self) {
        if !self.hicon.is_null() {
            unsafe { DestroyIcon(self.hicon) };
            self.hicon = null_mut();
        }
    }
}

struct App {
    hwnd: HWND,
    show: Show,
    /// 自動起動（Run キー方式）に引き継ぐ引数。今の起動引数をそのまま使う。StartupTask には引数を渡せない
    startup_args: String,
    /// 前回の累積 CPU 時間。今回との差分で使用率を出す
    cpu_times: Option<CpuTimes>,
    /// 最新の CPU 使用率。起動直後は差分が取れないので None
    cpu: Option<u32>,
    /// 前回のプロセス一覧。差分で各プログラムの CPU 使用率を出す
    processes: Option<Snapshot>,
    /// NtQuerySystemInformation 用のバッファ。使い回す
    process_buffer: Vec<u8>,
    cpu_tip: String,
    memory_tip: String,
    ticks: u32,
    cpu_icon: TrayIcon,
    memory_icon: TrayIcon,
    taskbar_created: u32,
}

impl App {
    /// 通知を出すアイコン。表示しているものの先頭
    fn notify_icon(&self) -> u32 {
        if self.show.cpu { tray::ICON_CPU } else { tray::ICON_MEMORY }
    }
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.borrow_mut().as_mut().map(f))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    i18n::init(cli::prescan_lang(&args)); // 引数エラーも指定した言語で出すため、解釈より先に決める
    let show = match cli::parse(&args) {
        Err(msg) => {
            message_box(&format!("{}\n\n{}", msg, i18n::s().usage), MB_ICONERROR);
            std::process::exit(2);
        }
        Ok(Command::Help) => {
            message_box(i18n::s().usage, MB_ICONINFORMATION);
            return;
        }
        Ok(Command::Run { show }) => show,
    };

    unsafe {
        // 二重起動はここで静かに終了する。ハンドルはプロセス終了まで保持する
        let _mutex = CreateMutexW(null(), 1, wide(r"Local\PerformanceTray.SingleInstance").as_ptr());
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        run(show);
    }
}

fn message_box(text: &str, icon: u32) {
    unsafe {
        MessageBoxW(null_mut(), wide(text).as_ptr(), wide("PerformanceTray").as_ptr(), MB_OK | icon);
    }
}

unsafe fn run(show: Show) {
    let hinstance = GetModuleHandleW(null());
    let class_name = wide("PerformanceTrayWindow");
    let mut wc: WNDCLASSW = zeroed();
    wc.lpfnWndProc = Some(wndproc);
    wc.hInstance = hinstance;
    wc.lpszClassName = class_name.as_ptr();
    RegisterClassW(&wc);

    // 不可視のトップレベルウィンドウ。TaskbarCreated などのブロードキャストを受けるため message-only にはしない
    let hwnd = CreateWindowExW(
        0, class_name.as_ptr(), wide("PerformanceTray").as_ptr(), WS_OVERLAPPED,
        0, 0, 0, 0, null_mut(), null_mut(), hinstance, null(),
    );
    if hwnd.is_null() {
        message_box(i18n::s().window_failed, MB_ICONERROR);
        std::process::exit(1);
    }

    let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());

    // 最初のプロセス一覧。メモリの上位はすぐ出せる。CPU の上位は 10 秒後の差分から
    let mut process_buffer = Vec::new();
    let processes = procs::snapshot(&mut process_buffer);
    let memory_tip = processes.as_ref().map_or_else(|| UNKNOWN_TIP.to_string(), memory_tip_for);

    APP.with(|a| {
        *a.borrow_mut() = Some(App {
            hwnd,
            show,
            startup_args: show.startup_args(),
            cpu_times: stats::cpu_times(), // 最初の差分の基準
            cpu: None,
            processes,
            process_buffer,
            cpu_tip: i18n::s().tooltip_measuring.to_string(),
            memory_tip,
            ticks: 0,
            cpu_icon: TrayIcon::new(tray::ICON_CPU, Style::Chip),
            memory_icon: TrayIcon::new(tray::ICON_MEMORY, Style::Module),
            taskbar_created,
        })
    });

    refresh(true);
    SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL_MS, None);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TIMER => {
            tick();
            0
        }
        tray::WM_TRAYICON => {
            let icon_id = hiword(lparam as usize);
            match loword(lparam as usize) {
                NIN_POPUPOPEN => set_hovered(icon_id, true),
                NIN_POPUPCLOSE => {
                    set_hovered(icon_id, false);
                    refresh(false); // 保留していた分に追いつく
                }
                NIN_SELECT | NIN_KEYSELECT => {
                    set_hovered(icon_id, false);
                    open_task_manager();
                }
                WM_CONTEXTMENU => {
                    set_hovered(icon_id, false);
                    show_menu(hwnd, loword(wparam) as i16 as i32, hiword(wparam) as i16 as i32);
                }
                _ => {}
            }
            0
        }
        WM_SETTINGCHANGE => {
            refresh(false); // ライト / ダークテーマの変更など
            0
        }
        WM_DESTROY => {
            cleanup();
            PostQuitMessage(0);
            0
        }
        _ => {
            if msg != 0 && Some(msg) == with_app(|a| a.taskbar_created) {
                // Explorer が再起動したのでアイコンを登録し直す
                with_app(|a| {
                    a.cpu_icon.added = false;
                    a.memory_icon.added = false;
                });
                refresh(true);
                return 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

/// ツールチップの開閉を記録する。
fn set_hovered(icon_id: u32, on: bool) {
    with_app(|a| {
        for icon in [&mut a.cpu_icon, &mut a.memory_icon] {
            if icon.id == icon_id {
                icon.hover.set(on);
            }
        }
    });
}

/// 1 秒ごとの計測。CPU は前回との差分で使用率を出し、10 回に 1 回プロセス一覧も採り直す。
fn tick() {
    with_app(|a| {
        a.cpu_icon.hover.tick();
        a.memory_icon.hover.tick();
        if let Some(cur) = stats::cpu_times() {
            match a.cpu_times.and_then(|prev| stats::cpu_percent(prev, cur)) {
                Some(percent) => {
                    a.cpu = Some(percent);
                    a.cpu_times = Some(cur);
                }
                None if a.cpu_times.is_none() => a.cpu_times = Some(cur),
                None => {} // 時間が進んでいない。次回まで前回の基準を保つ
            }
        }
        a.ticks = a.ticks.wrapping_add(1);
        if a.ticks % PROCESS_TICKS == 0 {
            sample_processes(a);
        }
    });
    refresh(false);
}

/// プロセス一覧を採り直し、ツールチップの文を作る。
fn sample_processes(a: &mut App) {
    let Some(cur) = procs::snapshot(&mut a.process_buffer) else { return };
    if let Some(prev) = &a.processes {
        a.cpu_tip = tip_text(procs::top_cpu(prev, &cur, TOP_N));
    }
    a.memory_tip = memory_tip_for(&cur);
    a.processes = Some(cur);
}

fn memory_tip_for(snapshot: &Snapshot) -> String {
    let total = stats::memory().map_or(0, |m| m.total);
    tip_text(procs::top_memory(snapshot, total, TOP_N))
}

fn tip_text(entries: Vec<procs::Entry>) -> String {
    if entries.is_empty() { UNKNOWN_TIP.to_string() } else { procs::format_entries(&entries) }
}

/// 現在の状態をトレイアイコン・ツールチップに反映する。
fn refresh(force: bool) {
    with_app(|a| {
        let memory = stats::memory().map(|m| m.percent());
        let light = is_light_taskbar();
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) }; // 100%:16px, 150%:24px
        if a.show.cpu {
            a.cpu_icon.sync(a.hwnd, a.cpu, light, &a.cpu_tip, size, force);
        }
        if a.show.memory {
            a.memory_icon.sync(a.hwnd, memory, light, &a.memory_tip, size, force);
        }
    });
}

/// タスクバーがライトテーマか（アイコンの色を黒 / 白で切り替えるため）。
fn is_light_taskbar() -> bool {
    unsafe {
        let mut value: u32 = 0;
        let mut size: u32 = 4;
        let rc = RegGetValueW(
            HKEY_CURRENT_USER,
            wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").as_ptr(),
            wide("SystemUsesLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as *mut _,
            &mut size,
        );
        rc == ERROR_SUCCESS && value != 0
    }
}

/// タスク マネージャーを開く。失敗したら通知で知らせる。
fn open_task_manager() {
    let rc = unsafe {
        ShellExecuteW(null_mut(), wide("open").as_ptr(), wide("taskmgr.exe").as_ptr(), null(), null(), SW_SHOWNORMAL)
    };
    if rc as isize <= 32 {
        with_app(|a| tray::notify(a.hwnd, a.notify_icon(), i18n::s().notify_title, i18n::s().task_manager_failed, true));
    }
}

fn show_menu(hwnd: HWND, x: i32, y: i32) {
    // TrackPopupMenuEx はモーダルループで WM_TIMER などを配送するので、借用を持ったまま入らない
    let Some((startup_args, notify_icon)) = with_app(|a| (a.startup_args.clone(), a.notify_icon())) else {
        return;
    };
    let startup_on = startup::is_enabled();

    unsafe {
        let text = i18n::s();
        let menu = CreatePopupMenu();
        AppendMenuW(menu, MF_STRING, CMD_TASK_MANAGER as usize, wide(text.menu_task_manager).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        let startup_flags = if startup_on { MF_STRING | MF_CHECKED } else { MF_STRING };
        AppendMenuW(menu, startup_flags, CMD_STARTUP as usize, wide(text.menu_startup).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, CMD_EXIT as usize, wide(text.menu_exit).as_ptr());

        SetForegroundWindow(hwnd); // メニュー外をクリックしたときに閉じるために必要
        let cmd = TrackPopupMenuEx(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY, x, y, hwnd, null()) as u32;
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        match cmd {
            CMD_TASK_MANAGER => open_task_manager(),
            CMD_STARTUP => toggle_startup(hwnd, notify_icon, startup_on, &startup_args),
            CMD_EXIT => {
                DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}

/// 自動起動の有効・無効を切り替える。
fn toggle_startup(hwnd: HWND, icon: u32, currently_on: bool, args: &str) {
    let text = i18n::s();
    if currently_on {
        if startup::disable() {
            tray::notify(hwnd, icon, text.notify_title, text.startup_off, false);
        } else {
            tray::notify(hwnd, icon, text.notify_title, text.startup_failed, true);
        }
        return;
    }
    match startup::enable(args) {
        Ok(()) => tray::notify(hwnd, icon, text.notify_title, text.startup_on, false),
        Err(startup::EnableError::DisabledInSettings) => {
            // Windows の設定で切られている分はアプリからは戻せないので、その画面を開いて任せる
            tray::notify(hwnd, icon, text.notify_title, text.startup_needs_settings, true);
            unsafe {
                ShellExecuteW(hwnd, null(), wide("ms-settings:startupapps").as_ptr(), null(), null(), SW_SHOWNORMAL);
            }
        }
        Err(startup::EnableError::Failed) => tray::notify(hwnd, icon, text.notify_title, text.startup_failed, true),
    }
}

fn cleanup() {
    with_app(|a| unsafe {
        KillTimer(a.hwnd, TIMER_ID);
        tray::remove(a.hwnd, tray::ICON_CPU);
        tray::remove(a.hwnd, tray::ICON_MEMORY);
        a.cpu_icon.destroy_icon();
        a.memory_icon.destroy_icon();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ツールチップが開いている間だけ更新を保留する() {
        let mut h = Hover::default();
        assert!(!h.blocks(true, false)); // 閉じているときは保留しない
        h.set(true);
        assert!(h.blocks(true, false));
        assert!(!h.blocks(false, false)); // まだ登録していないなら保留しない（最初の登録を止めない）
        assert!(!h.blocks(true, true)); // 強制更新（Explorer 再起動など）は通す
        h.set(false);
        assert!(!h.blocks(true, false));
    }

    #[test]
    fn 閉じた通知を取りこぼしても時間がたてば解ける() {
        let mut h = Hover::default();
        h.set(true);
        for _ in 0..HOVER_TIMEOUT_TICKS {
            h.tick();
            assert!(h.blocks(true, false), "期限内は保留したまま");
        }
        h.tick();
        assert!(!h.blocks(true, false), "期限を超えたら解ける");
    }

    #[test]
    fn 開き直すと数え直す() {
        let mut h = Hover::default();
        h.set(true);
        for _ in 0..HOVER_TIMEOUT_TICKS {
            h.tick();
        }
        h.set(true); // 閉じて開き直した（あるいは開いたまま再通知）
        h.tick();
        assert!(h.blocks(true, false));
    }

    #[test]
    fn 閉じているときは数えない() {
        let mut h = Hover::default();
        for _ in 0..(HOVER_TIMEOUT_TICKS * 2) {
            h.tick();
        }
        assert_eq!(h.ticks, 0);
        h.set(true);
        assert!(h.blocks(true, false));
    }
}
