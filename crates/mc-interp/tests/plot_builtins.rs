//! تست‌های توابع رسم گزاره‌ای (ROADMAP 3.2 — P3-a).
//!
//! سطح تست = دنبالهٔ `EvalOutcome.plot_ops` (گرام عملیات)؛ کاهشِ حالت شکل‌ها
//! در mc-plot/figure.rs جداگانه تست می‌شود. واگرایی‌های MATLAB-lite در
//! COMPATIBILITY ثبت شده‌اند (یک استایل در هر فراخوانی، بدون toggle-hold،
//! fplot با ۴۰۰ نمونهٔ ثابت، …).

use mc_interp::{eval_program, Environment};
use mc_parser::parse;
use mc_plot::spec::{DashSpec, HistNorm, MarkerSpec, PlotColorSpec, PlotOp, SeriesPair};

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
fn fill_pairs_share_one_style() {
    let mut env = Environment::new();
    let out = run(&mut env, "fill([0 1 1], [0 0 1], [2 3 2], [1 1 0], 'r')");
    assert!(out.error.is_none(), "{:?}", out.error);
    assert_eq!(out.plot_ops.len(), 1);
    match &out.plot_ops[0] {
        PlotOp::Fill { polys, style } => {
            assert_eq!(polys.len(), 2);
            assert_eq!(style.color, Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 }));
        }
        other => panic!("expected Fill, got {other:?}"),
    }
    // تعداد فرد = خطا
    assert!(err_of("fill([0 1], [0 1], [5 6])")
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
        PlotOp::Legend { entries } => {
            assert_eq!(entries, Some(vec!["sin".to_owned(), "cos".to_owned()]));
        }
        other => panic!("expected Legend, got {other:?}"),
    }
    match first_op("legend('off')") {
        PlotOp::Legend { entries } => assert!(entries.is_none()),
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
    assert!(err_of("axis('auto')").message_en.contains("auto"));
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
    assert!(err_of("hold()").message_en.contains("one argument"));
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
