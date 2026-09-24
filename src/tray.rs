//! トレイアイコンの描画と Shell_NotifyIcon の操作。
//!
//! CPU は四辺にピンの付いたチップの枠、メモリは下辺に端子の付いたメモリモジュールの形で、
//! どちらも中に使用率の数字を置く。数字は Segoe UI Bold で描き、収める幅に入らなければ
//! GDI の横方向スケーリングで詰める。図形はピクセルごとに被覆率を計算して描くので、
//! どのサイズでも縁が滑らかに出る。

use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HWND, RECT, SIZE};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::win::{copy_wide, wide};

/// トレイからのコールバックメッセージ
pub const WM_TRAYICON: u32 = WM_APP + 1;
/// CPU 用アイコンの識別子
pub const ICON_CPU: u32 = 1;
/// メモリ用アイコンの識別子
pub const ICON_MEMORY: u32 = 2;

/// この値以上の使用率は赤く描いて目立たせる
pub const WARN_PERCENT: u32 = 90;
const WARN_COLOR: Color = Color { r: 0xE7, g: 0x48, b: 0x56 };
const FONT_FACE: &str = "Segoe UI";

/// アイコンの形。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// CPU: 四辺にピンの付いたチップの枠。数字はその中に描く
    Chip,
    /// メモリ: 下辺に端子の付いたメモリモジュールの塗り。数字はその中を抜く
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// アイコンに描く文字。不明なら "--"。100 は 3 桁のまま描く（横に詰まる）。
pub fn label(percent: Option<u32>) -> String {
    match percent {
        Some(p) => p.min(100).to_string(),
        None => "--".to_string(),
    }
}

/// テーマと使用率から色を決める。ライトテーマは黒、ダークテーマは白、WARN_PERCENT 以上は赤。
pub fn color_for(percent: Option<u32>, light_theme: bool) -> Color {
    match percent {
        Some(p) if p >= WARN_PERCENT => WARN_COLOR,
        _ if light_theme => Color { r: 0, g: 0, b: 0 },
        _ => Color { r: 255, g: 255, b: 255 },
    }
}

/// 使用率を描いた HICON を作る。alpha は 0-255 で全体の濃さ。呼び出し側が DestroyIcon する。
pub fn render_icon(percent: Option<u32>, size: i32, style: Style, color: Color, alpha: u8) -> HICON {
    unsafe {
        let screen = GetDC(null_mut());
        let hdc = CreateCompatibleDC(screen);
        ReleaseDC(null_mut(), screen);

        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = size;
        bmi.bmiHeader.biHeight = -size; // 上から下
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB as u32;
        let mut bits: *mut core::ffi::c_void = null_mut();
        let color_bmp = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        let old_bmp = SelectObject(hdc, color_bmp);
        let pixel_count = (size * size) as usize;
        std::ptr::write_bytes(bits as *mut u8, 0, pixel_count * 4); // 黒・透明で初期化
        let px = bits as *mut u8;

        // 数字を白で描き、その濃さ（緑チャンネル）を文字の被覆率として取り出す
        let text_box = text_box(style, size);
        let mut wtext: Vec<u16> = label(percent).encode_utf16().collect();
        let font = fit_font(hdc, &wtext, text_box.em, text_box.max_width);
        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, 0x00FF_FFFF);
        let mut rc = text_box.rect;
        DrawTextW(hdc, wtext.as_mut_ptr(), wtext.len() as i32, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP);
        GdiFlush();
        let mut text_cov = vec![0u8; pixel_count];
        for (i, c) in text_cov.iter_mut().enumerate() {
            *c = *px.add(i * 4 + 1);
        }
        // 行送りで中央揃えすると数字が少し下に寄るので、描いた結果で枠の真ん中に合わせる
        center_in(&mut text_cov, size, &text_box.rect);
        SelectObject(hdc, old_font);
        DeleteObject(font);

        // BGRA・非乗算アルファで塗る。図形と文字の被覆率からピクセルごとにアルファを決める
        for y in 0..size {
            for x in 0..size {
                let i = (y * size + x) as usize;
                let a = pixel_alpha(style, x, y, size, text_cov[i] as f32 / 255.0);
                let p = px.add(i * 4);
                *p = color.b;
                *p.add(1) = color.g;
                *p.add(2) = color.r;
                *p.add(3) = (a.clamp(0.0, 1.0) * alpha as f32).round() as u8;
            }
        }

        SelectObject(hdc, old_bmp);
        DeleteDC(hdc);

        let mask = CreateBitmap(size, size, 1, 1, null()); // アルファ付きなのでマスクは形だけ
        let info = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color_bmp };
        let icon = CreateIconIndirect(&info);
        DeleteObject(mask);
        DeleteObject(color_bmp);
        icon
    }
}

unsafe fn create_font(em: i32, width: i32) -> HFONT {
    CreateFontW(-em, width, 0, 0, 700, 0, 0, 0, DEFAULT_CHARSET as u32, OUT_TT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32,
        ANTIALIASED_QUALITY as u32, (DEFAULT_PITCH | FF_DONTCARE) as u32, wide(FONT_FACE).as_ptr())
}

/// em の高さのフォントを作り、text の幅が max_width を超えるなら横に詰めたフォントに差し替える。
/// GDI は平均文字幅の指定に合わせてグリフを横方向に拡縮するので、それを使う。
unsafe fn fit_font(hdc: HDC, text: &[u16], em: i32, max_width: i32) -> HFONT {
    let font = create_font(em, 0);
    let old = SelectObject(hdc, font);
    let mut extent: SIZE = zeroed();
    GetTextExtentPoint32W(hdc, text.as_ptr(), text.len() as i32, &mut extent);
    let mut tm: TEXTMETRICW = zeroed();
    GetTextMetricsW(hdc, &mut tm);
    SelectObject(hdc, old);
    if extent.cx <= max_width || extent.cx <= 0 || tm.tmAveCharWidth <= 0 {
        return font;
    }
    DeleteObject(font);
    let width = (tm.tmAveCharWidth * max_width / extent.cx).max(1);
    create_font(em, width)
}

/// 数字を描く場所と大きさ。
struct TextBox {
    rect: RECT,
    /// フォントの em の高さ（px）
    em: i32,
    /// 収める幅（px）。超えるなら横に詰める
    max_width: i32,
}

/// size に比例した px 数。min より小さくはしない。
fn scaled(size: i32, ratio: f32, min: i32) -> i32 {
    ((size as f32 * ratio).round() as i32).max(min)
}

/// 形ごとの数字の置き場。
fn text_box(style: Style, size: i32) -> TextBox {
    match style {
        Style::Chip => {
            // 枠の内側いっぱいに使う
            let (x0, y0, x1, y1) = Chip::new(size).inner;
            TextBox {
                rect: RECT { left: x0 as i32, top: y0 as i32, right: x1 as i32, bottom: y1 as i32 },
                em: ((y1 - y0) * 1.1).round() as i32,
                max_width: (x1 - x0) as i32,
            }
        }
        Style::Module => {
            // 塗りの中。左右に余白を取り、端子の分だけ上に寄る
            let pad = scaled(size, 0.125, 1);
            let body_h = Module::new(size).body.3 as i32;
            TextBox {
                rect: RECT { left: pad, top: 1, right: size - pad, bottom: body_h - 1 },
                em: body_h - 2,
                max_width: size - 2 * pad,
            }
        }
    }
}

/// ピクセル (x, y) のアルファ（0-1）。text は文字の被覆率 0-1。
fn pixel_alpha(style: Style, x: i32, y: i32, size: i32, text: f32) -> f32 {
    match style {
        Style::Chip => {
            let c = Chip::new(size);
            c.frame_cov(x, y, size) + c.inner_cov(x, y) * text
        }
        Style::Module => {
            let m = Module::new(size);
            m.teeth_cov(x, y, size) + m.body_cov(x, y) * (1.0 - text)
        }
    }
}

/// 軸に平行な長方形 [x0,x1)×[y0,y1) がピクセル (x, y) を覆う割合（0-1）。
fn rect_cov(x: i32, y: i32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let ox = (x1.min(x as f32 + 1.0) - x0.max(x as f32)).clamp(0.0, 1.0);
    let oy = (y1.min(y as f32 + 1.0) - y0.max(y as f32)).clamp(0.0, 1.0);
    ox * oy
}

/// 角丸長方形 [x0,x1)×[y0,y1)（角の半径 r）がピクセル (x, y) を覆う割合（0-1）。縁は 1 px ぶん滑らかにする。
fn rrect_cov(x: i32, y: i32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
    let hw = (x1 - x0) / 2.0;
    let hh = (y1 - y0) / 2.0;
    if hw <= 0.0 || hh <= 0.0 {
        return 0.0;
    }
    let r = r.clamp(0.0, hw.min(hh));
    let qx = (x as f32 + 0.5 - (x0 + hw)).abs() - (hw - r);
    let qy = (y as f32 + 0.5 - (y0 + hh)).abs() - (hh - r);
    // 縁からの符号付き距離。外側は角の円までの距離、内側は近い辺までの距離（負）
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside = qx.max(qy).min(0.0);
    let distance = outside + inside - r;
    (0.5 - distance).clamp(0.0, 1.0)
}

/// チップ: 四辺にピンの付いた四角の枠。
struct Chip {
    /// 枠の外形 (x0, y0, x1, y1)
    body: (f32, f32, f32, f32),
    /// 枠の内側。数字はここに描く
    inner: (f32, f32, f32, f32),
    /// 枠の太さ = ピンの幅
    t: f32,
    radius: f32,
}

impl Chip {
    fn new(size: i32) -> Chip {
        let pin = scaled(size, 0.09, 1) as f32; // 16px:1, 24px:2, 32px:3
        let t = scaled(size, 1.0 / 16.0, 1) as f32; // 16px:1, 24px:2, 32px:2
        let s = size as f32;
        Chip {
            body: (pin, pin, s - pin, s - pin),
            inner: (pin + t, pin + t, s - pin - t, s - pin - t),
            t,
            radius: s * 0.08,
        }
    }

    fn inner_cov(&self, x: i32, y: i32) -> f32 {
        let (x0, y0, x1, y1) = self.inner;
        rrect_cov(x, y, x0, y0, x1, y1, (self.radius - self.t).max(0.0))
    }

    /// 枠とピン。
    fn frame_cov(&self, x: i32, y: i32, size: i32) -> f32 {
        let (x0, y0, x1, y1) = self.body;
        let ring = (rrect_cov(x, y, x0, y0, x1, y1, self.radius) - self.inner_cov(x, y)).max(0.0);
        (ring + self.pins_cov(x, y, size)).min(1.0)
    }

    /// 四辺のピン。各辺に 3 本、辺の 1/4・1/2・3/4 の位置。
    fn pins_cov(&self, x: i32, y: i32, size: i32) -> f32 {
        let (x0, y0, x1, y1) = self.body;
        let s = size as f32;
        let len = x1 - x0;
        let mut pins = 0.0;
        for k in 1..=3 {
            // ピンの位置はピクセル境界に揃えてにじませない
            let a0 = (x0 + len * k as f32 / 4.0 - self.t / 2.0).round();
            let a1 = a0 + self.t;
            pins += rect_cov(x, y, a0, 0.0, a1, y0); // 上
            pins += rect_cov(x, y, a0, y1, a1, s); // 下
            pins += rect_cov(x, y, 0.0, a0, x0, a1); // 左
            pins += rect_cov(x, y, x1, a0, s, a1); // 右
        }
        pins.min(1.0)
    }
}

/// メモリモジュール: 角丸の本体と、下辺に並ぶ端子。
struct Module {
    body: (f32, f32, f32, f32),
    radius: f32,
    /// 端子 1 本の幅 = 端子どうしの間隔
    tooth_w: f32,
    teeth: i32,
    /// 最初の端子の左端
    teeth_start: f32,
}

impl Module {
    fn new(size: i32) -> Module {
        let s = size as f32;
        let teeth_h = scaled(size, 0.1875, 2) as f32; // 16px:3, 24px:5, 32px:6
        let w = scaled(size, 1.0 / 16.0, 1) as f32;
        let avail = s - 2.0 * w;
        let teeth = ((avail / w + 1.0) / 2.0).floor() as i32; // (2n-1)w が avail に収まる n
        let total = (2 * teeth - 1) as f32 * w;
        Module {
            body: (0.0, 0.0, s, s - teeth_h),
            radius: s * 0.15,
            tooth_w: w,
            teeth,
            teeth_start: ((s - total) / 2.0).floor(),
        }
    }

    fn body_cov(&self, x: i32, y: i32) -> f32 {
        let (x0, y0, x1, y1) = self.body;
        rrect_cov(x, y, x0, y0, x1, y1, self.radius)
    }

    fn teeth_cov(&self, x: i32, y: i32, size: i32) -> f32 {
        let mut cov = 0.0;
        for k in 0..self.teeth {
            let a0 = self.teeth_start + (2 * k) as f32 * self.tooth_w;
            cov += rect_cov(x, y, a0, self.body.3, a0 + self.tooth_w, size as f32);
        }
        cov.min(1.0)
    }
}

/// 描かれた部分の外接矩形が rect の中心に来るよう、カバレッジ全体をずらす。
fn center_in(coverage: &mut [u8], size: i32, rect: &RECT) {
    let s = size as usize;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (s, s, 0usize, 0usize);
    for y in 0..s {
        for x in 0..s {
            if coverage[y * s + x] != 0 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x > max_x {
        return; // 何も描かれていない
    }
    let w = (max_x - min_x + 1) as isize;
    let h = (max_y - min_y + 1) as isize;
    // 余りの 1 px は右・下に寄せる
    let dx = rect.left as isize + ((rect.right - rect.left) as isize - w) / 2 - min_x as isize;
    let dy = rect.top as isize + ((rect.bottom - rect.top) as isize - h) / 2 - min_y as isize;
    if dx == 0 && dy == 0 {
        return;
    }
    let src = coverage.to_vec();
    coverage.fill(0);
    for y in 0..s {
        for x in 0..s {
            let (nx, ny) = (x as isize + dx, y as isize + dy);
            if (0..s as isize).contains(&nx) && (0..s as isize).contains(&ny) {
                coverage[ny as usize * s + nx as usize] = src[y * s + x];
            }
        }
    }
}

fn base_data(hwnd: HWND, id: u32) -> NOTIFYICONDATAW {
    let mut d: NOTIFYICONDATAW = unsafe { zeroed() };
    d.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    d.hWnd = hwnd;
    d.uID = id;
    d
}

/// トレイにアイコンを登録する。Explorer 再起動時（TaskbarCreated）にも呼ぶ。
pub fn add(hwnd: HWND, id: u32, icon: HICON, tip: &str) -> bool {
    unsafe {
        let mut d = base_data(hwnd, id);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        d.uCallbackMessage = WM_TRAYICON;
        d.hIcon = icon;
        copy_wide(&mut d.szTip, tip);
        Shell_NotifyIconW(NIM_DELETE, &d); // 前回の異常終了で残っていた分を消す
        if Shell_NotifyIconW(NIM_ADD, &d) == 0 {
            return false;
        }
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &d) != 0
    }
}

/// アイコンとツールチップのうち、渡されたものだけを更新する。
///
/// NIF_SHOWTIP は毎回付ける。NOTIFYICON_VERSION_4 では標準のツールチップは既定で抑止されていて、
/// Explorer はこのフラグを NIM_MODIFY ごとの状態として扱うため、アイコンだけの更新で省くと
/// その後ツールチップが出なくなる。
pub fn update(hwnd: HWND, id: u32, icon: Option<HICON>, tip: Option<&str>) {
    unsafe {
        let mut d = base_data(hwnd, id);
        d.uFlags = NIF_SHOWTIP;
        if let Some(icon) = icon {
            d.uFlags |= NIF_ICON;
            d.hIcon = icon;
        }
        if let Some(tip) = tip {
            d.uFlags |= NIF_TIP;
            copy_wide(&mut d.szTip, tip);
        }
        if d.uFlags != NIF_SHOWTIP {
            Shell_NotifyIconW(NIM_MODIFY, &d);
        }
    }
}

/// 通知（バルーン）を出す。NIF_SHOWTIP は update と同じ理由で付ける。
pub fn notify(hwnd: HWND, id: u32, title: &str, text: &str, warning: bool) {
    unsafe {
        let mut d = base_data(hwnd, id);
        d.uFlags = NIF_INFO | NIF_SHOWTIP;
        d.dwInfoFlags = if warning { NIIF_WARNING } else { NIIF_INFO };
        copy_wide(&mut d.szInfoTitle, title);
        copy_wide(&mut d.szInfo, text);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}

pub fn remove(hwnd: HWND, id: u32) {
    unsafe {
        let d = base_data(hwnd, id);
        Shell_NotifyIconW(NIM_DELETE, &d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn 使用率を文字にする() {
        assert_eq!(label(Some(0)), "0");
        assert_eq!(label(Some(7)), "7");
        assert_eq!(label(Some(45)), "45");
        assert_eq!(label(Some(100)), "100");
        assert_eq!(label(Some(150)), "100"); // 上限で丸める
        assert_eq!(label(None), "--");
    }

    #[test]
    fn テーマと使用率で色を選ぶ() {
        let black = Color { r: 0, g: 0, b: 0 };
        let white = Color { r: 255, g: 255, b: 255 };
        assert_eq!(color_for(Some(10), true), black);
        assert_eq!(color_for(Some(10), false), white);
        assert_eq!(color_for(None, true), black); // 不明はテーマ色のまま（呼び出し側で薄くする）
        assert_eq!(color_for(None, false), white);
        assert_eq!(color_for(Some(WARN_PERCENT - 1), false), white);
        assert_eq!(color_for(Some(WARN_PERCENT), false), WARN_COLOR);
        assert_eq!(color_for(Some(WARN_PERCENT), true), WARN_COLOR); // 警告色はテーマに依らない
        assert_eq!(color_for(Some(100), true), WARN_COLOR);
    }

    #[test]
    fn 長方形の被覆率() {
        assert_eq!(rect_cov(3, 3, 0.0, 0.0, 8.0, 8.0), 1.0); // 完全に内側
        assert_eq!(rect_cov(8, 3, 0.0, 0.0, 8.0, 8.0), 0.0); // 境界の外
        assert_eq!(rect_cov(7, 3, 0.0, 0.0, 7.5, 8.0), 0.5); // 横半分
        assert_eq!(rect_cov(2, 2, 2.5, 2.5, 3.0, 3.0), 0.25); // 4 分の 1
    }

    #[test]
    fn 角丸の被覆率() {
        // 大きい角丸: 内側と辺の中央は塗られ、四隅は抜け、角の途中は中間値
        let big = |x, y| rrect_cov(x, y, 0.0, 0.0, 16.0, 16.0, 4.0);
        assert!(close(big(8, 8), 1.0));
        assert!(close(big(0, 8), 1.0));
        assert!(close(big(8, 0), 1.0));
        assert!(close(big(0, 0), 0.0));
        assert!(close(big(15, 15), 0.0));
        let edge = big(1, 1);
        assert!(edge > 0.0 && edge < 1.0, "実際の値: {}", edge);

        // 半径が小さくても内側は完全に塗られる（内側の距離を見ていない実装だと半透明になる）
        for r in [0.0, 0.28, 1.28, 4.0] {
            assert!(close(rrect_cov(8, 8, 3.0, 3.0, 13.0, 13.0, r), 1.0), "r = {}", r);
            assert!(close(rrect_cov(3, 8, 3.0, 3.0, 13.0, 13.0, r), 1.0), "r = {} の左辺", r);
            assert!(close(rrect_cov(2, 8, 3.0, 3.0, 13.0, 13.0, r), 0.0), "r = {} の外", r);
        }
        assert!(close(rrect_cov(3, 3, 3.0, 3.0, 13.0, 13.0, 0.0), 1.0)); // 半径 0 なら角も四角いまま
        assert!(close(rrect_cov(13, 5, 3.0, 3.0, 13.5, 13.0, 0.0), 0.5)); // 半端な縁は半分
    }

    #[test]
    fn チップの寸法はピクセル境界に揃う() {
        for size in [16, 24, 32] {
            let c = Chip::new(size);
            for v in [c.body.0, c.body.2, c.inner.0, c.inner.2, c.t] {
                assert_eq!(v.fract(), 0.0, "size {} の値 {} が整数でない", size, v);
            }
            assert!(c.inner.2 - c.inner.0 >= 12.0, "size {} の内側が狭すぎる", size);
            // ピンは各辺 3 本で、左右対称
            let s = size as f32;
            let first = (0..size).find(|&x| c.pins_cov(x, 0, size) > 0.0).unwrap() as f32;
            let last = (0..size).rev().find(|&x| c.pins_cov(x, 0, size) > 0.0).unwrap() as f32 + 1.0;
            assert!((first - (s - last)).abs() <= 1.0, "size {} のピンが偏っている", size);
        }
        // 16px: ピン 1、枠 1、内側は [2,14)
        let c = Chip::new(16);
        assert_eq!(c.body, (1.0, 1.0, 15.0, 15.0));
        assert_eq!(c.inner, (2.0, 2.0, 14.0, 14.0));
        assert!(close(c.frame_cov(4, 0, 16), 1.0)); // 上のピン（辺の 1/4）
        assert!(close(c.frame_cov(8, 0, 16), 1.0)); // 中央
        assert!(close(c.frame_cov(11, 0, 16), 1.0)); // 3/4
        assert!(close(c.frame_cov(5, 0, 16), 0.0)); // ピンの隙間
        assert!(close(c.frame_cov(0, 4, 16), 1.0)); // 左のピン
        assert!(close(c.frame_cov(1, 8, 16), 1.0)); // 枠
        assert!(close(c.frame_cov(8, 8, 16), 0.0)); // 内側は枠でない
        assert!(close(c.inner_cov(8, 8), 1.0));
    }

    #[test]
    fn メモリモジュールの端子は左右対称に並ぶ() {
        for size in [16, 24, 32] {
            let m = Module::new(size);
            let total = (2 * m.teeth - 1) as f32 * m.tooth_w;
            let right_margin = size as f32 - (m.teeth_start + total);
            assert!((m.teeth_start - right_margin).abs() <= 1.0, "size {} の端子が偏っている", size);
            assert!(m.teeth >= 5, "size {} の端子が少なすぎる: {}", size, m.teeth);
            assert_eq!(m.teeth_start.fract(), 0.0);
        }
        let m = Module::new(16);
        assert_eq!((m.teeth, m.tooth_w, m.teeth_start), (7, 1.0, 1.0));
        assert_eq!(m.teeth_cov(1, 15, 16), 1.0); // 端子
        assert_eq!(m.teeth_cov(2, 15, 16), 0.0); // 隙間
        assert_eq!(m.teeth_cov(1, 5, 16), 0.0); // 本体の高さには端子はない
        assert!(close(m.body_cov(8, 5), 1.0));
    }

    #[test]
    fn 描いた部分を枠の中央に寄せる() {
        // 4x4 の左上 1 px に描いたものは中央付近 (1,1) に動く
        let mut c = vec![0u8; 16];
        c[0] = 200;
        center_in(&mut c, 4, &RECT { left: 0, top: 0, right: 4, bottom: 4 });
        assert_eq!(c.iter().filter(|&&v| v != 0).count(), 1);
        assert_eq!(c[1 * 4 + 1], 200);

        // 枠が右下 2x2 なら、その中央 (2,2) に動く
        let mut c = vec![0u8; 16];
        c[0] = 200;
        center_in(&mut c, 4, &RECT { left: 2, top: 2, right: 4, bottom: 4 });
        assert_eq!(c[2 * 4 + 2], 200);

        // すでに中央なら変わらない（2x2 の塊が 4x4 の中央）
        let mut c = vec![0u8; 16];
        for (x, y) in [(1, 1), (2, 1), (1, 2), (2, 2)] {
            c[y * 4 + x] = 255;
        }
        let before = c.clone();
        center_in(&mut c, 4, &RECT { left: 0, top: 0, right: 4, bottom: 4 });
        assert_eq!(c, before);

        // 何も描かれていなければそのまま
        let mut c = vec![0u8; 16];
        center_in(&mut c, 4, &RECT { left: 0, top: 0, right: 4, bottom: 4 });
        assert!(c.iter().all(|&v| v == 0));
    }

    #[test]
    fn 数字の置き場はアイコンの中に収まる() {
        for style in [Style::Chip, Style::Module] {
            for size in [16, 24, 32] {
                let b = text_box(style, size);
                assert!(b.em > 0 && b.max_width > 0, "{:?} {}", style, size);
                assert!(b.rect.left >= 0 && b.rect.top >= 0 && b.rect.right <= size && b.rect.bottom <= size, "{:?} {}", style, size);
                assert!(b.rect.right - b.rect.left >= b.max_width, "{:?} {}", style, size);
                assert!(b.rect.bottom > b.rect.top, "{:?} {}", style, size);
            }
        }
    }

    #[test]
    fn アルファは範囲内() {
        for style in [Style::Chip, Style::Module] {
            for size in [16, 24, 32] {
                for text in [0.0, 0.5, 1.0] {
                    for y in 0..size {
                        for x in 0..size {
                            let a = pixel_alpha(style, x, y, size, text);
                            assert!((0.0..=1.0001).contains(&a), "{:?} size {} ({}, {}) = {}", style, size, x, y, a);
                        }
                    }
                }
            }
        }
    }
}
