extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_session::{config::ErrorOutputType, EarlyDiagCtxt};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub fn init_hooks() {
    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::install_ice_hook("https://github.com/0xCCF4/UntrustedValue/issues", |_| ());
    rustc_driver::init_rustc_env_logger(&early_dcx);
    init_tracing();
}

fn init_tracing() {
    if let Ok(filter) = EnvFilter::try_from_env("TAINT_LOG") {
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::ENTER)
            .with_env_filter(filter)
            .without_time()
            .init();
    }
}

/*
pub struct CwdFileLoader {
    cwd: PathBuf,
}


impl CwdFileLoader {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
    pub fn transform_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.cwd.join(path)
    }
}

impl rustc_span::source_map::FileLoader for CwdFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        self.transform_path(path).exists()
    }
    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        let mut text = String::default();
        File::open(self.transform_path(path))?.read_to_string(&mut text)?;
        Ok(text)
    }
    fn read_binary_file(&self, path: &Path) -> std::io::Result<Arc<[u8]>> {
        let mut data = Vec::default();
        File::open(self.transform_path(path))?.read_to_end(&mut data)?;
        Ok(Arc::from(data.into_boxed_slice()))
    }
}
    */

pub fn run_compiler(
    mut args: Vec<String>,
    callbacks: &mut (dyn rustc_driver::Callbacks + Send),
) -> anyhow::Result<()> {
    if let Some(sysroot) = compile_time_sysroot() {
        let sysroot_flag = "--sysroot";
        if !args.iter().any(|e| e == sysroot_flag) {
            args.push(sysroot_flag.to_owned());
            args.push(sysroot);
        }
    }

    // let cwd_loader: Box<(dyn rustc_span::source_map::FileLoader + Send + std::marker::Sync + 'static)> = Box::new(CwdFileLoader::new(cwd));

    let exit_code = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::RunCompiler::new(&args, callbacks)
            //run_compiler.set_file_loader(Some(cwd_loader));
            //.set_using_internal_features(Arc::clone(&using_internal_features))
            .run()
    });

    if exit_code == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Compiler failed with exit code {}",
            exit_code
        ))
    }
}

#[allow(clippy::option_env_unwrap)]
fn compile_time_sysroot() -> Option<String> {
    if option_env!("RUSTC_STAGE").is_some() {
        None
    } else {
        let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
        let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
        Some(match (home, toolchain) {
            (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
            _ => option_env!("RUST_SYSROOT")
                .expect("To build this without rustup, set the RUST_SYSROOT env var at build time")
                .to_owned(),
        })
    }
}
