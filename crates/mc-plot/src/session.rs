//! سینک عملیات رسم — کانال بدون‌حالت بین مفسر و کاهش‌دهنده.
//!
//! توابع رسم داخلی، عملیات را با [`emit`] می‌نویسند؛ `eval_program` در
//! پایان هر ارزیابی (چه موفق، چه خطا — متلب نمودارهای پیش از خطا را نگه
//! می‌دارد) با [`take`] تخلیه‌اش می‌کند و در `EvalOutcome.plot_ops` می‌گذارد.
//! thread-local است پس تست‌های موازی cargo هم‌دیگر را آلوده نمی‌کنند
//! (همان الگوی STMT_TICK در mc-interp).

use crate::spec::PlotOp;
use std::cell::RefCell;

thread_local! {
    static SINK: RefCell<Vec<PlotOp>> = const { RefCell::new(Vec::new()) };
}

/// ثبت یک عملیات رسم (به‌ترتیب).
pub fn emit(op: PlotOp) {
    SINK.with(|s| s.borrow_mut().push(op));
}

/// تخلیه و بازگرداندن کل عملیات‌های ثبت‌شده روی این نخ (به‌ترتیب).
pub fn take() -> Vec<PlotOp> {
    SINK.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::SeriesData;

    fn sample_op() -> PlotOp {
        PlotOp::Line {
            series: vec![crate::spec::SeriesPair {
                data: SeriesData {
                    x: vec![1.0],
                    y: vec![2.0],
                },
                style: crate::spec::StyleSpec::default(),
            }],
        }
    }

    #[test]
    fn take_drains_in_order_and_resets() {
        emit(PlotOp::Hold { on: true });
        emit(sample_op());
        let drained = take();
        assert_eq!(drained.len(), 2);
        assert!(matches!(drained[0], PlotOp::Hold { on: true }));
        assert!(matches!(drained[1], PlotOp::Line { .. }));
        // تخلیهٔ دوم = خالی (بدون نشت بین فراخوانی‌ها)
        assert!(take().is_empty());
    }
}
