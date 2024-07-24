#[derive(Debug, Clone)]

pub struct TaintSourceDefinition<'a> {
    pub taint_module_name: &'a str,
    pub taint_module_description: &'a str,
    pub sources: Vec<TaintSource<'a>>,
}

#[derive(Debug, Clone)]
pub struct TaintSource<'a> {
    pub functions: Vec<&'a str>,
    pub description: &'a str,
}

mod generated;

pub fn get_taint_sources_definitions() -> Vec<TaintSourceDefinition<'static>> {
    generated::get_taint_sources_definitions()
}
