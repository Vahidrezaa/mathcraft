//! پالت رنگ نمودار — توکن‌های معنایی سند ۰۶ بخش ۳ و رنگ‌های نامی متلب.
//!
//! دو لایهٔ رنگ داریم (قرارداد v2 — بدون بازنویسی در حرفه‌ای‌شدن):
//! - [`PALETTE`] ‏۸ توکن معنایی CDS است که **در Dart** به هگز resolve می‌شود؛
//!   پالت‌ایندکس یعنی «رنگ سری بعدی طبق تم» — با تغییر تم، نمودار بدون
//!   بازتولید موتور بازرنگ می‌شود.
//! - [`MATLAB_NAMED`] رنگ‌های نامی متلب (`'r'`, `'g'`, …) با مقدار **دقیق**
//!   RGB‌اند — پارتی متلب، نه توکن تم.

/// تعداد رنگ‌های پالت دسته‌بندی (سند ۰۶ بخش ۳) — سیکل ایندکس پالت.
pub const PALETTE_LEN: u8 = 8;

/// پالت ۸ رنگی دسته‌بندی (سند ۰۶ بخش ۳؛ deuteranopia-safe ترتیبی) —
/// مقدار مرجع برای تست/مستند؛ resolve هگز در سمت Dart انجام می‌شود.
/// ترتیب: cyan, amber, violet, emerald, pink, blue, amber-2, red.
pub const PALETTE: [(u8, u8, u8); PALETTE_LEN as usize] = [
    (0x22, 0xD3, 0xEE), // #22D3EE
    (0xF5, 0xA5, 0x24), // #F5A524
    (0xA7, 0x8B, 0xFA), // #A78BFA
    (0x34, 0xD3, 0x99), // #34D399
    (0xF4, 0x72, 0xB6), // #F472B6
    (0x60, 0xA5, 0xFA), // #60A5FA
    (0xFB, 0xBF, 0x24), // #FBBF24
    (0xF8, 0x71, 0x71), // #F87171
];

/// ایندکس بعدی سیکل پالت (برای سری بدون رنگ صریح — رفتار متلب:
/// `plot(x,y)` به ترتیب رنگ‌ها پیش می‌رود).
pub fn cycle(index: u64) -> u8 {
    (index % u64::from(PALETTE_LEN)) as u8
}

/// رنگ‌های نامی تک‌حرفی متلب در LineSpec — مقدار دقیق (پارتی متلب، سند ۰۴ §8.1).
pub const MATLAB_NAMED: [(&str, (u8, u8, u8)); 8] = [
    ("r", (255, 0, 0)),
    ("g", (0, 255, 0)),
    ("b", (0, 0, 255)),
    ("c", (0, 255, 255)),
    ("m", (255, 0, 255)),
    ("y", (255, 255, 0)),
    ("k", (0, 0, 0)),
    ("w", (255, 255, 255)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_doc06_tokens() {
        assert_eq!(PALETTE[0], (0x22, 0xD3, 0xEE));
        assert_eq!(PALETTE[7], (0xF8, 0x71, 0x71));
        assert_eq!(PALETTE.len(), 8);
    }

    #[test]
    fn palette_cycle_wraps_at_eight() {
        assert_eq!(cycle(0), 0);
        assert_eq!(cycle(7), 7);
        assert_eq!(cycle(8), 0);
        assert_eq!(cycle(19), 3);
    }

    #[test]
    fn matlab_named_covers_the_eight_short_colors() {
        assert_eq!(MATLAB_NAMED[0], ("r", (255, 0, 0)));
        assert_eq!(MATLAB_NAMED[6], ("k", (0, 0, 0)));
        assert_eq!(MATLAB_NAMED.len(), 8);
    }
}
