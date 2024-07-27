#![feature(rustc_private)]
#![feature(box_patterns)]

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
