use crate::analysis::build_plan::{CompileMode, TargetKind};
use crate::{
    analysis::{
        build_plan::{BuildPlan, Invocation},
        callbacks::TaintCompilerCallbacks,
        invocation_environment::InvocationEnvironment,
    },
    rustc,
};
use anyhow::anyhow;
use std::io::{BufReader, Read};
use std::{
    env, fs,
    process::{Command, Stdio},
};
use crate::analysis::taint_source::{get_taint_sources_definitions, TaintSource};

pub fn execute_build_plan(mut build_plan: BuildPlan) -> anyhow::Result<()> {
    let taint_sources = get_taint_sources_definitions();
    let mut actual_used_taint_sources = Vec::default();
    for module in &taint_sources {
        actual_used_taint_sources.extend(module.sources.clone().into_iter());
    }
    let actual_used_taint_sources = &actual_used_taint_sources;

    let total_invocations = build_plan.invocations.len();

    for i in 0..build_plan.invocations.len() {
        let current = build_plan.invocations.get(i).unwrap();

        println!(
            "Compiling {}/{}: {} v{}",
            i + 1,
            total_invocations,
            current.package_name,
            current.package_version
        );

        let links = current.links.clone();

        

        match current.compile_mode {
            CompileMode::Build => {
                let results = execute_build_invocation_mir_analysis(current, &actual_used_taint_sources)?;

                println!(
                    " - Found {} functions",
                    results.internal_interface_functions.len()
                );
                for func in results.internal_interface_functions.iter().take(10) {
                    println!("    - {}", func.function_name)
                }
                if results.internal_interface_functions.len() > 10 {
                    println!("      ...")
                }
            }
            CompileMode::RunCustomBuild => {
                let mut cmd = Command::new(&current.program)
                    .args(&current.args)
                    .envs(&current.env)
                    .current_dir(
                        &current
                            .cwd
                            .clone()
                            .map(Ok)
                            .unwrap_or_else(env::current_dir)?,
                    )
                    .stdout(Stdio::piped())
                    .spawn()?;
                let exit_status = cmd.wait()?;

                if !exit_status.success() {
                    return Err(anyhow::anyhow!("Failed to run custom build script"));
                }

                let mut reader = BufReader::new(cmd.stdout.unwrap());
                let mut output = String::new();
                reader.read_to_string(&mut output)?;

                for line in output.lines() {
                    let command = BuildScriptCommand::parse(line);
                    match command {
                        None => {
                            println!(" - Unknown command: {}", line);
                        }
                        Some(Err(e)) => return Err(e),
                        Some(Ok(cmd)) => {
                            let mut next = build_plan.invocations.get_mut(i + 1);
                            if next.is_none() {
                                continue;
                            }
                            let next = next.as_mut().unwrap();

                            match cmd {
                                BuildScriptCommand::LinkArg(arg) => {
                                    if matches!(
                                        next.target_kind,
                                        TargetKind::Bench
                                            | TargetKind::Bin
                                            | TargetKind::ExampleBin
                                            | TargetKind::Test
                                    ) || next.target_kind.is_cdylib()
                                    {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkArgBin(bin, arg) => {
                                    if next.target_kind == TargetKind::Bin
                                        && next.package_name == bin
                                    {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkArgBins(arg) => {
                                    if next.target_kind == TargetKind::Bin {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkLib(arg) => {
                                    if matches!(next.target_kind, TargetKind::Lib(_)) {
                                        next.args.push("-l".to_owned());
                                        next.args.push(arg);
                                    }
                                }
                                BuildScriptCommand::LinkArgTests(arg) => {
                                    if matches!(next.target_kind, TargetKind::Test) {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkArgExamples(arg) => {
                                    if matches!(
                                        next.target_kind,
                                        TargetKind::ExampleBin | TargetKind::ExampleLib(_)
                                    ) {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkArgBenches(arg) => {
                                    if matches!(next.target_kind, TargetKind::Bench) {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::LinkSearch(kind, path) => {
                                    let prefix = match kind {
                                        Some(kind) => format!("{}=", kind),
                                        None => "".to_owned(),
                                    };
                                    next.args.push("-L".to_owned());
                                    next.args.push(prefix + &path);
                                }
                                BuildScriptCommand::Flags(flags) => {
                                    let split = flags
                                        .split(" ")
                                        .filter(|s| s.to_lowercase().starts_with("-l"));
                                    for flag in split {
                                        next.args.push(flag.to_owned());
                                    }
                                }
                                BuildScriptCommand::Cfg(key, value) => {
                                    let combined = if let Some(value) = value {
                                        format!("{}={}", key, value)
                                    } else {
                                        key
                                    };
                                    next.args.push("--cfg".to_owned());
                                    next.args.push(combined);
                                }
                                BuildScriptCommand::Env(key, value) => {
                                    next.env.insert(key, value);
                                }
                                BuildScriptCommand::CdylibLinkArg(arg) => {
                                    if next.target_kind.is_cdylib() {
                                        next.args.push("-C".to_owned());
                                        next.args.push("link-arg=".to_owned() + &arg);
                                    }
                                }
                                BuildScriptCommand::RustcCheckCfg(_arg) => {}
                                BuildScriptCommand::RerunIfChanged(_arg) => {}
                            }
                        }
                    }
                }
            }
        }

        // create hardlinks
        for (link_name, target) in links {
            if fs::metadata(&link_name).is_ok() {
                fs::remove_file(&link_name)?;
            }

            if fs::metadata(&target).is_err() {
                // target does not exist
                continue;
            }

            if let Err(error) = fs::hard_link(&target, &link_name) {
                return Err(anyhow::anyhow!("Failed to create hardlink: {}", error));
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn execute_build_invocation_original_rustc(invocation: &Invocation) -> anyhow::Result<()> {
    // let environment = InvocationEnvironment::enter(&invocation);
    let mut command = Command::new(&invocation.program)
        .args(&invocation.args)
        .envs(&invocation.env)
        .current_dir(
            &invocation
                .cwd
                .clone()
                .map(Ok)
                .unwrap_or_else(env::current_dir)?,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let error = command.wait()?;
    if !error.success() {
        return Err(anyhow::anyhow!("Failed to compile"));
    }

    Ok(())
}

fn execute_build_invocation_mir_analysis<'tsrc>(
    invocation: &Invocation,
    taint_sources: &'tsrc Vec<TaintSource<'static>>
) -> anyhow::Result<TaintCompilerCallbacks<'tsrc>> {
    // print args
    let mut args = invocation.args.clone();
    args.insert(0, invocation.program.clone());

    let environment = InvocationEnvironment::enter(invocation);
    let mut callbacks = TaintCompilerCallbacks {
        package_name: invocation.package_name.clone(),
        package_version: invocation.package_version.clone(),
        internal_interface_functions: Vec::default(),
        taint_sources,
    };

    rustc::run_compiler(args, callbacks.cast_to_dyn())?;
    drop(environment);

    Ok(callbacks)
}

#[derive(Debug)]
enum BuildScriptCommand {
    LinkArg(String),
    LinkArgBin(String, String), // BIN, FLAG
    LinkArgBins(String),
    LinkLib(String),
    LinkArgTests(String),
    LinkArgExamples(String),
    LinkArgBenches(String),
    LinkSearch(Option<String>, String), // KIND, PATH
    Flags(String),
    Cfg(String, Option<String>), // KEY, VALUE
    Env(String, String),         // KEY, VALUE
    CdylibLinkArg(String),
    RerunIfChanged(String),
    RustcCheckCfg(String),
}

impl BuildScriptCommand {
    pub fn parse(text: &str) -> Option<anyhow::Result<Self>> {
        const RUSTC_LINK_ARG: &str = "cargo:rustc-link-arg=";
        const RUSTC_LINK_ARG_BIN: &str = "cargo:rustc-link-arg-bin=";
        const RUSTC_LINK_ARG_BINS: &str = "cargo:rustc-link-arg-bins=";
        const RUSTC_LINK_LIB: &str = "cargo:rustc-link-lib=";
        const RUSTC_LINK_ARG_TESTS: &str = "cargo:rustc-link-arg=tests=";
        const RUSTC_LINK_ARG_EXAMPLES: &str = "cargo:rustc-link-arg=examples=";
        const RUSTC_LINK_ARG_BENCHES: &str = "cargo:rustc-link-arg=benches=";
        const RUSTC_LINK_SEARCH: &str = "cargo:rustc-link-search=";
        const RUSTC_FLAGS: &str = "cargo:rustc-flags=";
        const RUSTC_CFG: &str = "cargo:rustc-cfg=";
        const RUSTC_ENV: &str = "cargo:rustc-env=";
        const RUSTC_CDYLIB_LINK_ARG: &str = "cargo:rustc-cdylib-link-arg=";
        const RUSTC_RERUN_IF_CHANGED: &str = "cargo:rerun-if-changed=";
        const RUSTC_CHECK_CFG: &str = "cargo:rustc-check-cfg=";

        Some(Ok(if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG) {
            BuildScriptCommand::LinkArg(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG_BIN) {
            let parts = arg.split('=').collect::<Vec<&str>>();
            if parts.len() < 2 {
                return Some(Err(anyhow!("Link arg not properly formatted")));
            }
            let bin = parts[0].to_owned();
            let arg = parts[1..].join("=");
            BuildScriptCommand::LinkArgBin(bin, arg)
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG_BINS) {
            BuildScriptCommand::LinkArgBins(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_LIB) {
            BuildScriptCommand::LinkLib(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG_TESTS) {
            BuildScriptCommand::LinkArgTests(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG_EXAMPLES) {
            BuildScriptCommand::LinkArgExamples(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_ARG_BENCHES) {
            BuildScriptCommand::LinkArgBenches(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_LINK_SEARCH) {
            let parts: Vec<&str> = arg.split('=').collect();
            if parts.len() == 1 {
                BuildScriptCommand::LinkSearch(None, parts[0].to_owned())
            } else {
                BuildScriptCommand::LinkSearch(Some(parts[0].to_owned()), parts[1..].join("="))
            }
        } else if let Some(arg) = text.strip_prefix(RUSTC_FLAGS) {
            BuildScriptCommand::Flags(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_CFG) {
            let parts: Vec<&str> = arg.split('=').collect();
            if parts.len() == 1 {
                BuildScriptCommand::Cfg(parts[0].to_owned(), None)
            } else {
                BuildScriptCommand::Cfg(parts[0].to_owned(), Some(parts[1..].join("=")))
            }
        } else if let Some(arg) = text.strip_prefix(RUSTC_ENV) {
            let parts: Vec<&str> = arg.split('=').collect();
            if parts.len() < 2 {
                return Some(Err(anyhow!("Env not properly formatted")));
            }
            BuildScriptCommand::Env(parts[0].to_owned(), parts[1..].join("=").to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_CDYLIB_LINK_ARG) {
            BuildScriptCommand::CdylibLinkArg(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_RERUN_IF_CHANGED) {
            BuildScriptCommand::RerunIfChanged(arg.to_owned())
        } else if let Some(arg) = text.strip_prefix(RUSTC_CHECK_CFG) {
            BuildScriptCommand::RustcCheckCfg(arg.to_owned())
        } else {
            return None;
        }))
    }
}
