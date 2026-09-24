//! CPU 使用率とメモリ使用率の取得。
//!
//! CPU は GetSystemTimes の累積値を 2 回読んだ差分から「アイドルでなかった時間の割合」を出す。
//! メモリは GlobalMemoryStatusEx の物理メモリ量から計算する。どちらも読み取り専用の公開 API で、
//! 管理者権限は要らず、システムには何も書かない。

use std::mem::{size_of, zeroed};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Threading::GetSystemTimes;

/// GetSystemTimes の累積値（100 ns 単位、全コアの合計）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    /// カーネル時間 + ユーザー時間。カーネル時間にはアイドル時間が含まれている
    pub total: u64,
}

fn filetime_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

/// 現在の累積 CPU 時間。取得できなければ None。
pub fn cpu_times() -> Option<CpuTimes> {
    unsafe {
        let mut idle: FILETIME = zeroed();
        let mut kernel: FILETIME = zeroed();
        let mut user: FILETIME = zeroed();
        (GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0).then(|| CpuTimes {
            idle: filetime_u64(&idle),
            total: filetime_u64(&kernel) + filetime_u64(&user),
        })
    }
}

/// 2 回の計測の差分から CPU 使用率 (0-100) を出す。時間が進んでいなければ None。
pub fn cpu_percent(prev: CpuTimes, cur: CpuTimes) -> Option<u32> {
    let total = cur.total.checked_sub(prev.total).filter(|t| *t > 0)?;
    let idle = cur.idle.saturating_sub(prev.idle).min(total);
    Some(percent(total - idle, total))
}

/// 物理メモリの量（バイト）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memory {
    pub total: u64,
    pub available: u64,
}

impl Memory {
    /// 使用中の量。タスク マネージャーの「使用中」と同じく 搭載量 - 利用可能量
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// 使用率 (0-100)
    pub fn percent(&self) -> u32 {
        percent(self.used(), self.total)
    }
}

/// 現在の物理メモリの状態。取得できなければ None。
pub fn memory() -> Option<Memory> {
    unsafe {
        let mut status: MEMORYSTATUSEX = zeroed();
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
        (GlobalMemoryStatusEx(&mut status) != 0).then(|| Memory {
            total: status.ullTotalPhys,
            available: status.ullAvailPhys,
        })
    }
}

/// part / whole を四捨五入した百分率。whole が 0 なら 0。100 を超えない。
fn percent(part: u64, whole: u64) -> u32 {
    if whole == 0 {
        return 0;
    }
    let part = part.min(whole) as u128;
    let whole = whole as u128;
    ((part * 100 + whole / 2) / whole) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1 << 30;

    fn times(idle: u64, total: u64) -> CpuTimes {
        CpuTimes { idle, total }
    }

    #[test]
    fn 差分から使用率を出す() {
        assert_eq!(cpu_percent(times(0, 0), times(750, 1000)), Some(25));
        assert_eq!(cpu_percent(times(100, 200), times(100, 1200)), Some(100)); // アイドルが増えていない
        assert_eq!(cpu_percent(times(100, 200), times(1100, 1200)), Some(0)); // 全部アイドル
        assert_eq!(cpu_percent(times(0, 0), times(1, 3)), Some(67)); // 四捨五入
        assert_eq!(cpu_percent(times(0, 0), times(2, 3)), Some(33));
    }

    #[test]
    fn 時間が進んでいなければ不明() {
        assert_eq!(cpu_percent(times(500, 1000), times(500, 1000)), None);
        assert_eq!(cpu_percent(times(500, 1000), times(400, 900)), None); // 巻き戻り
    }

    #[test]
    fn アイドルが全体を超えても壊れない() {
        // 別々に読まれる値なので、ごくまれにアイドルの増分が全体の増分を超えることがある
        assert_eq!(cpu_percent(times(0, 0), times(1500, 1000)), Some(0));
    }

    #[test]
    fn メモリの使用量と使用率() {
        let m = Memory { total: 16 * GB, available: 8 * GB };
        assert_eq!(m.used(), 8 * GB);
        assert_eq!(m.percent(), 50);

        let m = Memory { total: 16 * GB, available: 3 * GB };
        assert_eq!(m.percent(), 81); // 13/16 = 81.25

        let m = Memory { total: 0, available: 0 };
        assert_eq!(m.used(), 0);
        assert_eq!(m.percent(), 0);

        let m = Memory { total: 4 * GB, available: 5 * GB }; // 利用可能が搭載量を超える異常値
        assert_eq!(m.used(), 0);
        assert_eq!(m.percent(), 0);
    }

    #[test]
    fn 百分率の丸め() {
        assert_eq!(percent(1, 200), 1); // 0.5 は切り上げ
        assert_eq!(percent(0, 200), 0);
        assert_eq!(percent(199, 200), 100); // 99.5
        assert_eq!(percent(198, 200), 99);
        assert_eq!(percent(300, 200), 100); // 上限で丸める
        assert_eq!(percent(u64::MAX, u64::MAX), 100); // あふれない
    }
}
