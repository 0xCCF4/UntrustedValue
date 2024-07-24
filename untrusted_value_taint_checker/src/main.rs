#![feature(rustc_private)]

use std::env;
use std::io::Read;
use std::process::{Command, Stdio};
use untrusted_value_taint_checker::analysis::build_plan::{BuildPlan, CompileMode};
use untrusted_value_taint_checker::cargo::{self};
use untrusted_value_taint_checker::rustc::init_hooks;

pub fn main() {
    init_hooks();

    let mut cargo_direct_args = vec![];
    let mut rustc_args = vec![];

    env::args()
        .skip(1)
        .take_while(|arg| arg != "--")
        .filter(|arg| !arg.starts_with("--message-format="))
        .for_each(|arg| {
            cargo_direct_args.push(arg);
        });

    env::args()
        .skip(1)
        .skip_while(|arg| arg != "--")
        .skip(1)
        .for_each(|arg| {
            rustc_args.push(arg);
        });

    let forbidden_args = [
        "--bins",
        "--examples",
        "--example",
        "--tests",
        "--test",
        "--benches",
        "--bench",
        "--all-targets",
    ];
    for arg in &cargo_direct_args {
        if forbidden_args.contains(&arg.as_str()) {
            eprintln!("The argument {arg} is invalid for use on the taint checker. To select a target use --lib or --bin {{name}}");
            return;
        }
    }

    let target_directory = tempfile::tempdir().unwrap();
    let target_directory_string = if let Some(path) = target_directory.path().to_str() {
        path
    } else {
        panic!("Unable to gen temp directory");
    };

    println!("Target dir {}", target_directory_string);

    let mut command = Command::new("cargo")
        .arg("build")
        .arg("--message-format=json-render-diagnostics")
        .arg("--target-dir")
        .arg(target_directory_string)
        .arg("--build-plan")
        .arg("-Z")
        .arg("unstable-options")
        .args(&cargo_direct_args)
        .arg("--")
        .args(&rustc_args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = std::io::BufReader::new(command.stdout.take().unwrap());

    let output = command.wait().expect("Couldn't get cargo's exit status");
    if !output.success() {
        eprintln!("Compile error. Run cargo to check where the compile error happened.");
        return;
    }

    let mut build_plan_text = String::default();
    if let Err(e) = reader.read_to_string(&mut build_plan_text) {
        eprintln!("Error reading cargo output: {}", e);
        return;
    }

    let build_plan = build_plan_text
        .lines()
        .filter(|line| line.contains("invocations"))
        .collect::<Vec<&str>>();

    if build_plan.len() != 1 {
        eprintln!(
            "Error parsing build plan. The build plan contains {} lines",
            build_plan.len()
        );
        return;
    }

    let mut build_plan: BuildPlan = match serde_json::from_str(build_plan[0]) {
        Ok(build_plan) => build_plan,
        Err(e) => {
            eprintln!("Error parsing build plan: {}", e);
            return;
        }
    };

    for invocation in build_plan.invocations.iter_mut() {
        if let CompileMode::Build = invocation.compile_mode {
            invocation.args.retain(|arg| !arg.starts_with("--json="));
        }
    }

    match cargo::execute_build_plan(build_plan) {
        Ok(_) => println!("Build succeeded"),
        Err(e) => eprintln!("Build failed: {}", e),
    }

    drop(target_directory);
}
