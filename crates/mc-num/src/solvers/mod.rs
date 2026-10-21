//! حلگرهای عددی — fzero (Brent) + integral (Gauss–Kronrod G7/K15 تطبیقی) + ODE.
//!
//! ساختار:
//! - `fzero`: روش Brent (دوبخشی + امتیانی + درجه‌۲ وارون) با جست‌وجوی براکت خودکار
//!   (سند ۰۴ §5.1).
//! - `integral`: G7/K15 تطبیقی با صف اولویت (بزرگ‌ترین خطا اول) و بودجه ۱۰٬۰۰۰
//!   فراخوانی تابع (سند ۰۴ §5.3) + تبدیل متغیر برای بازه‌های بی‌نهایت.
//! - `ode45/ode23`: Dormand–Prince 5(4) و Bogacki–Shampine 3(2) تطبیقی با FSAL
//!   (سند ۰۴ §4) + cancel همکاری‌گر (سند ۰۲ §3).

pub mod integral;
pub mod ode;
pub mod root;

pub use integral::{bounds_from, integrate, Bounds, QuadResult};
pub use ode::{ode23, ode45, OdeOpts, OdeResult};
pub use root::{fzero_bracket, fzero_scalar, BrentResult};
