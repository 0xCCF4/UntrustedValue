use rustc_middle::ty;
use rustc_hir::intravisit::Visitor as HirVisitor;


pub struct FunctionInfo {
    pub function_name: String,
    pub body_id: rustc_hir::BodyId,
    pub local_def_id: rustc_hir::def_id::LocalDefId,
    pub span: rustc_span::Span,
}

pub struct CrateFunctionFinder<'tcx> {
    tcx: ty::TyCtxt<'tcx>,
    internal_functions: Vec<FunctionInfo>,
}

impl<'tcx> CrateFunctionFinder<'tcx> {
    pub fn new(tcx: ty::TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            internal_functions: Vec::default(),
        }
    }
    pub fn results(self) -> Vec<FunctionInfo> {
        self.internal_functions
    }
}

impl<'v, 'tcx> HirVisitor<'v> for CrateFunctionFinder<'tcx> {
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
