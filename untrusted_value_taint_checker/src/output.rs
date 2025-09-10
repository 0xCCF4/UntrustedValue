use crate::analysis::taint_problem::TaintProblemOwned;
use crate::analysis::taint_source::TaintSourceDefinitionOwned;
use crate::semver_to_string;
use crate::IRSpan;
use owo_colors::colors::xterm::LightGray;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisProblem {
    pub description: String,
    pub problematic_function: String,
    pub problematic_function_loc: IRSpan,
    pub taint_source: String,
    pub taint_source_loc: IRSpan,
    pub detailed: TaintProblemOwned,
}

#[derive(Debug, Clone, Serialize)]
pub enum PackageAnalysisResult {
    Success,
    Failure(Vec<AnalysisProblem>),
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    pub package_name: String,
    #[serde(
        deserialize_with = "semver_from_string",
        serialize_with = "semver_to_string"
    )]
    pub package_version: semver::Version,
    pub result: PackageAnalysisResult,
}

#[derive(Debug, Clone, Serialize)]
pub enum ProgramOutput {
    AnalysisResult(AnalysisResult),
    TaintSourceList(TaintSourceDefinitionOwned),
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum OutputFormatType {
    Console,
    Json,
}

impl FromStr for OutputFormatType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" => Ok(OutputFormatType::Console),
            "json" => Ok(OutputFormatType::Json),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

pub trait OutputFormat {
    fn to_console(&self) -> String;
    fn to_json(&self) -> String;

    fn to_format(&self, format: OutputFormatType) -> String {
        match format {
            OutputFormatType::Console => self.to_console(),
            OutputFormatType::Json => self.to_json(),
        }
    }
}

impl OutputFormat for AnalysisResult {
    fn to_console(&self) -> String {
        let mut result = "".to_owned();
        match &self.result {
            PackageAnalysisResult::Success => {
                result.push_str(&format!(
                    "{} in package {} v{}",
                    "No problems found".green(),
                    self.package_name.yellow(),
                    self.package_version.fg::<LightGray>()
                ));
            }
            PackageAnalysisResult::Failure(problems) => {
                for problem in problems {
                    result.push_str(&format!(
                        "{} found in {}:{} {:?}\n",
                        "Sanitizing problem".red().bold(),
                        self.package_name.yellow(),
                        problem.problematic_function.italic().yellow(),
                        problem.problematic_function_loc.fg::<LightGray>()
                    ));
                    result.push_str(&format!(
                        " {} Usage of {} at {}\n",
                        "|".bold(),
                        problem.taint_source.bold().blue(),
                        problem.taint_source_loc.fg::<LightGray>()
                    ));
                    result.push_str(&format!(
                        " {} without wrapping the result as {} is discouraged\n",
                        "|".bold(),
                        "UntrustedValue".bold().green()
                    ));
                    result.push_str(&format!(
                        " {} {} {}\n",
                        "|".bold(),
                        ">".italic(),
                        problem.description.italic()
                    ));
                    result.push_str(&format!(
                        " {} Make sure to wrap the result like this {}{}(...){}\n\n",
                        "|".bold(),
                        "UntrustedValue::from(".bold().green(),
                        problem.taint_source.bold().blue(),
                        ")".bold().green()
                    ));
                }
            }
        }
        result
    }
    fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|error| format!("Error serializing to JSON: {}", error))
    }
}

impl OutputFormat for TaintSourceDefinitionOwned {
    fn to_console(&self) -> String {
        let mut result = "".to_owned();
        result.push_str(&format!(
            "Taint sources module {}:\n",
            self.taint_module_name.bold().green()
        ));
        result.push_str(&format!(
            " {} {}\n",
            "|".bold(),
            ("> ".to_owned() + &self.taint_module_description)
                .italic()
                .blue()
        ));
        for source in &self.sources {
            result.push_str(&format!(
                " {} {}\n",
                "|".bold(),
                ("> ".to_owned() + &source.description).italic().blue()
            ));
            for function in &source.functions {
                result.push_str(&format!(
                    " {} - {}\n",
                    "|".bold(),
                    function.italic().yellow()
                ));
            }
        }
        result
    }
    fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|error| format!("Error serializing to JSON: {}", error))
    }
}

impl OutputFormat for ProgramOutput {
    fn to_console(&self) -> String {
        match self {
            ProgramOutput::Message(message) => message.to_owned(),
            ProgramOutput::AnalysisResult(analysis_result) => analysis_result.to_console(),
            ProgramOutput::TaintSourceList(taint_source_list) => taint_source_list.to_console(),
        }
    }
    fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|error| format!("Error serializing to JSON: {}", error))
    }
}
