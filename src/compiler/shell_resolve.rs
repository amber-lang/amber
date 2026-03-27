use super::AmberCompiler;
use crate::utils::ShellType;
use std::env;
use std::process::Command;

impl AmberCompiler {
    // On Windows, look for `bash.exe` on PATH.
    #[cfg(windows)]
    pub fn find_shell(_target: Option<ShellType>) -> Option<Command> {
        if let Some(paths) = env::var_os("PATH") {
            for path in env::split_paths(&paths) {
                let path = path.join("bash.exe");
                if path.exists() {
                    return Some(Command::new(path));
                }
            }
        }
        None
    }

    /// Return bash command. In some situations, mainly for testing purposes, this can return a command, for example, containerized execution which is not bash but behaves like bash.
    #[cfg(not(windows))]
    pub fn find_shell(target: Option<ShellType>) -> Option<Command> {
        if env::var("AMBER_TEST_STRATEGY").is_ok_and(|value| value == "docker") {
            let mut command = Command::new("docker");
            let args_string = env::var("AMBER_TEST_ARGS")
                .expect("Please pass docker arguments in AMBER_TEST_ARGS environment variable.");
            let mut args: Vec<&str> = args_string.split_whitespace().collect();
            if args.first() == Some(&"exec") && !args.contains(&"-i") {
                // `docker exec` needs `-i` to pass piped stdin through to interactive Amber input tests.
                args.insert(1, "-i");
            }
            command.args(args);
            Some(command)
        } else {
            Self::runtime_shell_command(env::var("AMBER_SHELL").ok(), target).map(Command::new)
        }
    }

    pub fn resolve_target_shell(target: Option<ShellType>) -> ShellType {
        // Allow test runs to force a compiler target that differs from the runtime shell.
        if target.is_none() {
            if let Some(target) = env::var("AMBER_TEST_TARGET")
                .ok()
                .map(|target| target.parse().unwrap())
            {
                return target;
            }
        }
        if let Some(target) = target {
            return target;
        }
        if let Ok(shell) = env::var("AMBER_SHELL") {
            if let Some(target) = Self::target_from_shell_path(&shell) {
                return target;
            }
        }

        #[cfg(not(windows))]
        {
            Self::find_runtime_shell_name()
                .as_deref()
                .and_then(Self::target_from_shell_path)
                .unwrap_or(ShellType::BashModern)
        }

        #[cfg(windows)]
        {
            ShellType::BashModern
        }
    }

    pub(crate) fn target_from_shell_path(shell: &str) -> Option<ShellType> {
        let shell = shell.to_ascii_lowercase();
        if shell.contains("zsh") {
            Some(ShellType::Zsh)
        } else if shell.contains("ksh") {
            Some(ShellType::Ksh)
        } else if shell.contains("bash") {
            Some(ShellType::BashModern)
        } else {
            None
        }
    }

    #[cfg(not(windows))]
    fn find_runtime_shell_name() -> Option<String> {
        ["bash", "zsh", "ksh"]
            .into_iter()
            .find(|shell| {
                env::var_os("PATH").is_some_and(|paths| {
                    env::split_paths(&paths).any(|path| path.join(shell).is_file())
                })
            })
            .map(String::from)
    }

    #[cfg(not(windows))]
    pub(crate) fn runtime_shell_command(
        shell: Option<String>,
        target: Option<ShellType>,
    ) -> Option<String> {
        if let Some(shell) = shell {
            Some(shell)
        } else if let Some(target) = target {
            Some(target.family_name().to_string())
        } else {
            Self::find_runtime_shell_name()
        }
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolve_target_shell_uses_test_target_override_when_target_is_missing() {
        let _guard = env_lock().lock().unwrap();
        let previous_target = env::var("AMBER_TEST_TARGET").ok();
        let previous_strategy = env::var("AMBER_TEST_STRATEGY").ok();
        unsafe {
            env::set_var("AMBER_TEST_TARGET", "bash-3.2");
            env::set_var("AMBER_TEST_STRATEGY", "docker");
        }

        assert_eq!(
            AmberCompiler::resolve_target_shell(None),
            ShellType::BashLegacy
        );

        if let Some(previous) = previous_target {
            unsafe {
                env::set_var("AMBER_TEST_TARGET", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_TARGET");
            }
        }
        if let Some(previous) = previous_strategy {
            unsafe {
                env::set_var("AMBER_TEST_STRATEGY", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_STRATEGY");
            }
        }
    }

    #[test]
    fn resolve_target_shell_keeps_explicit_target_over_test_target_override() {
        let _guard = env_lock().lock().unwrap();
        let previous_target = env::var("AMBER_TEST_TARGET").ok();
        let previous_strategy = env::var("AMBER_TEST_STRATEGY").ok();
        unsafe {
            env::set_var("AMBER_TEST_TARGET", "bash-3.2");
            env::set_var("AMBER_TEST_STRATEGY", "docker");
        }

        assert_eq!(
            AmberCompiler::resolve_target_shell(Some(ShellType::Zsh)),
            ShellType::Zsh
        );

        if let Some(previous) = previous_target {
            unsafe {
                env::set_var("AMBER_TEST_TARGET", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_TARGET");
            }
        }
        if let Some(previous) = previous_strategy {
            unsafe {
                env::set_var("AMBER_TEST_STRATEGY", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_STRATEGY");
            }
        }
    }

    #[test]
    fn resolve_target_shell_uses_test_target_override_without_docker_strategy() {
        let _guard = env_lock().lock().unwrap();
        let previous_target = env::var("AMBER_TEST_TARGET").ok();
        let previous_strategy = env::var("AMBER_TEST_STRATEGY").ok();
        unsafe {
            env::set_var("AMBER_TEST_TARGET", "bash-3.2");
            env::remove_var("AMBER_TEST_STRATEGY");
        }

        assert_eq!(
            AmberCompiler::resolve_target_shell(None),
            ShellType::BashLegacy
        );

        if let Some(previous) = previous_target {
            unsafe {
                env::set_var("AMBER_TEST_TARGET", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_TARGET");
            }
        }
        if let Some(previous) = previous_strategy {
            unsafe {
                env::set_var("AMBER_TEST_STRATEGY", previous);
            }
        } else {
            unsafe {
                env::remove_var("AMBER_TEST_STRATEGY");
            }
        }
    }
}
