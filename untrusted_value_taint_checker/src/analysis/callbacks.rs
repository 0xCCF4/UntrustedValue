use std::path::PathBuf;

use crate::analysis::taint_source::TaintSource;
use petgraph::dot::Dot;
use rustc_driver::Compilation;
use rustc_middle::{mir::visit::Visitor as VisitorMir, ty::TyCtxt};
use tracing::{event, span, Level};

use super::{
    hir::crate_function_finder::{CrateFunctionFinder, FunctionInfo},
    mir::data_flow::DataFlowTaintTracker,
};

pub struct TaintCompilerCallbacks<'tsrc> {
    pub package_name: String,
    pub package_version: semver::Version,
    pub taint_sources: &'tsrc Vec<TaintSource<'static>>,
    pub internal_interface_functions: Vec<FunctionInfo>,
}

impl<'tsrc> TaintCompilerCallbacks<'tsrc> {
    pub fn cast_to_dyn(&mut self) -> &mut (dyn rustc_driver::Callbacks + Send) {
        self
    }
}

impl<'tsrc> rustc_driver::Callbacks for TaintCompilerCallbacks<'tsrc> {
    /// All the work we do happens after analysis, so that we can make assumptions about the validity of the MIR.
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        compiler.sess.dcx().abort_if_errors();
        enter_with_fn(queries, self, mir_analysis);
        compiler.sess.dcx().abort_if_errors();
        Compilation::Continue
    }
}

fn enter_with_fn<'tcx, TyCtxtFn>(
    queries: &'tcx rustc_interface::Queries<'tcx>,
    callback_data: &mut TaintCompilerCallbacks,
    enter_fn: TyCtxtFn,
) where
    TyCtxtFn: Fn(TyCtxt, &mut TaintCompilerCallbacks),
{
    queries
        .global_ctxt()
        .unwrap()
        .enter(move |context| enter_fn(context, callback_data));
}

pub fn mir_analysis(tcx: TyCtxt, callback_data: &mut TaintCompilerCallbacks) {
    // let mut finder = TaintAttributeFinder::new(tcx);

    let span = span!(Level::TRACE, "Public interface analysis");
    let _enter = span.enter();

    let mut hir_analysis = CrateFunctionFinder::new(tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut hir_analysis);
    let functions = hir_analysis.results();

    for finfo in &functions {
        event!(
            Level::TRACE,
            function_name = finfo.function_name,
            source_code = format!("{:?}", finfo.span)
        );
    }

    for function in &functions {
        if callback_data.package_name == "sample"
            || callback_data.package_name.starts_with("untrusted_value")
        {
            let body = tcx.optimized_mir(function.local_def_id);
            let mut tracker = DataFlowTaintTracker::new(tcx, body);

            println!("{}", function.function_name);
            tracker.visit_body(body);
            println!("\n\n\n");
            let dir_path = PathBuf::from("/tmp/taint/").join(&callback_data.package_name);
            std::fs::create_dir_all(&dir_path).expect("Failed to create directory");
            let dot_file = dir_path.join(&function.function_name).with_extension("dot");
            let dot = Dot::with_config(&tracker.data_dependency_graph, &[]);
            std::fs::write(&dot_file, format!("{:?}", dot)).expect("Failed to write dot file");
            let pdf_file = dot_file.with_extension("pdf");
            std::process::Command::new("dot")
                .arg("-Tpdf")
                .arg("-o")
                .arg(&pdf_file)
                .arg(&dot_file)
                .output()
                .expect("Failed to execute dot command");
            std::fs::remove_file(&dot_file).expect("Failed to delete dot file");
        }
        println!("\n\n\n");
    }
    let mut functions = functions;
    callback_data.internal_interface_functions = std::mem::take(&mut functions);
}
