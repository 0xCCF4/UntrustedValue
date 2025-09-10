use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TaintSourceDefinition<'a> {
    pub taint_module_name: &'a str,
    pub taint_module_description: &'a str,
    pub sources: Vec<TaintSource<'a>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TaintSource<'a> {
    pub functions: Vec<&'a str>,
    pub description: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSourceDefinitionOwned {
    pub taint_module_name: String,
    pub taint_module_description: String,
    pub sources: Vec<TaintSourceOwned>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSourceOwned {
    pub functions: Vec<String>,
    pub description: String,
}

impl From<TaintSource<'_>> for TaintSourceOwned {
    fn from(source: TaintSource) -> Self {
        TaintSourceOwned {
            functions: source.functions.iter().map(|s| s.to_string()).collect(),
            description: source.description.to_owned(),
        }
    }
}

impl From<&TaintSource<'_>> for TaintSourceOwned {
    fn from(source: &TaintSource) -> Self {
        TaintSourceOwned {
            functions: source.functions.iter().map(|s| s.to_string()).collect(),
            description: source.description.to_owned(),
        }
    }
}

impl From<TaintSourceDefinition<'_>> for TaintSourceDefinitionOwned {
    fn from(def: TaintSourceDefinition) -> Self {
        TaintSourceDefinitionOwned {
            taint_module_name: def.taint_module_name.to_owned(),
            taint_module_description: def.taint_module_description.to_owned(),
            sources: def.sources.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<&TaintSourceDefinition<'_>> for TaintSourceDefinitionOwned {
    fn from(def: &TaintSourceDefinition) -> Self {
        TaintSourceDefinitionOwned {
            taint_module_name: def.taint_module_name.to_owned(),
            taint_module_description: def.taint_module_description.to_owned(),
            sources: def.sources.iter().map(|s| s.into()).collect(),
        }
    }
}

mod generated;

pub fn get_taint_sources_definitions() -> Vec<TaintSourceDefinition<'static>> {
    generated::get_taint_sources_definitions()
}
