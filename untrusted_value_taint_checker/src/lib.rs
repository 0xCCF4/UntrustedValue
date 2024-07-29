#![feature(rustc_private)]
#![feature(box_patterns)]

use std::{fmt::{Debug, Display}, path::PathBuf};

use rustc_middle::ty::TyCtxt;

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

pub mod cargo;
pub mod rustc;
pub mod analysis {
    // pub mod attribute_finder;
    pub mod build_plan;
    pub mod callbacks;
    pub mod invocation_environment;
    pub mod taint_problem;
    pub mod taint_source;

    pub mod hir {
        pub mod crate_function_finder;
    }
    pub mod mir {
        pub mod data_flow;
        pub mod data_flow_checker;
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct IRSpan {
    pub file_name: Option<PathBuf>,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl IRSpan {
    pub fn new(span: rustc_span::Span, tcx: TyCtxt) -> IRSpan {
        let file = tcx.sess.source_map().lookup_source_file(span.lo()).name.clone().into_local_path();
        let start = tcx.sess.source_map().lookup_char_pos(span.lo());
        let end = tcx.sess.source_map().lookup_char_pos(span.hi());
        IRSpan {
            file_name: file,
            start_line: start.line,
            start_col: start.col.0,
            end_line: end.line,
            end_col: end.col.0,
        }
    }
}

impl Debug for IRSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}-{}:{}", self.file_name.as_ref().map(|p| p.to_string_lossy()).unwrap_or("unknown".into()),
            self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}

impl Display for IRSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}