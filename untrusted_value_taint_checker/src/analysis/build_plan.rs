use std::path::PathBuf;

use config::Map;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuildPlan {
    pub invocations: Vec<BuildInvocation>,
    pub inputs: Vec<PathBuf>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CompileMode {
    #[serde(rename = "run-custom-build")]
    RunCustomBuild,
    #[serde(rename = "build")]
    Build,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuildInvocation {
    pub package_name: String,
    pub package_version: String,
    pub target_kind: Vec<String>,
    pub kind: Option<String>,
    pub compile_mode: CompileMode,
    pub outputs: Vec<PathBuf>,
    pub links: Map<String, String>,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Map<String, String>,
    pub cwd: PathBuf,
}
