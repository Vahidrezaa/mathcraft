//! تست‌های توابع رسم گزاره‌ای (ROADMAP 3.2 — P3-a).
//!
//! سطح تست = دنبالهٔ `EvalOutcome.plot_ops` (گرام عملیات)؛ کاهشِ حالت شکل‌ها
//! در mc-plot/figure.rs جداگانه تست می‌شود. واگرایی‌های MATLAB-lite در
//! COMPATIBILITY ثبت شده‌اند (یک استایل در هر فراخوانی، بدون toggle-hold،
//! fplot با ۴۰۰ نمونهٔ ثابت، …).

use mc_interp::{eval_program, Environment};
use mc_parser::parse;
use mc_plot::figure::CellSnapshot;
use mc_plot::spec::{
    DashSpec, FillPair, HistNorm, LegendPos, MarkerSpec, PlotColorSpec, PlotOp, SeriesPair,
};

fn run(env: &mut Environment, src: &str) -> mc_interp::EvalOutcome {
    match parse(src) {
        Ok(p) => eval_program(&p, env),
        Err(e) => mc_interp::EvalOutcome {
            error: Some(e),
            ..Default::default()
        },
    }
}

/// خطای اول + متن EN برای پیام‌ها.
fn err_of(src: &str) -> mc_core::McError {
    let mut env = Environment::new();
    let out = run(&mut env, src);
    out.error
        .unwrap_or_else(|| panic!("expected error for: {src}"))
}

fn first_op(src: &str) -> PlotOp {
    let mut env = Environment::new();
    let out = run(&mut env, src);
    assert!(out.error.is_none(), "unexpected error: {:?}", out.error);
    out.plot_ops
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no plot op emitted for: {src}"))
}

#[test]
fn plot_y_vector_gets_default_x() {
    match first_op("plot([3 1 4 1])") {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 1);
            assert_eq!(series[0].data.x, vec![1.0, 2.0, 3.0, 4.0]);
            assert_eq!(series[0].data.y, vec![3.0, 1.0, 4.0, 1.0]);
            assert!(series[0].style.color.is_none()); // رنگ = resolve کاهش‌دهنده
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_xy_explicit() {
    match first_op("plot([1 2 3], [10 20 30])") {
        PlotOp::Line { series } => {
            assert_eq!(series[0].data.x, vec![1.0, 2.0, 3.0]);
            assert_eq!(series[0].data.y, vec![10.0, 20.0, 30.0]);
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_matrix_y_is_one_call_with_column_series() {
    let mut env = Environment::new();
    let out = run(&mut env, "plot([1 2 3], [1 4; 2 5; 3 6])");
    assert!(out.error.is_none());
    assert_eq!(out.plot_ops.len(), 1);
    match &out.plot_ops[0] {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 2); // دو ستون = دو سری در «یک» فراخوانی
            assert_eq!(series[0].data.y, vec![1.0, 2.0, 3.0]);
            assert_eq!(series[1].data.y, vec![4.0, 5.0, 6.0]);
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_length_mismatch_is_positioned_error() {
    let e = err_of("x = 1:3;\ny = 1:4;\nplot(x, y)");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR");
    assert_eq!(e.line, 3);
    assert!(e.col >= 1);
    assert!(e.message_en.contains("lengths must match"));
}

#[test]
fn plot_linespec_r_dashed_is_exact_red() {
    match first_op("plot(1:10, 'r--')") {
        PlotOp::Line { series } => {
            let s = &series[0].style;
            assert_eq!(s.color, Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 }));
            assert_eq!(s.dash, Some(DashSpec::Dashed));
            assert_eq!(s.marker, None);
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_b_star_is_marker_only_invisible_line() {
    match first_op("plot(1:10, 'b*')") {
        PlotOp::Line { series } => {
            let s = &series[0].style;
            assert_eq!(s.marker, Some(MarkerSpec::Star));
            assert_eq!(s.dash, Some(DashSpec::Invisible));
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_name_value_linewidth_and_color() {
    match first_op("plot(1:10, 'LineWidth', 4, 'Color', 'red')") {
        PlotOp::Line { series } => {
            assert_eq!(series[0].style.width, Some(4.0));
            assert_eq!(
                series[0].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // سه‌تایی متلب 0..1 و پذیرش 0..255
    match first_op("plot(1:10, 'Color', [1 0 0])") {
        PlotOp::Line { series } => assert_eq!(
            series[0].style.color,
            Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
        ),
        other => panic!("expected Line, got {other:?}"),
    }
    match first_op("plot(1:10, 'Color', [128 64 0])") {
        PlotOp::Line { series } => assert_eq!(
            series[0].style.color,
            Some(PlotColorSpec::Rgb {
                r: 128,
                g: 64,
                b: 0
            })
        ),
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_displayname_is_rejected_with_message() {
    let e = err_of("plot(1:10, 'DisplayName', 's')");
    assert!(e.message_en.contains("DisplayName"));
}

#[test]
fn plot_unknown_style_string_is_rejected() {
    let e = err_of("plot(1:10, 'q')");
    assert!(e.message_en.contains("LineSpec"));
}

#[test]
fn scatter_defaults_to_circle_marker() {
    // سطح op: استایل خالی — پیش‌فرض‌ها (دایره/خط نامرئی) در کاهش‌دهندهٔ
    // FigureState اعمال می‌شوند (تست جدا در mc-plot/figure.rs)
    match first_op("scatter([1 2], [3 4])") {
        PlotOp::Scatter { series } => {
            assert_eq!(series[0].style.marker, None);
        }
        other => panic!("expected Scatter, got {other:?}"),
    }
    // استایل صریح مارکر را بازنویسی می‌کند
    match first_op("scatter([1 2], [3 4], 'k+')") {
        PlotOp::Scatter { series } => {
            assert_eq!(series[0].style.marker, Some(MarkerSpec::Plus));
        }
        other => panic!("expected Scatter, got {other:?}"),
    }
}

#[test]
fn bar_width_positional_and_stacked_flag() {
    match first_op("bar([1 2 3], [4 5 6], 0.5, 'stacked')") {
        PlotOp::Bar {
            series,
            width,
            stacked,
            horizontal,
        } => {
            assert_eq!(series.len(), 1);
            assert_eq!(width, Some(0.5));
            assert!(stacked);
            assert!(!horizontal);
        }
        other => panic!("expected Bar, got {other:?}"),
    }
    // عرض خارج بازه رد می‌شود
    let e = err_of("bar([1 2], [3 4], 1.7)");
    assert!(e.message_en.contains("width"));
}

#[test]
fn bar_matrix_groups_columns_in_one_call() {
    let mut env = Environment::new();
    let out = run(&mut env, "bar([1 2 3], [1 2 3; 4 5 6; 7 8 9])");
    match &out.plot_ops[0] {
        PlotOp::Bar { series, .. } => assert_eq!(series.len(), 3),
        other => panic!("expected Bar, got {other:?}"),
    }
}

#[test]
fn histogram_default_ten_bins_cover_all_data() {
    match first_op("histogram(0:99)") {
        PlotOp::Histogram {
            edges,
            counts,
            norm,
            ..
        } => {
            assert_eq!(edges.len(), 11);
            let sum: f64 = counts.iter().sum();
            assert_eq!(sum, 100.0);
            assert_eq!(norm, HistNorm::Count);
        }
        other => panic!("expected Histogram, got {other:?}"),
    }
}

#[test]
fn histogram_nbins_edges_and_normalization() {
    match first_op("histogram(0:99, 5, 'Normalization', 'cdf')") {
        PlotOp::Histogram {
            edges,
            counts,
            norm,
            ..
        } => {
            assert_eq!(edges.len(), 6);
            assert_eq!(norm, HistNorm::Cdf);
            let last = counts.last().copied().unwrap_or(0.0);
            assert!((last - 1.0).abs() < 1e-9, "cdf must end at 1, got {last}");
        }
        other => panic!("expected Histogram, got {other:?}"),
    }
    match first_op("histogram(rand(100,1), [0 0.5 1])") {
        PlotOp::Histogram { edges, .. } => assert_eq!(edges, vec![0.0, 0.5, 1.0]),
        other => panic!("expected Histogram, got {other:?}"),
    }
    // یال‌های ناصعودی رد
    let e = err_of("histogram(1:10, [3 2 1])");
    assert!(e.message_en.contains("increasing"));
    // نرمال‌سازی ناشناخته رد
    let e = err_of("histogram(1:10, 'Normalization', 'weird')");
    assert!(e.message_en.contains("normalization"));
}

#[test]
fn area_carries_zero_baseline() {
    match first_op("area([1 2 3], [2 4 6])") {
        PlotOp::Area { series, baseline } => {
            assert_eq!(series.len(), 1);
            assert_eq!(baseline, 0.0);
        }
        other => panic!("expected Area, got {other:?}"),
    }
}

#[test]
fn fill_pairs_with_trailing_color() {
    let mut env = Environment::new();
    let out = run(&mut env, "fill([0 1 1], [0 0 1], [2 3 2], [1 1 0], 'r')");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert_eq!(out.plot_ops.len(), 1);
    match &out.plot_ops[0] {
        PlotOp::Fill { polys } => {
            assert_eq!(polys.len(), 2);
            // رنگ فقط چندضلعی دوم را می‌گیرد؛ اولی پیش‌فرض (سیکل پالت)
            assert!(polys[0].style.color.is_none());
            assert_eq!(
                polys[1].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
        }
        other => panic!("expected Fill, got {other:?}"),
    }
    // رنگ سه‌تایی نامعتبر (طول ۲) = خطا
    assert!(err_of("fill([0 1], [0 1], [5 6])")
        .message_en
        .contains("color"));
    // یک عدد تنها / سه‌تایی عددی = خطای روشن
    assert!(err_of("fill([0 1 2])").message_en.contains("pairs"));
    assert!(err_of("fill([0 1], [0 1], [1 0 0])")
        .message_en
        .contains("pairs"));
    // زوج ناهم‌طول = خطا
    assert!(err_of("fill([0 1], [0 1 2])").message_en.contains("equal"));
}

#[test]
fn polarplot_emits_polar_op() {
    match first_op("t = 0:0.1:6.28; polarplot(t, sin(2*t))") {
        PlotOp::Polar { series } => {
            assert_eq!(series.len(), 1);
            assert!((series[0].data.x[0] - 0.0).abs() < 1e-9);
        }
        other => panic!("expected Polar, got {other:?}"),
    }
}

#[test]
fn errorbar_forms() {
    // متقارن: (y, e)
    match first_op("errorbar([1 2 3], [0.1 0.2 0.3])") {
        PlotOp::ErrorBar { series } => {
            assert!(series[0].y_err.is_some());
            assert!(series[0].x_err.is_none());
        }
        other => panic!("expected ErrorBar, got {other:?}"),
    }
    // (x, y, yneg, ypos)
    match first_op("errorbar([1 2 3], [1 2 3], [0.1 0.1 0.1], [0.2 0.2 0.2])") {
        PlotOp::ErrorBar { series } => {
            let e = series[0].y_err.clone().unwrap_or_default();
            assert_eq!(e.neg, vec![0.1; 3]);
            assert_eq!(e.pos, vec![0.2; 3]);
        }
        other => panic!("expected ErrorBar, got {other:?}"),
    }
    // شش‌تایی کامل
    match first_op(
        "errorbar([1 2 3], [1 2 3], [0.1 0.1 0.1], [0.1 0.1 0.1], [0.2 0.2 0.2], [0.2 0.2 0.2])",
    ) {
        PlotOp::ErrorBar { series } => {
            assert!(series[0].x_err.is_some());
            assert!(series[0].y_err.is_some());
        }
        other => panic!("expected ErrorBar, got {other:?}"),
    }
    // طول ناهماهنگ e رد می‌شود (ناهماهنگی x/y در xy_args می‌گیرد)
    assert!(err_of("errorbar([1 2 3], [10 20 30], [1 2])")
        .message_en
        .contains("match the data length"));
}

#[test]
fn fplot_samples_domain_with_400_points() {
    match first_op("fplot(@(x) sin(x), [0 2*pi])") {
        PlotOp::Line { series } => {
            assert_eq!(series[0].data.x.len(), 400);
            assert!((series[0].data.x[0] - 0.0).abs() < 1e-9);
            assert!((series[0].data.x[399] - 2.0 * std::f64::consts::PI).abs() < 1e-9);
            // نمونه‌ها دقیقاً روی sin همان xها (نه فرض π/2 برای ایندکس دلخواه)
            for k in [0, 7, 100, 217, 399] {
                let xk = series[0].data.x[k];
                assert!(
                    (series[0].data.y[k] - xk.sin()).abs() < 1e-12,
                    "sample {k}: x={xk}"
                );
            }
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // دامنهٔ پیش‌فرض [-5, 5]
    match first_op("fplot(@(x) x)") {
        PlotOp::Line { series } => {
            assert!((series[0].data.x[0] + 5.0).abs() < 1e-9);
            assert!((series[0].data.x[399] - 5.0).abs() < 1e-9);
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // بدون هندل رد
    assert!(err_of("fplot(3)").message_en.contains("function handle"));
    // دامنهٔ نزولی رد
    assert!(err_of("fplot(@(x) x, [2 1])")
        .message_en
        .contains("increasing"));
}

#[test]
fn subplot_validates_grid_and_index() {
    match first_op("subplot(2, 3, 4)") {
        PlotOp::SubplotSelect { rows, cols, index } => {
            assert_eq!((rows, cols, index), (2, 3, 4));
        }
        other => panic!("expected SubplotSelect, got {other:?}"),
    }
    assert!(err_of("subplot(2, 2, 5)").message_en.contains("p must be"));
    assert!(err_of("subplot(0, 2, 1)").message_en.contains("positive"));
    assert!(err_of("subplot(2.5, 2, 1)").message_en.contains("integers"));
}

#[test]
fn legend_entries_and_off() {
    match first_op("legend('sin', 'cos')") {
        PlotOp::Legend { entries, position } => {
            assert_eq!(entries, Some(vec!["sin".to_owned(), "cos".to_owned()]));
            assert_eq!(position, LegendPos::Best);
        }
        other => panic!("expected Legend, got {other:?}"),
    }
    match first_op("legend('off')") {
        PlotOp::Legend { entries, .. } => assert!(entries.is_none()),
        other => panic!("expected Legend, got {other:?}"),
    }
    assert!(err_of("legend()").message_en.contains("labels"));
    assert!(err_of("legend(3)").message_en.contains("string"));
}

#[test]
fn axis_modes_and_limits() {
    match first_op("axis([0 1 -2 2])") {
        PlotOp::AxisLimits {
            min_x,
            max_x,
            min_y,
            max_y,
        } => assert_eq!((min_x, max_x, min_y, max_y), (0.0, 1.0, -2.0, 2.0)),
        other => panic!("expected AxisLimits, got {other:?}"),
    }
    assert!(matches!(
        first_op("axis('equal')"),
        PlotOp::AxisEqual { on: true }
    ));
    assert!(matches!(first_op("axis('tight')"), PlotOp::AxisTight));
    assert!(matches!(
        first_op("axis('off')"),
        PlotOp::AxisVisible { on: false }
    ));
    // axis('auto') از 3.5: پاک‌شدن حدود دستی هر دو محور
    {
        let mut env = Environment::new();
        let out = run(&mut env, "axis('auto')");
        assert!(out.error.is_none(), "{:?}", out.error);
        assert!(matches!(out.plot_ops[0], PlotOp::XLim { limits: None }));
        assert!(matches!(out.plot_ops[1], PlotOp::YLim { limits: None }));
    }
    assert!(err_of("axis([1 0 0 1])").message_en.contains("xmin<xmax"));
    assert!(err_of("axis([0 1 2])").message_en.contains("ymin"));
}

#[test]
fn grid_and_hold_on_off() {
    assert!(matches!(first_op("grid('on')"), PlotOp::Grid { on: true }));
    assert!(matches!(
        first_op("grid('off')"),
        PlotOp::Grid { on: false }
    ));
    assert!(err_of("grid('maybe')").message_en.contains("'on' or 'off'"));
    assert!(matches!(first_op("hold('on')"), PlotOp::Hold { on: true }));
    // toggle بدون آرگومان پشتیبانی نمی‌شود (MATLAB-lite)
    // hold بدون آرگومان = toggle (از 3.5 — پارتی متلب)
    assert!(matches!(first_op("hold()"), PlotOp::HoldToggle));
}

#[test]
fn labels_and_title_take_single_string() {
    assert!(matches!(
        first_op("xlabel('time (s)')"),
        PlotOp::XLabel { text } if text == "time (s)"
    ));
    assert!(matches!(
        first_op("ylabel('amplitude')"),
        PlotOp::YLabel { text } if text == "amplitude"
    ));
    assert!(matches!(
        first_op("title('My Plot')"),
        PlotOp::Title { text } if text == "My Plot"
    ));
    assert!(err_of("title(3)").message_en.contains("string"));
    assert!(err_of("title('a', 'b')").message_en.contains("exactly one"));
}

#[test]
fn figure_new_and_select() {
    match first_op("figure(3)") {
        PlotOp::ActivateFigure { id } => assert_eq!(id, Some(3)),
        other => panic!("expected ActivateFigure, got {other:?}"),
    }
    match first_op("figure()") {
        PlotOp::ActivateFigure { id } => assert!(id.is_none()),
        other => panic!("expected ActivateFigure, got {other:?}"),
    }
    assert!(err_of("figure(0)").message_en.contains("1..=64"));
}

#[test]
fn plot_functions_are_statement_only() {
    // داخل انتساب → خطای رجیستری با پیام روشن
    let e = err_of("h = plot(1:3)");
    assert!(e.message_en.contains("statement-only"));
    // داخل عبارت مرکب
    let e = err_of("1 + plot(1:3)");
    assert!(e.message_en.contains("statement-only"));
    // گزارهٔ ساده بدون '; هم بی‌اکو و بدون ans است
    let mut env = Environment::new();
    let out = run(&mut env, "plot(1:3)");
    assert!(out.error.is_none());
    assert!(out.outputs.is_empty(), "no echo: {:?}", out.outputs);
    assert!(env.get("ans").is_none(), "ans must not be set");
}

#[test]
fn user_symbol_shadows_the_statement_path() {
    // متغیر هم‌نام تقدم دارد (الگوی disp) — فراخوانی به مسیر عادی می‌رود
    let mut env = Environment::new();
    let out = run(&mut env, "plot = 3; plot(1:3)");
    assert!(out.error.is_some());
    assert!(out.plot_ops.is_empty());
}

#[test]
fn ops_survive_until_error_and_do_not_leak_between_runs() {
    let mut env = Environment::new();
    let out = run(&mut env, "plot(1:3); this_is_undefined()");
    assert!(out.error.is_some());
    assert_eq!(out.plot_ops.len(), 1, "plots before the error are kept");
    // اجرای بعدی از صفر شروع می‌شود (سینک تخلیه شده)
    let out = run(&mut env, "1 + 1");
    assert!(out.error.is_none());
    assert!(out.plot_ops.is_empty());
}

#[test]
fn hold_then_two_plots_emit_three_ops_in_order() {
    let mut env = Environment::new();
    let out = run(&mut env, "hold('on'); plot(1:3); plot(1:3, 'r')");
    assert_eq!(out.plot_ops.len(), 3);
    assert!(matches!(out.plot_ops[0], PlotOp::Hold { on: true }));
    assert!(matches!(out.plot_ops[1], PlotOp::Line { .. }));
    assert!(matches!(out.plot_ops[2], PlotOp::Line { .. }));
}

#[test]
fn ops_from_inside_user_functions_reach_the_outcome() {
    let mut env = Environment::new();
    let out = run(&mut env, "function f(); plot(1:3); end; f();");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert_eq!(out.plot_ops.len(), 1);
}

#[test]
fn complex_all_real_is_plottable_mixed_is_not() {
    assert!(matches!(
        first_op("plot(complex(1:3, 0))"),
        PlotOp::Line { .. }
    ));
    let e = err_of("plot(complex(1:3, 1))");
    assert!(e.message_en.contains("complex"));
}

#[test]
fn blank_style_string_is_rejected() {
    // رشتهٔ خالی/فاصله استایل نیست — خطای روشن (متلب هم ورودی تهی را نمی‌پذیرد)
    let e = err_of("plot(1:10, ' ')");
    assert!(e.message_en.contains("unknown option"));
}

#[test]
fn plot_ops_field_exists_and_defaults_empty() {
    let mut env = Environment::new();
    let out = run(&mut env, "42");
    assert!(out.plot_ops.is_empty());
    let _pairs: Vec<SeriesPair> = Vec::new(); // نوع عمومی است
}

// ---------- گسترش 3.5: پوشش امکانات پایه‌ای ۲بعدی متلب ----------

#[test]
fn plot_multi_group_with_per_group_linespec() {
    let mut env = Environment::new();
    let out = run(&mut env, "plot([1 2], [1 4], 'r--', [3 4], [9 6], 'b:o')");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert_eq!(out.plot_ops.len(), 1);
    match &out.plot_ops[0] {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 2);
            // گروه ۱: قرمز خط‌چین
            assert_eq!(
                series[0].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
            assert_eq!(series[0].style.dash, Some(DashSpec::Dashed));
            // گروه ۲: آبی نقطه‌چین با مارکر دایره (فقط مارکر → خط نامرئی)
            assert_eq!(
                series[1].style.color,
                Some(PlotColorSpec::Rgb { r: 0, g: 0, b: 255 })
            );
            assert_eq!(series[1].style.dash, Some(DashSpec::Dotted));
            assert_eq!(series[1].style.marker, Some(MarkerSpec::Circle));
            assert_eq!(series[1].data.x, vec![3.0, 4.0]);
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_global_name_values_apply_to_all_groups() {
    let mut env = Environment::new();
    let out = run(
        &mut env,
        "plot([1 2], [1 4], 'r', [3 4], [9 6], 'LineWidth', 3)",
    );
    assert!(out.error.is_none(), "{:?}", out.error);
    match &out.plot_ops[0] {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 2);
            assert_eq!(series[0].style.width, Some(3.0));
            assert_eq!(series[1].style.width, Some(3.0));
        }
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn plot_name_values_marker_and_linestyle() {
    match first_op("plot([1 2 3], [1 2 1], 'LineStyle', '--', 'Marker', 's')") {
        PlotOp::Line { series } => {
            assert_eq!(series[0].style.dash, Some(DashSpec::Dashed));
            assert_eq!(series[0].style.marker, Some(MarkerSpec::Square));
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // MarkerSize صادقانه خطا می‌دهد (قرارداد v3)
    let msg = err_of("plot([1 2], [1 2], 'MarkerSize', 8)").message_en;
    assert!(msg.contains("MarkerSize"), "{msg}");
}

#[test]
fn semilog_family_sets_log_flags() {
    // پس از گام 3.5.2 ترتیب Line→SetLog است؛ SetLog دومی است (fresh_axes
    // در Line دیگر پرچم لگاریتمی همین فراخوانی را نمی‌کُشد).
    fn setlog_of(src: &str) -> PlotOp {
        let mut env = Environment::new();
        let out = run(&mut env, src);
        assert!(out.error.is_none(), "{:?}", out.error);
        assert!(
            matches!(out.plot_ops[0], PlotOp::Line { .. }),
            "ops: {:?}",
            out.plot_ops
        );
        out.plot_ops[1].clone()
    }
    match (
        setlog_of("semilogx([1 10 100], [1 2 3])"),
        setlog_of("semilogy([1 2 3], [1 10 100])"),
        setlog_of("loglog([1 10], [1 10])"),
    ) {
        (
            PlotOp::SetLog { x, y },
            PlotOp::SetLog { x: x2, y: y2 },
            PlotOp::SetLog { x: x3, y: y3 },
        ) => {
            assert!((x, y) == (true, false));
            assert!((x2, y2) == (false, true));
            assert!((x3, y3) == (true, true));
        }
        _ => panic!("expected SetLog ops"),
    }
    // semilogx با چند گروه هم کار می‌کند
    let mut env = Environment::new();
    let out = run(&mut env, "semilogx([1 10], [1 2], 'r', [1 10], [2 1], 'b')");
    assert!(out.error.is_none());
    assert!(matches!(out.plot_ops[0], PlotOp::Line { ref series } if series.len() == 2));
    assert!(matches!(out.plot_ops[1], PlotOp::SetLog { x: true, .. }));
}

#[test]
fn stem_synthesizes_risers_and_heads() {
    match first_op("stem([1 2 3], [2 -1 4], 'r')") {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 2);
            // رایزرها: زوج [x,0]→[x,y] با گپ NaN (مقایسهٔ NaN-aware)
            let nan_eq = |a: &Vec<f64>, b: &Vec<f64>| {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|(p, q)| p == q || (p.is_nan() && q.is_nan()))
            };
            assert!(nan_eq(
                &series[0].data.x,
                &vec![1.0, 1.0, f64::NAN, 2.0, 2.0, f64::NAN, 3.0, 3.0, f64::NAN]
            ));
            assert!(nan_eq(
                &series[0].data.y,
                &vec![0.0, 2.0, f64::NAN, 0.0, -1.0, f64::NAN, 0.0, 4.0, f64::NAN]
            ));
            assert_eq!(series[0].style.marker, None);
            assert_eq!(
                series[0].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
            // سرها: فقط مارکر (خط نامرئی) با دایرهٔ پیش‌فرض
            assert_eq!(series[1].style.dash, Some(DashSpec::Invisible));
            assert_eq!(series[1].style.marker, Some(MarkerSpec::Circle));
            assert_eq!(series[1].data.y, vec![2.0, -1.0, 4.0]);
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // 'filled' پذیرفته می‌شود (no-op — مارکر پر در قرارداد v3)
    {
        let mut env = Environment::new();
        let out = run(&mut env, "stem([1], [2], 'filled')");
        assert!(out.error.is_none(), "{:?}", out.error);
    }
}

#[test]
fn stairs_synthesizes_step_polyline() {
    match first_op("stairs([1 2 3], [5 6 7])") {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 1);
            assert_eq!(series[0].data.x, vec![1.0, 2.0, 2.0, 3.0, 3.0]);
            assert_eq!(series[0].data.y, vec![5.0, 5.0, 6.0, 6.0, 7.0]);
        }
        other => panic!("expected Line, got {other:?}"),
    }
    // فقط y هم مجاز (x = 1..n)
    match first_op("stairs([7 8 9])") {
        PlotOp::Line { series } => assert_eq!(series[0].data.x, vec![1.0, 2.0, 2.0, 3.0, 3.0]),
        other => panic!("expected Line, got {other:?}"),
    }
}

#[test]
fn barh_is_horizontal_bar() {
    match first_op("barh([3 1 2])") {
        PlotOp::Bar {
            horizontal, series, ..
        } => {
            assert!(horizontal);
            assert_eq!(series[0].data.y, vec![3.0, 1.0, 2.0]);
        }
        other => panic!("expected Bar, got {other:?}"),
    }
}

#[test]
fn xlim_ylim_vector_and_auto() {
    match first_op("xlim([0 5])") {
        PlotOp::XLim { limits } => assert_eq!(limits, Some([0.0, 5.0])),
        other => panic!("expected XLim, got {other:?}"),
    }
    match first_op("ylim('auto')") {
        PlotOp::YLim { limits } => assert!(limits.is_none()),
        other => panic!("expected YLim, got {other:?}"),
    }
    // جفت نامعتبر رد می‌شود
    assert!(err_of("xlim([5 0])").message_en.contains("increasing"));
    assert!(err_of("xlim([0])").message_en.contains("increasing"));
}

#[test]
fn axis_auto_clears_both() {
    let mut env = Environment::new();
    let out = run(&mut env, "axis('auto')");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert!(matches!(out.plot_ops[0], PlotOp::XLim { limits: None }));
    assert!(matches!(out.plot_ops[1], PlotOp::YLim { limits: None }));
}

#[test]
fn xticks_positions_and_labels() {
    match first_op("xticks([0 2 4])") {
        PlotOp::XTicks { ticks } => {
            assert_eq!(ticks.len(), 3);
            assert_eq!(ticks[0].position, 0.0);
            assert_eq!(ticks[0].label.as_deref(), Some("0"));
            assert_eq!(ticks[2].label.as_deref(), Some("4"));
        }
        other => panic!("expected XTicks, got {other:?}"),
    }
    // برچسب‌های سفارشی
    match first_op("yticks([1 2], 'lo', 'hi')") {
        PlotOp::YTicks { ticks } => {
            assert_eq!(ticks[0].label.as_deref(), Some("lo"));
            assert_eq!(ticks[1].label.as_deref(), Some("hi"));
        }
        other => panic!("expected YTicks, got {other:?}"),
    }
    // ناهم‌خوانی تعداد = خطا
    assert!(err_of("xticks([0 1], 'only-one')")
        .message_en
        .contains("counts must match"));
    // خالی = خودکار
    match first_op("xticks([])") {
        PlotOp::XTicks { ticks } => assert!(ticks.is_empty()),
        other => panic!("expected XTicks, got {other:?}"),
    }
}

#[test]
fn xticklabels_standalone_gives_actionable_error() {
    let msg = err_of("xticklabels('a')").message_en;
    assert!(msg.contains("xticks"), "{msg}");
}

#[test]
fn legend_location_name_value() {
    match first_op("legend('a', 'b', 'Location', 'SouthWest')") {
        PlotOp::Legend { entries, position } => {
            assert_eq!(entries, Some(vec!["a".to_owned(), "b".to_owned()]));
            assert_eq!(position, LegendPos::SouthWest);
        }
        other => panic!("expected Legend, got {other:?}"),
    }
    // case-insensitive
    match first_op("legend('a', 'Location', 'northeast')") {
        PlotOp::Legend { position, .. } => assert_eq!(position, LegendPos::NorthEast),
        other => panic!("expected Legend, got {other:?}"),
    }
    assert!(err_of("legend('a', 'Location', 'middle')")
        .message_en
        .contains("location"));
}

#[test]
fn hold_without_argument_toggles() {
    match first_op("hold()") {
        PlotOp::HoldToggle => {}
        other => panic!("expected HoldToggle, got {other:?}"),
    }
    // on/off صریح همان قبلی (فرم فراخوانی — سینتکس فرمانِ بدون پرانتز نیست)
    match first_op("hold('on')") {
        PlotOp::Hold { on } => assert!(on),
        other => panic!("expected Hold, got {other:?}"),
    }
}

#[test]
fn fill_pair_colors_per_polygon() {
    let mut env = Environment::new();
    let out = run(&mut env, "fill([0 1], [0 1], 'r', [2 3], [0 1], 'green')");
    assert!(out.error.is_none(), "{:?}", out.error);
    match &out.plot_ops[0] {
        PlotOp::Fill { polys } => {
            assert_eq!(polys.len(), 2);
            assert_eq!(
                polys[0].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
            assert_eq!(
                polys[1].style.color,
                Some(PlotColorSpec::Rgb { r: 0, g: 255, b: 0 })
            );
            let _: Vec<&FillPair> = polys.iter().collect();
        }
        other => panic!("expected Fill, got {other:?}"),
    }
}

// ---------- گام 3.5.1: سینتکس فرمانی متلب (باگ گزارش کاربر) ----------

/// دستور پخت دقیق کاربر (گزارش باگ ۲۰۲۶-۱۱-۱۸) — قبلاً روی `hold on`
/// به خطای رجیستری «statement-only» می‌خورد.
#[test]
fn user_recipe_three_lines_runs() {
    let src = concat!(
        "x = 0:0.1:2*pi;\n",
        "hold on; plot(x, sin(x), 'r--o', x, cos(x), 'b:s'); hold off\n",
        "legend('sin','cos','Location','SouthWest'); grid on\n",
    );
    let mut env = Environment::new();
    let out = run(&mut env, src);
    assert!(out.error.is_none(), "recipe failed: {:?}", out.error);
    let ops = &out.plot_ops;
    assert_eq!(ops.len(), 5, "ops: {ops:?}");
    assert!(matches!(ops[0], PlotOp::Hold { on: true }));
    match &ops[1] {
        PlotOp::Line { series } => {
            assert_eq!(series.len(), 2);
            // 'r--o': قرمز خط‌چین با دایره
            assert_eq!(
                series[0].style.color,
                Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 })
            );
            assert_eq!(series[0].style.dash, Some(DashSpec::Dashed));
            assert_eq!(series[0].style.marker, Some(MarkerSpec::Circle));
            // 'b:s': آبی نقطه‌چین با مربع
            assert_eq!(series[1].style.dash, Some(DashSpec::Dotted));
            assert_eq!(series[1].style.marker, Some(MarkerSpec::Square));
        }
        other => panic!("expected Line, got {other:?}"),
    }
    assert!(matches!(ops[2], PlotOp::Hold { on: false }));
    match &ops[3] {
        PlotOp::Legend { entries, position } => {
            assert_eq!(
                entries.as_deref(),
                Some(&["sin".to_owned(), "cos".to_owned()][..])
            );
            assert_eq!(*position, LegendPos::SouthWest);
        }
        other => panic!("expected Legend, got {other:?}"),
    }
    assert!(matches!(ops[4], PlotOp::Grid { on: true }));
}

#[test]
fn command_forms_for_plot_fns() {
    // grid on / grid off / axis equal / legend sin cos
    assert!(matches!(first_op("grid on"), PlotOp::Grid { on: true }));
    assert!(matches!(first_op("grid off"), PlotOp::Grid { on: false }));
    assert!(matches!(
        first_op("axis equal"),
        PlotOp::AxisEqual { on: true }
    ));
    assert!(matches!(first_op("axis tight"), PlotOp::AxisTight));
    match first_op("legend sin cos") {
        PlotOp::Legend { entries, .. } => {
            assert_eq!(entries, Some(vec!["sin".to_owned(), "cos".to_owned()]))
        }
        other => panic!("expected Legend, got {other:?}"),
    }
    // فرمان با نقطه‌ویرگول و بدون آن، هر دو
    let mut env = Environment::new();
    let out = run(&mut env, "grid on; hold on");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert!(matches!(out.plot_ops[0], PlotOp::Grid { on: true }));
    assert!(matches!(out.plot_ops[1], PlotOp::Hold { on: true }));
}

#[test]
fn bare_plot_fns_take_default_actions() {
    // hold برهنه = toggle، grid برهنه = toggle، figure برهنه = شکل تازه
    assert!(matches!(first_op("hold"), PlotOp::HoldToggle));
    assert!(matches!(first_op("grid"), PlotOp::GridToggle));
    match first_op("figure") {
        PlotOp::ActivateFigure { id } => assert!(id.is_none()),
        other => panic!("expected ActivateFigure, got {other:?}"),
    }
    // plot برهنه = سلول خالی (پارتی متلب)
    match first_op("plot") {
        PlotOp::Line { series } => assert!(series.is_empty()),
        other => panic!("expected empty Line, got {other:?}"),
    }
}

#[test]
fn shadowed_names_keep_normal_semantics() {
    // متغیر هم‌نام بر فرمان می‌چربد — مثل متلب `hold on` با hold عددی
    // به اندیس‌گذاری می‌خورد (ارور صادقانه، نه فراخوانی رسم)
    let e = err_of("hold = 5; hold on");
    assert!(e.message_en.contains("subscript"), "{e}");
    // فرمان ناشناخته → خطای صادقانه (نه کرش)
    let e2 = err_of("frobnicate yes");
    assert!(!e2.message_en.is_empty());
}

/// گام 3.5.2 — باگ کاربر: بعد از نشست legend('sin','cos')، اجرای
/// `semilogx(x,y,'LineWidth',1.5)` در همان نشست زیر نمودار تازه همچنان
/// sin/cos نشان می‌داد. دو سابمیشن واقعی روی یک Environment + fold در
/// FigureState — legend باید پاک شده باشد و لگاریتمی/شبکه روشن بمانند.
#[test]
fn user_repro_stale_legend_after_semilogx() {
    use mc_plot::figure::FigureState;
    let mut env = Environment::new();
    let out1 = run(
        &mut env,
        "x = 0:0.1:2*pi; plot(x, sin(x), 'r--o', x, cos(x), 'b:s'); \
         legend('sin','cos','Location','SouthWest')",
    );
    assert!(out1.error.is_none(), "{:?}", out1.error);
    let out2 = run(
        &mut env,
        "x = logspace(0,1,1000); y = 10.^(x-1); \
         semilogx(x,y,'LineWidth',1.5); grid on",
    );
    assert!(out2.error.is_none(), "{:?}", out2.error);

    // ترتیب op های semilogx: اول Line بعد SetLog (پس از گام 3.5.2)
    let mut saw_line_then_setlog = false;
    for w in out2.plot_ops.windows(2) {
        if matches!(w[0], PlotOp::Line { .. }) && matches!(w[1], PlotOp::SetLog { x: true, .. }) {
            saw_line_then_setlog = true;
        }
    }
    assert!(saw_line_then_setlog, "ops: {:?}", out2.plot_ops);

    let mut st = FigureState::new();
    for op in out1.plot_ops.iter().chain(&out2.plot_ops) {
        st.apply(op.clone());
    }
    let CellSnapshot::Line {
        series,
        axes,
        legend,
    } = &st.snapshots()[0].spec
    else {
        panic!("expected Line");
    };
    assert_eq!(series.len(), 1, "new submission replaces old series");
    assert_eq!(series[0].raw_len, 1000);
    assert!(legend.is_none(), "STALE sin/cos legend must be gone");
    assert!(axes.log_x, "semilogx keeps its own log flag");
    assert!(axes.grid, "grid on from the same submission");
}
