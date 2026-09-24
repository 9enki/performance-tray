//! 起動引数の解釈。

use crate::i18n::{self, Lang};

/// どのアイコンを出すか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Show {
    pub cpu: bool,
    pub memory: bool,
}

impl Show {
    pub const BOTH: Show = Show { cpu: true, memory: true };
    pub const CPU: Show = Show { cpu: true, memory: false };
    pub const MEMORY: Show = Show { cpu: false, memory: true };

    /// --show の値を解釈する。cpu / memory / both（大文字小文字は区別しない）。未知の値なら None。
    pub fn parse(text: &str) -> Option<Show> {
        match text.trim().to_ascii_lowercase().as_str() {
            "cpu" => Some(Show::CPU),
            "memory" => Some(Show::MEMORY),
            "both" => Some(Show::BOTH),
            _ => None,
        }
    }

    /// 自動起動の Run キーに書く引数。既定（両方）なら空。
    pub fn startup_args(&self) -> String {
        match *self {
            Show::CPU => "--show cpu".to_string(),
            Show::MEMORY => "--show memory".to_string(),
            _ => String::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run { show: Show },
    Help,
}

/// 表示言語だけを先に取り出す。エラーは本解釈の [`parse`] に任せるため、ここでは無視する。
/// 引数の解釈エラーも表示言語に合わせて出したいので、[`parse`] より前に呼ぶ。
pub fn prescan_lang(args: &[String]) -> Lang {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let value = if let Some(v) = arg.get(..7).filter(|p| p.eq_ignore_ascii_case("--lang=")).map(|_| &arg[7..]) {
            v
        } else if arg.eq_ignore_ascii_case("--lang") {
            i += 1;
            match args.get(i) {
                Some(v) => v.as_str(),
                None => return Lang::Auto,
            }
        } else {
            i += 1;
            continue;
        };
        if let Some(lang) = Lang::parse(value) {
            return lang;
        }
        i += 1;
    }
    Lang::Auto
}

/// 引数（プログラム名を除く）を解釈する。失敗なら Err(理由)。
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut show = Show::BOTH;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg.eq_ignore_ascii_case("--help") || arg == "-h" || arg == "/?" {
            return Ok(Command::Help);
        } else if arg.get(..7).map_or(false, |p| p.eq_ignore_ascii_case("--show=")) {
            show = parse_show(&arg[7..])?;
        } else if arg.eq_ignore_ascii_case("--show") {
            i += 1;
            let value = args.get(i).map(|s| s.as_str()).ok_or_else(|| i18n::s().arg_show_needs_value.to_string())?;
            show = parse_show(value)?;
        } else if arg.get(..7).map_or(false, |p| p.eq_ignore_ascii_case("--lang=")) {
            check_lang(&arg[7..])?;
        } else if arg.eq_ignore_ascii_case("--lang") {
            i += 1;
            let value = args.get(i).map(|s| s.as_str()).ok_or_else(|| i18n::s().arg_lang_needs_value.to_string())?;
            check_lang(value)?;
        } else {
            return Err(i18n::fill(i18n::s().arg_unknown, arg));
        }
        i += 1;
    }
    Ok(Command::Run { show })
}

fn parse_show(value: &str) -> Result<Show, String> {
    Show::parse(value).ok_or_else(|| i18n::fill(i18n::s().arg_bad_show, value))
}

/// 表示言語の値を検証する。実際の適用は [`prescan_lang`] 側で済んでいる。
fn check_lang(value: &str) -> Result<(), String> {
    match Lang::parse(value) {
        Some(_) => Ok(()),
        None => Err(format!("{} ({})", i18n::fill(i18n::s().arg_bad_lang, value), i18n::lang_list())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn show(cmd: Result<Command, String>) -> Show {
        match cmd {
            Ok(Command::Run { show }) => show,
            other => panic!("Run を期待したが {:?}", other),
        }
    }

    #[test]
    fn 既定は両方() {
        assert_eq!(show(parse(&args(&[]))), Show::BOTH);
    }

    #[test]
    fn 表示するアイコンの指定() {
        assert_eq!(show(parse(&args(&["--show", "cpu"]))), Show::CPU);
        assert_eq!(show(parse(&args(&["--show=memory"]))), Show::MEMORY);
        assert_eq!(show(parse(&args(&["--SHOW", "Both"]))), Show::BOTH);
        assert_eq!(show(parse(&args(&["--show", "cpu", "--show", "memory"]))), Show::MEMORY); // 後勝ち
    }

    #[test]
    fn 値の解釈() {
        assert_eq!(Show::parse("cpu"), Some(Show::CPU));
        assert_eq!(Show::parse(" CPU "), Some(Show::CPU));
        assert_eq!(Show::parse("memory"), Some(Show::MEMORY));
        assert_eq!(Show::parse("both"), Some(Show::BOTH));
        assert_eq!(Show::parse("mem"), None);
        assert_eq!(Show::parse(""), None);
    }

    #[test]
    fn 自動起動に渡す引数() {
        assert_eq!(Show::BOTH.startup_args(), "");
        assert_eq!(Show::CPU.startup_args(), "--show cpu");
        assert_eq!(Show::MEMORY.startup_args(), "--show memory");
        // 生成した引数を解釈し直すと同じ指定になる
        for s in [Show::BOTH, Show::CPU, Show::MEMORY] {
            let list: Vec<String> = s.startup_args().split_whitespace().map(|p| p.to_string()).collect();
            assert_eq!(show(parse(&list)), s);
        }
    }

    #[test]
    fn ヘルプ() {
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["/?"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["--show", "cpu", "--help"])), Ok(Command::Help));
    }

    #[test]
    fn 表示言語() {
        assert_eq!(prescan_lang(&args(&[])), Lang::Auto);
        assert_eq!(prescan_lang(&args(&["--lang", "en"])), Lang::En);
        assert_eq!(prescan_lang(&args(&["--lang=ja"])), Lang::Ja);
        assert_eq!(prescan_lang(&args(&["--show", "cpu", "--lang", "EN"])), Lang::En);
        assert_eq!(prescan_lang(&args(&["--lang", "zh-Hant"])), Lang::ZhHant);
        assert_eq!(prescan_lang(&args(&["--lang", "xx"])), Lang::Auto); // 不正値は parse 側でエラーにする
        assert_eq!(prescan_lang(&args(&["--lang"])), Lang::Auto);
        // --lang があってもアイコンの指定は変わらない
        assert_eq!(show(parse(&args(&["--lang", "en", "--show", "memory"]))), Show::MEMORY);
    }

    #[test]
    fn 異常系() {
        assert!(parse(&args(&["--lang"])).is_err()); // 値なし
        assert!(parse(&args(&["--lang", "xx"])).is_err()); // 未対応の言語
        assert!(parse(&args(&["--show"])).is_err()); // 値なし
        assert!(parse(&args(&["--show", "gpu"])).is_err()); // 値が不正
        assert!(parse(&args(&["--show="])).is_err()); // 空
        assert!(parse(&args(&["--bogus"])).is_err()); // 不明なオプション
        assert!(parse(&args(&["cpu"])).is_err()); // オプション名なし
    }
}
