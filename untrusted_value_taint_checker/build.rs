use std::{fs, process::Command};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TaintModuleJson {
    pub taint_module_name: String,
    pub description: String,
    pub content: Vec<TaintModuleLibraryJson>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaintModuleLibraryJson {
    pub module_prefix: String,
    pub taint_sources: Vec<TaintSourceJson>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaintSourceJson {
    pub functions: Vec<String>,
    pub description: String,
}

#[allow(clippy::useless_format)]
fn main() {
    println!("cargo::rerun-if-changed=src/analysis/taint_source/");

    // list files in taint_source directory
    let taint_source_dir = std::path::Path::new("src/analysis/taint_source/");
    let taint_source_files =
        std::fs::read_dir(taint_source_dir).expect("Cannot read taint_source directory");

    let mut taint_modules = Vec::new();

    for file in taint_source_files {
        let file = file.expect("Cannot read directory entry");
        let file_path = file.path();
        if file_path.extension() != Some("json".as_ref()) {
            continue;
        }
        let file = fs::File::open(&file_path).expect("Cannot open file");

        let taint_module: TaintModuleJson = serde_json::from_reader(file).unwrap_or_else(|e| {
            panic!(
                "Cannot parse JSON in file {}: {}",
                file_path.to_string_lossy(),
                e
            )
        });
        taint_modules.push(taint_module);
    }

    let mut generated_code = String::new();

    generated_code.push_str(&format!("vec![\n"));
    let mut number = 0;
    for module in taint_modules {
        generated_code.push_str(&format!("        TaintSourceDefinition {{\n"));
        generated_code.push_str(&format!(
            "            taint_module_name: \"{}\",\n",
            module.taint_module_name
        ));
        generated_code.push_str(&format!(
            "            taint_module_description: \"{}\",\n",
            module.description
        ));
        generated_code.push_str(&format!("            sources: vec![\n"));
        for library in module.content {
            let prefix = if library.module_prefix.is_empty() {
                "".to_owned()
            } else {
                format!("{}::", library.module_prefix)
            };

            for source in library.taint_sources {
                generated_code.push_str(&format!("                TaintSource {{\n"));
                generated_code.push_str(&format!(
                    "                    description: \"{}\",\n",
                    source.description
                ));
                generated_code.push_str(&format!("                    functions: vec![\n"));

                for function in source.functions {
                    generated_code.push_str(&format!(
                        "                        \"{}{}\",\n",
                        prefix, function
                    ));
                }

                generated_code.push_str(&format!("                ],}},\n"));
            }
        }
        generated_code.push_str(&format!("        ],}},\n"));
        number += 1;
    }
    generated_code.push_str(&format!("    ]"));

    let template = std::fs::read_to_string(std::path::Path::new(
        "src/analysis/taint_source/generated.tpl",
    ))
    .expect("Cannot read template");
    let out_text = template
        .replace("<GENERATE>", &generated_code)
        .replace("<LEN>", format!("{number}").as_str());

    let out_file = std::path::Path::new("src/analysis/taint_source/generated.rs");
    fs::write(out_file, out_text).expect("Cannot write generated code");

    // run cargo fmt
    let cmd = Command::new("rustfmt").arg(out_file).spawn();

    let _ = cmd.map(|mut cmd| {
        cmd.wait().map(|exit| {
            if !exit.success() {
                panic!("Rustfmt returned non-zero exit code.")
            }
        })
    });
}
