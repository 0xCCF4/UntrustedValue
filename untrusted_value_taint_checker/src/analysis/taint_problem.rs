use rustc_span::Span;

use super::taint_source::TaintSource;

pub struct TaintProblem<'tsrc> {
    _taint_source: &'tsrc TaintSource<'static>,
    _used_at: Span,
}
