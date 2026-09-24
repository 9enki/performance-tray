//! 表示文字列。既定では Windows の表示言語に合わせ、対応が無い言語では英語を使う。
//! 起動引数 --lang で明示的に選ぶこともできる。
//!
//! 「タスク マネージャー」などの語は各言語の Windows の表記に合わせてある。
//! 日本語と英語以外は機械翻訳で、母語話者による確認は受けていない。

use std::sync::OnceLock;
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

/// 画面に出る文字列。`{}` を含むものは [`fill`] で値を埋める。
pub struct Strings {
    /// CPU アイコンのツールチップ。最初の 10 秒はまだ差分が取れないのでこれを出す
    pub tooltip_measuring: &'static str,

    pub menu_task_manager: &'static str,
    pub menu_startup: &'static str,
    pub menu_exit: &'static str,

    pub notify_title: &'static str,
    pub task_manager_failed: &'static str,
    pub startup_failed: &'static str,
    pub startup_on: &'static str,
    pub startup_off: &'static str,
    pub startup_needs_settings: &'static str, // Store 版で、Windows の設定側で無効にされているとき
    pub window_failed: &'static str,

    pub usage: &'static str,
    pub arg_show_needs_value: &'static str,
    pub arg_bad_show: &'static str,        // "Not a valid --show value: '{}' ..."
    pub arg_lang_needs_value: &'static str,
    pub arg_unknown: &'static str,         // "Unknown argument: {}"
    pub arg_bad_lang: &'static str,        // "Not a language: '{}'"
}

/// --show / --lang / --help の説明。どの言語でも同じ形にしてある。
macro_rules! usage {
    ($head:expr, $show:expr, $lang:expr, $help:expr) => {
        concat!(
            $head, "\n\n",
            "  --show <what>   ", $show, "\n",
            "  --lang <lang>   ", $lang, "\n",
            "  --help          ", $help
        )
    };
}

const EN: Strings = Strings {
    tooltip_measuring: "Measuring…",
    menu_task_manager: "Open Task Manager",
    menu_startup: "Start with Windows",
    menu_exit: "Exit",
    notify_title: "PerformanceTray",
    task_manager_failed: "Could not open Task Manager.",
    startup_failed: "Could not change the startup setting.",
    startup_on: "PerformanceTray will start with Windows.",
    startup_off: "PerformanceTray will no longer start with Windows.",
    startup_needs_settings: "Startup is turned off for this app in Windows Settings. Turn it on under Settings > Apps > Startup.",
    window_failed: "Could not create the window.",
    usage: usage!(
        "Usage: PerformanceTray.exe [--show cpu|memory|both] [--lang <lang>]",
        "Which icons to show: cpu, memory or both (default: both)",
        "Interface language (default: follow Windows)",
        "Show this help"
    ),
    arg_show_needs_value: "--show needs a value: cpu, memory or both.",
    arg_bad_show: "Not a valid --show value: '{}' (use cpu / memory / both)",
    arg_lang_needs_value: "--lang needs a language.",
    arg_unknown: "Unknown argument: {}",
    arg_bad_lang: "Not a language: '{}'",
};

const JA: Strings = Strings {
    tooltip_measuring: "計測中…",
    menu_task_manager: "タスク マネージャーを開く",
    menu_startup: "Windows 起動時に実行",
    menu_exit: "終了",
    notify_title: "PerformanceTray",
    task_manager_failed: "タスク マネージャーを開けませんでした。",
    startup_failed: "自動起動の設定を変更できませんでした。",
    startup_on: "Windows 起動時に実行します。",
    startup_off: "Windows 起動時に実行しません。",
    startup_needs_settings: "Windows の設定でこのアプリの自動起動がオフになっています。設定 > アプリ > スタートアップ でオンにしてください。",
    window_failed: "ウィンドウを作成できませんでした。",
    usage: usage!(
        "使い方: PerformanceTray.exe [--show cpu|memory|both] [--lang <言語>]",
        "表示するアイコン: cpu / memory / both（既定: both）",
        "表示言語（既定: Windows に従う）",
        "この説明を表示"
    ),
    arg_show_needs_value: "--show には cpu / memory / both のいずれかを指定してください。",
    arg_bad_show: "--show の値として解釈できません: '{}'（cpu / memory / both のいずれか）",
    arg_lang_needs_value: "--lang には言語を指定してください。",
    arg_unknown: "不明な引数です: {}",
    arg_bad_lang: "言語として解釈できません: '{}'",
};

const ZH_HANS: Strings = Strings {
    tooltip_measuring: "正在测量…",
    menu_task_manager: "打开任务管理器",
    menu_startup: "随 Windows 启动",
    menu_exit: "退出",
    notify_title: "PerformanceTray",
    task_manager_failed: "无法打开任务管理器。",
    startup_failed: "无法更改启动设置。",
    startup_on: "PerformanceTray 将随 Windows 启动。",
    startup_off: "PerformanceTray 将不再随 Windows 启动。",
    startup_needs_settings: "此应用的开机启动已在 Windows 设置中关闭。请在 设置 > 应用 > 启动 中开启。",
    window_failed: "无法创建窗口。",
    usage: usage!(
        "用法: PerformanceTray.exe [--show cpu|memory|both] [--lang <语言>]",
        "要显示的图标: cpu、memory 或 both（默认: both）",
        "界面语言（默认: 跟随 Windows）",
        "显示此帮助"
    ),
    arg_show_needs_value: "--show 需要一个值: cpu、memory 或 both。",
    arg_bad_show: "无效的 --show 值: '{}'（请使用 cpu / memory / both）",
    arg_lang_needs_value: "--lang 需要一个语言。",
    arg_unknown: "未知参数: {}",
    arg_bad_lang: "不是有效的语言: '{}'",
};

const ZH_HANT: Strings = Strings {
    tooltip_measuring: "正在測量…",
    menu_task_manager: "開啟工作管理員",
    menu_startup: "隨 Windows 啟動",
    menu_exit: "結束",
    notify_title: "PerformanceTray",
    task_manager_failed: "無法開啟工作管理員。",
    startup_failed: "無法變更啟動設定。",
    startup_on: "PerformanceTray 將隨 Windows 啟動。",
    startup_off: "PerformanceTray 將不再隨 Windows 啟動。",
    startup_needs_settings: "此應用程式的開機啟動已在 Windows 設定中關閉。請在 設定 > 應用程式 > 啟動 中開啟。",
    window_failed: "無法建立視窗。",
    usage: usage!(
        "用法: PerformanceTray.exe [--show cpu|memory|both] [--lang <語言>]",
        "要顯示的圖示: cpu、memory 或 both（預設: both）",
        "介面語言（預設: 跟隨 Windows）",
        "顯示此說明"
    ),
    arg_show_needs_value: "--show 需要一個值: cpu、memory 或 both。",
    arg_bad_show: "無效的 --show 值: '{}'（請使用 cpu / memory / both）",
    arg_lang_needs_value: "--lang 需要一個語言。",
    arg_unknown: "未知的參數: {}",
    arg_bad_lang: "不是有效的語言: '{}'",
};

const KO: Strings = Strings {
    tooltip_measuring: "측정 중…",
    menu_task_manager: "작업 관리자 열기",
    menu_startup: "Windows 시작 시 실행",
    menu_exit: "종료",
    notify_title: "PerformanceTray",
    task_manager_failed: "작업 관리자를 열 수 없습니다.",
    startup_failed: "시작 설정을 변경할 수 없습니다.",
    startup_on: "PerformanceTray가 Windows 시작 시 실행됩니다.",
    startup_off: "PerformanceTray가 더 이상 Windows 시작 시 실행되지 않습니다.",
    startup_needs_settings: "Windows 설정에서 이 앱의 시작 프로그램이 꺼져 있습니다. 설정 > 앱 > 시작 프로그램 에서 켜 주세요.",
    window_failed: "창을 만들 수 없습니다.",
    usage: usage!(
        "사용법: PerformanceTray.exe [--show cpu|memory|both] [--lang <언어>]",
        "표시할 아이콘: cpu, memory 또는 both (기본값: both)",
        "표시 언어 (기본값: Windows 설정에 따름)",
        "이 도움말 표시"
    ),
    arg_show_needs_value: "--show에는 cpu, memory 또는 both 중 하나를 지정해야 합니다.",
    arg_bad_show: "잘못된 --show 값: '{}' (cpu / memory / both 중 하나)",
    arg_lang_needs_value: "--lang에는 언어를 지정해야 합니다.",
    arg_unknown: "알 수 없는 인수: {}",
    arg_bad_lang: "언어로 해석할 수 없습니다: '{}'",
};

const DE: Strings = Strings {
    tooltip_measuring: "Wird gemessen…",
    menu_task_manager: "Task-Manager öffnen",
    menu_startup: "Mit Windows starten",
    menu_exit: "Beenden",
    notify_title: "PerformanceTray",
    task_manager_failed: "Der Task-Manager konnte nicht geöffnet werden.",
    startup_failed: "Die Autostart-Einstellung konnte nicht geändert werden.",
    startup_on: "PerformanceTray startet jetzt mit Windows.",
    startup_off: "PerformanceTray startet nicht mehr mit Windows.",
    startup_needs_settings: "Der Autostart dieser App ist in den Windows-Einstellungen deaktiviert. Aktivieren Sie ihn unter Einstellungen > Apps > Autostart.",
    window_failed: "Das Fenster konnte nicht erstellt werden.",
    usage: usage!(
        "Verwendung: PerformanceTray.exe [--show cpu|memory|both] [--lang <Sprache>]",
        "Anzuzeigende Symbole: cpu, memory oder both (Standard: both)",
        "Sprache der Oberfläche (Standard: wie Windows)",
        "Diese Hilfe anzeigen"
    ),
    arg_show_needs_value: "--show braucht einen Wert: cpu, memory oder both.",
    arg_bad_show: "Ungültiger Wert für --show: '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang braucht eine Sprache.",
    arg_unknown: "Unbekanntes Argument: {}",
    arg_bad_lang: "Keine gültige Sprache: '{}'",
};

const FR: Strings = Strings {
    tooltip_measuring: "Mesure en cours…",
    menu_task_manager: "Ouvrir le Gestionnaire des tâches",
    menu_startup: "Démarrer avec Windows",
    menu_exit: "Quitter",
    notify_title: "PerformanceTray",
    task_manager_failed: "Impossible d'ouvrir le Gestionnaire des tâches.",
    startup_failed: "Impossible de modifier le paramètre de démarrage.",
    startup_on: "PerformanceTray démarrera avec Windows.",
    startup_off: "PerformanceTray ne démarrera plus avec Windows.",
    startup_needs_settings: "Le démarrage automatique de cette application est désactivé dans les paramètres Windows. Activez-le dans Paramètres > Applications > Démarrage.",
    window_failed: "Impossible de créer la fenêtre.",
    usage: usage!(
        "Utilisation : PerformanceTray.exe [--show cpu|memory|both] [--lang <langue>]",
        "Icônes à afficher : cpu, memory ou both (par défaut : both)",
        "Langue de l'interface (par défaut : celle de Windows)",
        "Afficher cette aide"
    ),
    arg_show_needs_value: "--show attend une valeur : cpu, memory ou both.",
    arg_bad_show: "Valeur invalide pour --show : '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang attend une langue.",
    arg_unknown: "Argument inconnu : {}",
    arg_bad_lang: "Langue inconnue : '{}'",
};

const ES: Strings = Strings {
    tooltip_measuring: "Midiendo…",
    menu_task_manager: "Abrir el Administrador de tareas",
    menu_startup: "Iniciar con Windows",
    menu_exit: "Salir",
    notify_title: "PerformanceTray",
    task_manager_failed: "No se pudo abrir el Administrador de tareas.",
    startup_failed: "No se pudo cambiar la configuración de inicio.",
    startup_on: "PerformanceTray se iniciará con Windows.",
    startup_off: "PerformanceTray ya no se iniciará con Windows.",
    startup_needs_settings: "El inicio automático de esta aplicación está desactivado en la configuración de Windows. Actívelo en Configuración > Aplicaciones > Inicio.",
    window_failed: "No se pudo crear la ventana.",
    usage: usage!(
        "Uso: PerformanceTray.exe [--show cpu|memory|both] [--lang <idioma>]",
        "Iconos que mostrar: cpu, memory o both (predeterminado: both)",
        "Idioma de la interfaz (predeterminado: el de Windows)",
        "Mostrar esta ayuda"
    ),
    arg_show_needs_value: "--show necesita un valor: cpu, memory o both.",
    arg_bad_show: "Valor no válido para --show: '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang necesita un idioma.",
    arg_unknown: "Argumento desconocido: {}",
    arg_bad_lang: "No es un idioma: '{}'",
};

const PT: Strings = Strings {
    tooltip_measuring: "Medindo…",
    menu_task_manager: "Abrir o Gerenciador de Tarefas",
    menu_startup: "Iniciar com o Windows",
    menu_exit: "Sair",
    notify_title: "PerformanceTray",
    task_manager_failed: "Não foi possível abrir o Gerenciador de Tarefas.",
    startup_failed: "Não foi possível alterar a configuração de inicialização.",
    startup_on: "O PerformanceTray será iniciado com o Windows.",
    startup_off: "O PerformanceTray não será mais iniciado com o Windows.",
    startup_needs_settings: "A inicialização automática deste aplicativo está desativada nas configurações do Windows. Ative-a em Configurações > Aplicativos > Inicialização.",
    window_failed: "Não foi possível criar a janela.",
    usage: usage!(
        "Uso: PerformanceTray.exe [--show cpu|memory|both] [--lang <idioma>]",
        "Ícones a exibir: cpu, memory ou both (padrão: both)",
        "Idioma da interface (padrão: o do Windows)",
        "Mostrar esta ajuda"
    ),
    arg_show_needs_value: "--show precisa de um valor: cpu, memory ou both.",
    arg_bad_show: "Valor inválido para --show: '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang precisa de um idioma.",
    arg_unknown: "Argumento desconhecido: {}",
    arg_bad_lang: "Não é um idioma: '{}'",
};

const IT: Strings = Strings {
    tooltip_measuring: "Misurazione in corso…",
    menu_task_manager: "Apri Gestione attività",
    menu_startup: "Avvia con Windows",
    menu_exit: "Esci",
    notify_title: "PerformanceTray",
    task_manager_failed: "Impossibile aprire Gestione attività.",
    startup_failed: "Impossibile modificare l'impostazione di avvio.",
    startup_on: "PerformanceTray verrà avviato con Windows.",
    startup_off: "PerformanceTray non verrà più avviato con Windows.",
    startup_needs_settings: "L'avvio automatico di questa app è disattivato nelle impostazioni di Windows. Attivalo in Impostazioni > App > Avvio.",
    window_failed: "Impossibile creare la finestra.",
    usage: usage!(
        "Uso: PerformanceTray.exe [--show cpu|memory|both] [--lang <lingua>]",
        "Icone da mostrare: cpu, memory o both (predefinito: both)",
        "Lingua dell'interfaccia (predefinito: quella di Windows)",
        "Mostra questa guida"
    ),
    arg_show_needs_value: "--show richiede un valore: cpu, memory o both.",
    arg_bad_show: "Valore non valido per --show: '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang richiede una lingua.",
    arg_unknown: "Argomento sconosciuto: {}",
    arg_bad_lang: "Non è una lingua: '{}'",
};

const RU: Strings = Strings {
    tooltip_measuring: "Измерение…",
    menu_task_manager: "Открыть диспетчер задач",
    menu_startup: "Запускать вместе с Windows",
    menu_exit: "Выход",
    notify_title: "PerformanceTray",
    task_manager_failed: "Не удалось открыть диспетчер задач.",
    startup_failed: "Не удалось изменить настройку автозапуска.",
    startup_on: "PerformanceTray будет запускаться вместе с Windows.",
    startup_off: "PerformanceTray больше не будет запускаться вместе с Windows.",
    startup_needs_settings: "Автозапуск этого приложения отключён в параметрах Windows. Включите его в разделе Параметры > Приложения > Автозагрузка.",
    window_failed: "Не удалось создать окно.",
    usage: usage!(
        "Использование: PerformanceTray.exe [--show cpu|memory|both] [--lang <язык>]",
        "Какие значки показывать: cpu, memory или both (по умолчанию: both)",
        "Язык интерфейса (по умолчанию: как в Windows)",
        "Показать эту справку"
    ),
    arg_show_needs_value: "--show требует значение: cpu, memory или both.",
    arg_bad_show: "Недопустимое значение --show: '{}' (cpu / memory / both)",
    arg_lang_needs_value: "--lang требует указать язык.",
    arg_unknown: "Неизвестный аргумент: {}",
    arg_bad_lang: "Не является языком: '{}'",
};

/// 表示言語の指定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Windows の表示言語に従う
    Auto,
    En,
    Ja,
    ZhHans,
    ZhHant,
    Ko,
    De,
    Fr,
    Es,
    Pt,
    It,
    Ru,
}

/// --lang で受け付ける名前。先頭が正規の名前で、--help の一覧にも使う。
const LANG_NAMES: &[(Lang, &[&str])] = &[
    (Lang::En, &["en", "en-us", "en-gb", "english"]),
    (Lang::Ja, &["ja", "ja-jp", "japanese"]),
    (Lang::ZhHans, &["zh-hans", "zh", "zh-cn", "zh-sg"]),
    (Lang::ZhHant, &["zh-hant", "zh-tw", "zh-hk", "zh-mo"]),
    (Lang::Ko, &["ko", "ko-kr", "korean"]),
    (Lang::De, &["de", "de-de", "german"]),
    (Lang::Fr, &["fr", "fr-fr", "french"]),
    (Lang::Es, &["es", "es-es", "es-mx", "spanish"]),
    (Lang::Pt, &["pt", "pt-br", "pt-pt", "portuguese"]),
    (Lang::It, &["it", "it-it", "italian"]),
    (Lang::Ru, &["ru", "ru-ru", "russian"]),
];

// LANGID の下位 10 bit（主言語 ID）
const LANG_CHINESE: u16 = 0x04;
const LANG_GERMAN: u16 = 0x07;
const LANG_SPANISH: u16 = 0x0A;
const LANG_FRENCH: u16 = 0x0C;
const LANG_ITALIAN: u16 = 0x10;
const LANG_JAPANESE: u16 = 0x11;
const LANG_KOREAN: u16 = 0x12;
const LANG_PORTUGUESE: u16 = 0x16;
const LANG_RUSSIAN: u16 = 0x19;

impl Lang {
    /// "en" や "zh-Hant" などを解釈する。"auto" は Auto。未知の値なら None。
    pub fn parse(text: &str) -> Option<Lang> {
        let name = text.trim().to_ascii_lowercase();
        if name == "auto" {
            return Some(Lang::Auto);
        }
        LANG_NAMES
            .iter()
            .find(|(_, names)| names.contains(&name.as_str()))
            .map(|(lang, _)| *lang)
    }

    /// Windows の LANGID から表示言語を決める。対応が無ければ英語。
    fn from_langid(langid: u16) -> Lang {
        let primary = langid & 0x3FF;
        let sub = langid >> 10;
        match primary {
            LANG_JAPANESE => Lang::Ja,
            // 繁体字は台湾(1) / 香港(3) / マカオ(5)、簡体字は中国(2) / シンガポール(4)
            LANG_CHINESE => {
                if sub == 1 || sub == 3 || sub == 5 {
                    Lang::ZhHant
                } else {
                    Lang::ZhHans
                }
            }
            LANG_KOREAN => Lang::Ko,
            LANG_GERMAN => Lang::De,
            LANG_FRENCH => Lang::Fr,
            LANG_SPANISH => Lang::Es,
            LANG_PORTUGUESE => Lang::Pt,
            LANG_ITALIAN => Lang::It,
            LANG_RUSSIAN => Lang::Ru,
            _ => Lang::En,
        }
    }

    fn strings(self) -> &'static Strings {
        match self {
            Lang::Auto => Lang::from_langid(unsafe { GetUserDefaultUILanguage() }).strings(),
            Lang::En => &EN,
            Lang::Ja => &JA,
            Lang::ZhHans => &ZH_HANS,
            Lang::ZhHant => &ZH_HANT,
            Lang::Ko => &KO,
            Lang::De => &DE,
            Lang::Fr => &FR,
            Lang::Es => &ES,
            Lang::Pt => &PT,
            Lang::It => &IT,
            Lang::Ru => &RU,
        }
    }
}

/// --lang に指定できる言語名を並べた一文。エラーメッセージに添える。
pub fn lang_list() -> String {
    let names: Vec<&str> = LANG_NAMES.iter().map(|(_, n)| n[0]).collect();
    names.join(", ")
}

static CHOSEN: OnceLock<&'static Strings> = OnceLock::new();

/// 表示言語を決める。[`s`] を最初に呼ぶ前に 1 回だけ呼ぶこと。
pub fn init(lang: Lang) {
    let _ = CHOSEN.set(lang.strings());
}

/// 表示言語に合った文字列。[`init`] が呼ばれていなければ Windows の設定に従う。
pub fn s() -> &'static Strings {
    CHOSEN.get_or_init(|| Lang::Auto.strings())
}

/// テンプレート内の最初の `{}` を値に置き換える。
pub fn fill(template: &str, value: &str) -> String {
    template.replacen("{}", value, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 収録している全言語。テストで網羅するために使う。
    const ALL: &[(&str, &Strings)] = &[
        ("en", &EN), ("ja", &JA), ("zh-Hans", &ZH_HANS), ("zh-Hant", &ZH_HANT), ("ko", &KO),
        ("de", &DE), ("fr", &FR), ("es", &ES), ("pt", &PT), ("it", &IT), ("ru", &RU),
    ];

    #[test]
    fn 値を埋める() {
        assert_eq!(fill("Unknown argument: {}", "--x"), "Unknown argument: --x");
        assert_eq!(fill("no placeholder", "x"), "no placeholder");
        assert_eq!(fill("{} and {}", "a"), "a and {}"); // 置き換えるのは最初の 1 つだけ
    }

    #[test]
    fn 言語名を解釈する() {
        assert_eq!(Lang::parse("en"), Some(Lang::En));
        assert_eq!(Lang::parse("EN-US"), Some(Lang::En));
        assert_eq!(Lang::parse(" ja "), Some(Lang::Ja));
        assert_eq!(Lang::parse("zh-Hant"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-TW"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-CN"), Some(Lang::ZhHans));
        assert_eq!(Lang::parse("pt-BR"), Some(Lang::Pt));
        assert_eq!(Lang::parse("auto"), Some(Lang::Auto));
        assert_eq!(Lang::parse("xx"), None);
        assert_eq!(Lang::parse(""), None);
    }

    #[test]
    fn 言語名が重複していない() {
        let mut seen = Vec::new();
        for (_, names) in LANG_NAMES {
            for n in *names {
                assert!(!seen.contains(n), "重複した言語名: {}", n);
                assert_eq!(*n, n.to_ascii_lowercase(), "小文字で書くこと: {}", n);
                seen.push(n);
            }
        }
    }

    #[test]
    fn langid_から言語を選ぶ() {
        assert_eq!(Lang::from_langid(0x0411), Lang::Ja); // ja-JP
        assert_eq!(Lang::from_langid(0x0409), Lang::En); // en-US
        assert_eq!(Lang::from_langid(0x0804), Lang::ZhHans); // zh-CN
        assert_eq!(Lang::from_langid(0x0404), Lang::ZhHant); // zh-TW
        assert_eq!(Lang::from_langid(0x0C04), Lang::ZhHant); // zh-HK
        assert_eq!(Lang::from_langid(0x0412), Lang::Ko); // ko-KR
        assert_eq!(Lang::from_langid(0x0416), Lang::Pt); // pt-BR
        assert_eq!(Lang::from_langid(0x0419), Lang::Ru); // ru-RU
        assert_eq!(Lang::from_langid(0x040B), Lang::En); // fi-FI（未対応なので英語）
    }

    #[test]
    fn 書式指定子の数が全言語で一致する() {
        for (name, t) in ALL {
            for (field, template) in [
                ("arg_bad_show", t.arg_bad_show),
                ("arg_unknown", t.arg_unknown),
                ("arg_bad_lang", t.arg_bad_lang),
            ] {
                assert_eq!(template.matches("{}").count(), 1, "{} の {} に {{}} が 1 つない", name, field);
            }
        }
    }

    #[test]
    fn 値を持たない文字列に書式指定子がない() {
        for (name, t) in ALL {
            for (field, text) in [
                ("tooltip_measuring", t.tooltip_measuring),
                ("menu_task_manager", t.menu_task_manager),
                ("menu_startup", t.menu_startup),
                ("menu_exit", t.menu_exit),
                ("notify_title", t.notify_title),
                ("task_manager_failed", t.task_manager_failed),
                ("startup_failed", t.startup_failed),
                ("startup_on", t.startup_on),
                ("startup_off", t.startup_off),
                ("startup_needs_settings", t.startup_needs_settings),
                ("window_failed", t.window_failed),
                ("usage", t.usage),
                ("arg_show_needs_value", t.arg_show_needs_value),
                ("arg_lang_needs_value", t.arg_lang_needs_value),
            ] {
                assert!(!text.contains("{}"), "{} の {} に不要な {{}} がある", name, field);
                assert!(!text.is_empty(), "{} の {} が空", name, field);
            }
        }
    }

    #[test]
    fn 使い方に全オプションが載っている() {
        for (name, t) in ALL {
            for option in ["--show", "--lang", "--help", "cpu|memory|both"] {
                assert!(t.usage.contains(option), "{} の usage に {} がない", name, option);
            }
        }
    }

    #[test]
    fn 全言語に文字列がそろっている() {
        assert_eq!(ALL.len(), LANG_NAMES.len(), "ALL と LANG_NAMES の件数が違う");
        for (lang, names) in LANG_NAMES {
            // 正規名で引いた Strings が、その言語の strings() と同じ実体であること
            let by_name = Lang::parse(names[0]).expect("正規名が解釈できない");
            assert_eq!(by_name, *lang);
            assert!(std::ptr::eq(by_name.strings(), lang.strings()));
        }
    }

    #[test]
    fn 言語一覧を作れる() {
        let list = lang_list();
        assert!(list.starts_with("en, ja, zh-hans"), "実際の値: {}", list);
        assert_eq!(list.split(", ").count(), LANG_NAMES.len());
    }
}
