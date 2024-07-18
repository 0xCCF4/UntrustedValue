use syn::visit::Visit;

#[derive(Default)]
pub struct TaintChecker {}

impl<'ast> Visit<'ast> for TaintChecker {}
