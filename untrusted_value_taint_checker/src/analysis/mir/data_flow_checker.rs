use petgraph::graph::DiGraph;
use rustc_middle::mir;

use crate::analysis::taint_problem::TaintProblem;

use super::data_flow::DependencyGraphNode;

pub fn check_data_flow<'tsrc, 'tcx>(
    _sanitized_locals: Vec<mir::Local>,
    _data_dependency_graph: DiGraph<DependencyGraphNode<'tcx>, Vec<rustc_span::Span>>,
) -> Vec<TaintProblem<'tsrc>> {
    Vec::new()
}
