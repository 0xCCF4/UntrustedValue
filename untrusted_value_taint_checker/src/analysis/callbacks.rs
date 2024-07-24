extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_driver::Compilation;
use rustc_hir::intravisit::Visitor;
use rustc_middle::ty::TyCtxt;
use tracing::{event, span, Level};

pub struct TaintCompilerCallbacks {
    pub package_name: String,
    pub package_version: semver::Version,
    pub internal_interface_functions: Vec<FunctionInfo>,
}

impl TaintCompilerCallbacks {
    pub fn cast_to_dyn(&mut self) -> &mut (dyn rustc_driver::Callbacks + Send) {
        self
    }
}

impl rustc_driver::Callbacks for TaintCompilerCallbacks {
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

pub struct FunctionInfo {
    pub function_name: String,
    pub body_id: rustc_hir::BodyId,
    pub local_def_id: rustc_hir::def_id::LocalDefId,
    pub span: rustc_span::Span,
}

struct CrateVisitor<'a, 'tcx> {
    pub tcx: &'a TyCtxt<'tcx>,
    internal_functions: Vec<FunctionInfo>,
}

impl<'a, 'tcx> CrateVisitor<'a, 'tcx> {
    pub fn new(tcx: &'a TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            internal_functions: Vec::default(),
        }
    }
}

impl<'v, 'a, 'tcx> Visitor<'v> for CrateVisitor<'a, 'tcx> {
    fn visit_fn(
        &mut self,
        _kind: rustc_hir::intravisit::FnKind<'v>,
        _decl: &'v rustc_hir::FnDecl<'v>,
        body_id: rustc_hir::BodyId,
        span: rustc_span::Span,
        local_def_id: rustc_hir::def_id::LocalDefId,
    ) -> Self::Result {
        let function_name = self.tcx.def_path_str(local_def_id);
        self.internal_functions.push(FunctionInfo {
            body_id,
            span,
            local_def_id,
            function_name,
        });
    }
}

pub fn mir_analysis(tcx: TyCtxt, callback_data: &mut TaintCompilerCallbacks) {
    // let mut finder = TaintAttributeFinder::new(tcx);

    let span = span!(Level::TRACE, "Public interface analysis");
    let _enter = span.enter();

    let mut hir_analysis = CrateVisitor::new(&tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut hir_analysis);

    for finfo in &hir_analysis.internal_functions {
        event!(
            Level::TRACE,
            function_name = finfo.function_name,
            source_code = format!("{:?}", finfo.span)
        );
    }

    callback_data.internal_interface_functions =
        std::mem::take(&mut hir_analysis.internal_functions);
}
