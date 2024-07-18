#[allow(unused_imports)]
use syn::visit::Visit;

#[derive(Default)]
pub struct TaintChecker {}

impl TaintChecker {
    #[allow(unused_variables)]
    pub fn process_file(&self, file: &syn::File) {
        // register checkers here
    }
}
