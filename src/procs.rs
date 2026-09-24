//! プロセスごとの CPU 時間とメモリ使用量。ツールチップの「使用率の高いプログラム上位」に使う。
//!
//! NtQuerySystemInformation(SystemProcessInformation) を 1 回呼ぶだけで、全プロセスの
//! イメージ名・CPU 時間・プライベート ワーキング セットがまとめて取れる。タスク マネージャーが
//! 使うのと同じ情報源で、プロセスを個別に開かないので権限に左右されず、読み取りだけで済む。
//! 公式には「将来変わりうる」とされる API だが、この情報クラスの先頭部分の配置は
//! Windows Vista 以来変わっていない。ここではその先頭部分だけを読む。

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;

const SYSTEM_PROCESS_INFORMATION: u32 = 5;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004_u32 as i32;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023_u32 as i32;
/// 表示するプログラム名の最大文字数。これより長ければ切って "…" を付ける
const MAX_NAME_CHARS: usize = 15;

#[link(name = "ntdll")]
extern "system" {
    fn NtQuerySystemInformation(class: u32, buffer: *mut c_void, length: u32, return_length: *mut u32) -> i32;
}

#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *const u16,
}

/// SYSTEM_PROCESS_INFORMATION の先頭部分。この後にも項目とスレッド情報が続くが使わない。
#[repr(C)]
struct ProcessInfo {
    next_entry_offset: u32,
    number_of_threads: u32,
    working_set_private_size: i64,
    hard_fault_count: u32,
    number_of_threads_high_watermark: u32,
    cycle_time: u64,
    create_time: i64,
    user_time: i64,
    kernel_time: i64,
    image_name: UnicodeString,
    base_priority: i32,
    unique_process_id: usize,
}

/// あるプロセスの 1 回分の計測値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    pub pid: usize,
    /// 起動時刻。PID の再利用を見分けるために使う
    pub created: i64,
    /// イメージ名（例: chrome.exe）
    pub name: String,
    /// カーネル時間 + ユーザー時間（100 ns 単位）
    pub cpu_time: u64,
    /// プライベート ワーキング セット（バイト）。タスク マネージャーの「メモリ」列と同じ
    pub private_working_set: u64,
}

/// 全プロセスの計測値と、そのときの全 CPU の累積時間。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub processes: Vec<Process>,
    /// GetSystemTimes のカーネル + ユーザー時間（全コア合計、アイドル込み）。経過時間 × コア数に相当する
    pub system_total: u64,
}

/// 全プロセスを 1 回計測する。buffer は呼び出し間で使い回す（毎回の確保を避ける）。
pub fn snapshot(buffer: &mut Vec<u8>) -> Option<Snapshot> {
    if buffer.len() < 256 * 1024 {
        buffer.resize(256 * 1024, 0);
    }
    loop {
        let mut needed: u32 = 0;
        let status = unsafe {
            NtQuerySystemInformation(SYSTEM_PROCESS_INFORMATION, buffer.as_mut_ptr().cast(), buffer.len() as u32, &mut needed)
        };
        if status == STATUS_INFO_LENGTH_MISMATCH || status == STATUS_BUFFER_TOO_SMALL {
            // 呼んでいる間にプロセスが増えることがあるので、必要量より少し大きく取る
            let len = (needed as usize).max(buffer.len()) + 64 * 1024;
            buffer.resize(len, 0);
            continue;
        }
        if status < 0 {
            return None;
        }
        break;
    }
    let system_total = crate::stats::cpu_times()?.total;
    Some(Snapshot { processes: parse(buffer), system_total })
}

/// バッファ内の SYSTEM_PROCESS_INFORMATION の連鎖を読む。PID 0（Idle）は含めない。
fn parse(buffer: &[u8]) -> Vec<Process> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while offset + size_of::<ProcessInfo>() <= buffer.len() {
        // バッファの先頭が 8 バイト境界とは限らないので、揃っていなくても読める方法で取る
        let info: ProcessInfo = unsafe { std::ptr::read_unaligned(buffer.as_ptr().add(offset) as *const ProcessInfo) };
        if info.unique_process_id != 0 {
            out.push(Process {
                pid: info.unique_process_id,
                created: info.create_time,
                name: read_name(&info.image_name, buffer),
                cpu_time: info.kernel_time.max(0) as u64 + info.user_time.max(0) as u64,
                private_working_set: info.working_set_private_size.max(0) as u64,
            });
        }
        if info.next_entry_offset == 0 {
            break;
        }
        offset += info.next_entry_offset as usize;
    }
    out
}

/// イメージ名を読む。文字列はバッファの中を指しているので、範囲内であることを確認してから読む。
fn read_name(s: &UnicodeString, buffer: &[u8]) -> String {
    let start = buffer.as_ptr() as usize;
    let p = s.buffer as usize;
    let bytes = s.length as usize;
    if p == 0 || p < start || p + bytes > start + buffer.len() {
        return String::new();
    }
    let units: Vec<u16> = (0..bytes / 2).map(|i| unsafe { std::ptr::read_unaligned(s.buffer.add(i)) }).collect();
    String::from_utf16_lossy(&units)
}

/// ツールチップの 1 行分。
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    pub percent: f32,
}

/// 2 回の計測の差分から、CPU 使用率の高いプログラム上位 n 件。同じイメージ名のプロセスはまとめる。
/// 使用率は全コアの合計に対する割合で、アイコンの数字と同じ基準。
pub fn top_cpu(prev: &Snapshot, cur: &Snapshot, n: usize) -> Vec<Entry> {
    let total = cur.system_total.saturating_sub(prev.system_total);
    if total == 0 {
        return Vec::new();
    }
    let before: HashMap<(usize, i64), u64> = prev.processes.iter().map(|p| ((p.pid, p.created), p.cpu_time)).collect();
    let deltas = cur.processes.iter().map(|p| {
        // 計測の間に始まったプロセスは前回の値が無いので、累積時間をそのまま今回分として数える
        let earlier = before.get(&(p.pid, p.created)).copied().unwrap_or(0);
        (p.name.as_str(), p.cpu_time.saturating_sub(earlier))
    });
    rank(deltas, |sum| sum as f64 * 100.0 / total as f64, n)
}

/// プライベート ワーキング セットの大きいプログラム上位 n 件。物理メモリの搭載量に対する割合。
pub fn top_memory(cur: &Snapshot, total_physical: u64, n: usize) -> Vec<Entry> {
    if total_physical == 0 {
        return Vec::new();
    }
    let sizes = cur.processes.iter().map(|p| (p.name.as_str(), p.private_working_set));
    rank(sizes, |sum| sum as f64 * 100.0 / total_physical as f64, n)
}

/// 名前ごとに値を合計し、割合に変えて大きい順に n 件返す。名前の大文字小文字は区別しない。
fn rank<'a>(items: impl Iterator<Item = (&'a str, u64)>, percent: impl Fn(u64) -> f64, n: usize) -> Vec<Entry> {
    let mut sums: HashMap<String, (String, u64)> = HashMap::new();
    for (name, value) in items {
        let entry = sums.entry(name.to_lowercase()).or_insert_with(|| (display_name(name), 0));
        entry.1 = entry.1.saturating_add(value);
    }
    let mut entries: Vec<Entry> =
        sums.into_values().map(|(name, sum)| Entry { name, percent: percent(sum) as f32 }).collect();
    entries.sort_by(|a, b| b.percent.total_cmp(&a.percent).then_with(|| a.name.cmp(&b.name)));
    entries.truncate(n);
    entries
}

/// 表示用のプログラム名。".exe" を落とし、長ければ切って "…" を付ける。
pub fn display_name(image: &str) -> String {
    let base = match image.len() >= 4 && image[image.len() - 4..].eq_ignore_ascii_case(".exe") {
        true => &image[..image.len() - 4],
        false => image,
    };
    if base.chars().count() <= MAX_NAME_CHARS {
        base.to_string()
    } else {
        let mut s: String = base.chars().take(MAX_NAME_CHARS - 1).collect();
        s.push('…');
        s
    }
}

/// 上位一覧をツールチップの文にする。1 行は「35.2% chrome」。
pub fn format_entries(entries: &[Entry]) -> String {
    entries.iter().map(|e| format!("{:.1}% {}", e.percent, e.name)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: usize, name: &str, cpu_time: u64, memory: u64) -> Process {
        Process { pid, created: 100, name: name.to_string(), cpu_time, private_working_set: memory }
    }

    fn snap(system_total: u64, processes: Vec<Process>) -> Snapshot {
        Snapshot { processes, system_total }
    }

    fn names(entries: &[Entry]) -> Vec<(&str, f32)> {
        entries.iter().map(|e| (e.name.as_str(), e.percent)).collect()
    }

    #[test]
    fn 表示名() {
        assert_eq!(display_name("chrome.exe"), "chrome");
        assert_eq!(display_name("CHROME.EXE"), "CHROME");
        assert_eq!(display_name("System"), "System");
        assert_eq!(display_name(".exe"), "");
        assert_eq!(display_name("exe"), "exe");
        assert_eq!(display_name("SearchIndexer.exe"), "SearchIndexer"); // 13 文字はそのまま
        assert_eq!(display_name("abcdefghijklmno.exe"), "abcdefghijklmno"); // 15 文字はそのまま
        assert_eq!(display_name("SecurityHealthService.exe"), "SecurityHealth…"); // 14 文字 + …
        assert_eq!(display_name("日本語の長い名前のプログラム名前.exe"), "日本語の長い名前のプログラム…"); // 16 文字 → 14 文字 + …
    }

    #[test]
    fn cpu使用率の上位() {
        let prev = snap(1000, vec![proc(1, "chrome.exe", 100, 0), proc(2, "Code.exe", 50, 0), proc(3, "idle.exe", 10, 0)]);
        let cur = snap(2000, vec![
            proc(1, "chrome.exe", 400, 0), // +300 → 30%
            proc(2, "Code.exe", 150, 0),   // +100 → 10%
            proc(3, "idle.exe", 10, 0),    // +0
            proc(4, "new.exe", 50, 0),     // 計測の間に始まった → 50 → 5%
        ]);
        let top = top_cpu(&prev, &cur, 5);
        assert_eq!(names(&top), vec![("chrome", 30.0), ("Code", 10.0), ("new", 5.0), ("idle", 0.0)]);
        assert_eq!(top_cpu(&prev, &cur, 2).len(), 2);
    }

    #[test]
    fn 同じ名前のプロセスはまとめる() {
        let prev = snap(0, vec![proc(1, "chrome.exe", 0, 0), proc(2, "Chrome.exe", 0, 0), proc(3, "CHROME.EXE", 0, 0)]);
        let cur = snap(1000, vec![proc(1, "chrome.exe", 100, 0), proc(2, "Chrome.exe", 200, 0), proc(3, "CHROME.EXE", 300, 0)]);
        assert_eq!(names(&top_cpu(&prev, &cur, 5)), vec![("chrome", 60.0)]);
    }

    #[test]
    fn pidが再利用されたら新しいプロセスとして数える() {
        let prev = snap(0, vec![Process { created: 1, ..proc(7, "a.exe", 900, 0) }]);
        let cur = snap(1000, vec![Process { created: 2, ..proc(7, "a.exe", 100, 0) }]);
        assert_eq!(names(&top_cpu(&prev, &cur, 5)), vec![("a", 10.0)]); // 900 → 100 の巻き戻りにはならない
    }

    #[test]
    fn 終了したプロセスは無視し時間が進まなければ空() {
        let prev = snap(1000, vec![proc(1, "gone.exe", 500, 0)]);
        let cur = snap(2000, vec![]);
        assert!(top_cpu(&prev, &cur, 5).is_empty());
        assert!(top_cpu(&prev, &prev, 5).is_empty()); // system_total の差が 0
    }

    #[test]
    fn メモリ使用量の上位() {
        const GB: u64 = 1 << 30;
        let cur = snap(0, vec![
            proc(1, "chrome.exe", 0, GB),
            proc(2, "chrome.exe", 0, GB),
            proc(3, "Code.exe", 0, GB / 2),
            proc(4, "tiny.exe", 0, 0),
        ]);
        assert_eq!(names(&top_memory(&cur, 16 * GB, 5)), vec![("chrome", 12.5), ("Code", 3.125), ("tiny", 0.0)]);
        assert_eq!(names(&top_memory(&cur, 16 * GB, 1)), vec![("chrome", 12.5)]);
        assert!(top_memory(&cur, 0, 5).is_empty()); // 搭載量が不明なら出さない
    }

    #[test]
    fn 同率なら名前順() {
        let cur = snap(0, vec![proc(1, "b.exe", 0, 100), proc(2, "a.exe", 0, 100), proc(3, "c.exe", 0, 200)]);
        assert_eq!(names(&top_memory(&cur, 1000, 5)), vec![("c", 20.0), ("a", 10.0), ("b", 10.0)]);
    }

    #[test]
    fn ツールチップの文() {
        let entries = vec![
            Entry { name: "chrome".to_string(), percent: 35.25 },
            Entry { name: "Code".to_string(), percent: 0.04 },
        ];
        assert_eq!(format_entries(&entries), "35.2% chrome\n0.0% Code");
        assert_eq!(format_entries(&[]), "");
    }

    #[test]
    fn ツールチップは最悪でも128文字に収まる() {
        // szTip は NUL 込みで 128 文字。値は "100.0%"、名前は最長の 15 文字が 5 行
        let entries: Vec<Entry> =
            (0..5).map(|i| Entry { name: display_name(&format!("{}bcdefghijklmnopqrstuvwxyz.exe", i)), percent: 100.0 }).collect();
        let text = format_entries(&entries);
        assert!(text.encode_utf16().count() <= 127, "{} 文字: {}", text.encode_utf16().count(), text);
    }

    #[test]
    fn 構造体の配置がntdllの定義と一致する() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(ProcessInfo, working_set_private_size), 8);
        assert_eq!(offset_of!(ProcessInfo, cycle_time), 24);
        assert_eq!(offset_of!(ProcessInfo, create_time), 32);
        assert_eq!(offset_of!(ProcessInfo, kernel_time), 48);
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(offset_of!(ProcessInfo, image_name), 0x38);
            assert_eq!(offset_of!(ProcessInfo, unique_process_id), 0x50);
            assert_eq!(size_of::<ProcessInfo>(), 0x58);
        }
    }

    #[test]
    fn バッファを読む() {
        // Idle（PID 0）と System（PID 4）の 2 エントリを組み立てる。名前はバッファの末尾に置く
        let entry_size = 0x100usize;
        let mut buf = vec![0u8; entry_size * 2 + 64];
        let name: Vec<u16> = "System".encode_utf16().collect();
        let name_offset = entry_size * 2;
        for (i, u) in name.iter().enumerate() {
            buf[name_offset + i * 2..name_offset + i * 2 + 2].copy_from_slice(&u.to_le_bytes());
        }
        let name_ptr = unsafe { buf.as_ptr().add(name_offset) } as *const u16;
        let idle = ProcessInfo {
            next_entry_offset: entry_size as u32, number_of_threads: 8, working_set_private_size: 0, hard_fault_count: 0,
            number_of_threads_high_watermark: 0, cycle_time: 0, create_time: 0, user_time: 0, kernel_time: 999,
            image_name: UnicodeString { length: 0, maximum_length: 0, buffer: std::ptr::null() }, base_priority: 0,
            unique_process_id: 0,
        };
        let system = ProcessInfo {
            next_entry_offset: 0, number_of_threads: 100, working_set_private_size: 4096, hard_fault_count: 0,
            number_of_threads_high_watermark: 0, cycle_time: 0, create_time: 12345, user_time: 10, kernel_time: 20,
            image_name: UnicodeString { length: (name.len() * 2) as u16, maximum_length: (name.len() * 2) as u16, buffer: name_ptr },
            base_priority: 8, unique_process_id: 4,
        };
        unsafe {
            std::ptr::write_unaligned(buf.as_mut_ptr() as *mut ProcessInfo, idle);
            std::ptr::write_unaligned(buf.as_mut_ptr().add(entry_size) as *mut ProcessInfo, system);
        }
        let processes = parse(&buf);
        assert_eq!(processes, vec![Process { pid: 4, created: 12345, name: "System".to_string(), cpu_time: 30, private_working_set: 4096 }]);
    }

    #[test]
    fn 実機で全プロセスを読める() {
        let mut buffer = Vec::new();
        let s = snapshot(&mut buffer).expect("NtQuerySystemInformation が失敗");
        assert!(s.processes.iter().any(|p| p.pid == 4 && p.name == "System"), "System プロセスが見つからない");
        assert!(s.processes.iter().all(|p| p.pid != 0));
        let me = std::process::id() as usize;
        assert!(s.processes.iter().any(|p| p.pid == me && p.private_working_set > 0), "自分自身が見つからない");
        assert!(s.system_total > 0);
    }
}
