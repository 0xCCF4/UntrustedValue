use std::{
    fs,
    process::{Command, Stdio},
};

use crate::{
    analysis::{
        build_plan::{BuildInvocation, BuildPlan, CompileMode},
        callbacks::TaintCompilerCallbacks,
        invocation_environment::InvocationEnvironment,
    },
    rustc,
};
use anyhow::anyhow;

pub fn execute_build_plan(build_plan: BuildPlan) -> anyhow::Result<()> {
    let total_invocations = build_plan.invocations.len();

    for (i, invocation) in build_plan.invocations.into_iter().enumerate() {
        println!(
            "Compiling {}/{}: {} v{}",
            i + 1,
            total_invocations,
            invocation.package_name,
            invocation.package_version
        );

        let links = invocation.links.clone();

        match invocation.compile_mode {
            CompileMode::Build => {
                let results = execute_build_invocation_mir_analysis(invocation)?;

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
                let mut cmd = Command::new(invocation.program)
                    .args(invocation.args)
                    .envs(invocation.env)
                    .current_dir(invocation.cwd)
                    .spawn()?;
                let exit_status = cmd.wait()?;

                if !exit_status.success() {
                    return Err(anyhow::anyhow!("Failed to run custom build script"));
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
fn execute_build_invocation_original_rustc(invocation: BuildInvocation) -> anyhow::Result<()> {
    // let environment = InvocationEnvironment::enter(&invocation);
    let mut command = Command::new(invocation.program)
        .args(invocation.args)
        .envs(invocation.env)
        .current_dir(invocation.cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let error = command.wait()?;
    if !error.success() {
        return Err(anyhow::anyhow!("Failed to compile"));
    }

    Ok(())
}

fn execute_build_invocation_mir_analysis(
    invocation: BuildInvocation,
) -> anyhow::Result<TaintCompilerCallbacks> {
    // print args
    let mut args = invocation.args.clone();
    args.insert(
        0,
        invocation
            .program
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow!("Path is not an UTF8-string"))?
            .to_owned(),
    );

    let environment = InvocationEnvironment::enter(&invocation);
    let mut callbacks = TaintCompilerCallbacks {
        package_name: invocation.package_name,
        package_version: invocation.package_version,
        internal_interface_functions: Vec::default(),
    };

    rustc::run_compiler(args, callbacks.cast_to_dyn())?;
    drop(environment);

    Ok(callbacks)
}
