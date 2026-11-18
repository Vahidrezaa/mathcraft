//! بودجهٔ عملکرد گام 3.6 (سند ۰۷ §P3 + ROADMAP §3.6):
//! «۳ سری × ۱۰۰ هزار نقطه → آماده‌سازی موتور ≤ ۵۰ms» — fold کل دنبالهٔ ops،
//! decimation LTTB هر سری به ≤ [`mc_plot::lttb::THRESHOLD`] و ساخت snapshot
//! انتقالی، همه یکجا. دادهٔ کامل هرگز از Rust بیرون نمی‌رود (سند ۰۲ §7) —
//! این تست همان مرز قرارداد را می‌سنجد، نه ترسیم Dart (آن سمت در
//! app/test/golden_plots_test.dart با بودجهٔ خودش پوشش داده شده است).

use mc_plot::figure::{CellSnapshot, FigureState};
use mc_plot::lttb::THRESHOLD;
use mc_plot::spec::{PlotOp, SeriesData, SeriesPair, StyleSpec};
use std::time::Instant;

/// پروفایل سه‌سریِ سند ۰۷ §P3 — جنس داده با فاز متفاوت برای هر سری تا
/// LTTB ناچار سه مسیر مستقل کامل را طی کند (نه میان‌بر یکسان).
fn series(k: usize, n: usize) -> SeriesPair {
    let x: Vec<f64> = (0..n).map(|i| i as f64 * 0.001).collect();
    let phase = 0.37 + 0.11 * k as f64;
    let y: Vec<f64> = (0..n).map(|i| ((i as f64) * phase).sin()).collect();
    SeriesPair {
        data: SeriesData { x, y },
        style: StyleSpec::default(),
    }
}

#[test]
fn three_series_100k_fold_decimate_snapshot_within_50ms() {
    const N: usize = 100_000;

    // fold — ارزان و خطی؛ اما بخشی از «آماده‌سازی تا اولین فریم» است
    let mut state = FigureState::new();
    state.apply(PlotOp::Line {
        series: vec![series(0, N), series(1, N), series(2, N)],
    });

    // مسیر انتقال: snapshot = decimation LTTB + تیک + هندسه محور
    let t0 = Instant::now();
    let snaps = state.snapshots();
    let elapsed = t0.elapsed();

    assert_eq!(snaps.len(), 1, "یک شکل ساخته شود");
    let CellSnapshot::Line { series, .. } = &snaps[0].spec else {
        panic!("سلول Line انتظار می‌رود");
    };
    assert_eq!(series.len(), 3, "سه سری باید همگی منتقل شوند");
    for s in series {
        assert_eq!(s.raw_len as usize, N, "طول خام در snapshot حفظ می‌شود");
        assert!(
            s.x.len() <= THRESHOLD,
            "سری باید به ≤ {} نقطه کاسته شود، بود {}",
            THRESHOLD,
            s.x.len()
        );
        assert_eq!(s.x.len(), s.y.len());
    }

    assert!(
        elapsed.as_millis() <= 50,
        "آماده‌سازی موتور {:.1?}ms از بودجهٔ ۵۰ms سند ۰۷ §P3 فراتر رفت",
        elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
fn raw_series_stays_full_for_redecimation() {
    // قرارداد: zoom re-decimation از دادهٔ خام می‌کشد (raw_len را دیدیم)؛
    // اینجا خود مسیر series_raw را با یک سری ۱۰۰هزار تایی می‌سنجیم.
    const N: usize = 100_000;
    let mut state = FigureState::new();
    state.apply(PlotOp::Line {
        series: vec![series(3, N)],
    });
    let snaps = state.snapshots();
    let (figure_id, series_id) = match &snaps[0].spec {
        CellSnapshot::Line { series, .. } => (snaps[0].figure_id, series[0].id),
        _ => panic!("سلول Line انتظار می‌رود"),
    };
    let raw = state
        // سلول پیش‌فرض پلات بدون subplot = اندیس ۱ (قرارداد یک‌مبنای subplot)
        .series_raw(figure_id, 1, series_id)
        .expect("سری خام باید موجود باشد");
    assert_eq!(raw.0.len(), N);
    assert_eq!(raw.1.len(), N);
}
