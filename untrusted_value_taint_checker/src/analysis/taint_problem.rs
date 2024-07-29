use crate::IRSpan;

use super::taint_source::TaintSource;

#[derive(Debug, Clone)]
pub enum TaintUsageType {
    ReturnedToCaller,
    FunctionCall(IRSpan),
    Assembly,
    ControlFlow,
    Copied,
}

#[derive(Debug, Clone)]
pub enum TaintProblemType {
    DataDependencyLoop {
        statement_chain: Vec<IRSpan>,
        loop_closure: IRSpan,
    },
    Duplicated {
        statement_chain: Vec<IRSpan>,
        targets: Vec<IRSpan>,
    },
    Used {
        statement_chain: Vec<IRSpan>,
        used_in: IRSpan,
        usage_type: TaintUsageType,
    },
}

#[derive(Debug, Clone)]

pub struct TaintProblem<'tsrc> {
    pub taint_source: &'tsrc TaintSource<'static>,
    pub source_func_sig: String,
    pub source_span: IRSpan,
    pub problem_type: TaintProblemType,
}
