use std::env;
use std::path::PathBuf;

use super::build_plan::Invocation;

pub struct InvocationEnvironment {
    vars: Vec<(String, String)>,
    override_vars: Vec<(String, String)>,
    cwd: PathBuf,
}

impl InvocationEnvironment {
    pub fn enter(environment: &Invocation) -> InvocationEnvironment {
        let mut result = InvocationEnvironment {
            vars: Vec::with_capacity(environment.env.len()),
            cwd: env::current_dir().unwrap(),
            override_vars: Vec::with_capacity(environment.env.len()),
        };

        for (key, value) in environment.env.iter() {
            result.vars.push((key.to_owned(), value.to_owned()));
        }

        if let Some(cwd) = &environment.cwd {
            env::set_current_dir(cwd).unwrap();
        }

        for (key, value) in environment.env.iter() {
            env::set_var(key, value);
            result.override_vars.push((key.clone(), value.clone()));
        }

        result
    }
}

impl Drop for InvocationEnvironment {
    fn drop(&mut self) {
        for (key, _) in self.override_vars.iter() {
            env::remove_var(key);
        }

        for (key, value) in self.vars.iter() {
            env::set_var(key, value);
        }

        env::set_current_dir(&self.cwd).unwrap();
    }
}
