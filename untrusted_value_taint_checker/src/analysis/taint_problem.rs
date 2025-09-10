use crate::IRSpan;
use serde::{Deserialize, Serialize};

use super::taint_source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintUsageType {
    ReturnedToCaller,
    FunctionCall(IRSpan),
    Assembly,
    ControlFlow,
    Copied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Into<TaintProblemOwned> for TaintProblem<'_> {
    fn into(self) -> TaintProblemOwned {
        TaintProblemOwned {
            taint_source_description: self.taint_source.description.to_owned(),
            source_func_sig: self.source_func_sig,
            source_span: self.source_span,
            problem_type: self.problem_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintProblemOwned {
    pub taint_source_description: String,
    pub source_func_sig: String,
    pub source_span: IRSpan,
    pub problem_type: TaintProblemType,
}
