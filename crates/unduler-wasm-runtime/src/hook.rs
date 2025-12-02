//! WASM hook plugin wrapper with action execution.

use std::path::{Path, PathBuf};
use std::process::Command;

use wasmtime::Store;
use wasmtime::component::{Component, Linker};

use crate::{WasmEngine, WasmError, WasmResult};

// Generate bindings from WIT
wasmtime::component::bindgen!({
    world: "unduler-hook",
    path: "../unduler-plugin/wit",
});

/// Whitelisted commands that hooks are allowed to execute.
const ALLOWED_COMMANDS: &[&str] = &["cargo", "npm", "yarn", "pnpm", "gh", "git"];

/// Store state for hook plugins.
pub struct HookState {
    /// Working directory for the hook (repository root).
    #[allow(dead_code)]
    workdir: PathBuf,
}

impl HookState {
    /// Creates a new hook state.
    fn new(workdir: PathBuf) -> Self {
        Self { workdir }
    }
}

/// Result of executing a command action.
#[derive(Debug)]
pub struct CommandOutput {
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// Result of processing hook actions.
#[derive(Debug, Default)]
pub struct ActionResults {
    /// Command outputs in order of execution.
    pub command_outputs: Vec<CommandOutput>,
    /// Files written.
    pub files_written: Vec<PathBuf>,
    /// Any errors that occurred.
    pub errors: Vec<String>,
}

impl ActionResults {
    /// Returns true if all actions succeeded.
    #[must_use]
    pub fn success(&self) -> bool {
        self.errors.is_empty() && self.command_outputs.iter().all(|o| o.exit_code == 0)
    }
}

/// WASM hook plugin wrapper.
pub struct WasmHook {
    store: Store<HookState>,
    instance: UndulerHook,
    workdir: PathBuf,
}

impl WasmHook {
    /// Creates a new WASM hook from a component with a working directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be instantiated.
    pub fn from_component(
        engine: &WasmEngine,
        component: &Component,
        workdir: PathBuf,
    ) -> WasmResult<Self> {
        let mut store = Store::new(engine.inner(), HookState::new(workdir.clone()));
        let linker = Linker::new(engine.inner());

        let instance = UndulerHook::instantiate(&mut store, component, &linker)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        Ok(Self {
            store,
            instance,
            workdir,
        })
    }

    /// Creates a new WASM hook from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be loaded or instantiated.
    pub fn from_file(engine: &WasmEngine, path: &Path, workdir: PathBuf) -> WasmResult<Self> {
        let component = engine.load_component(path)?;
        Self::from_component(engine, &component, workdir)
    }

    /// Returns the working directory for this hook.
    #[must_use]
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    /// Gets plugin information.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn info(&mut self) -> WasmResult<PluginInfo> {
        self.instance
            .unduler_plugin_hook()
            .call_info(&mut self.store)
            .map_err(|e| WasmError::FunctionCall {
                name: "info".to_string(),
                reason: e.to_string(),
            })
    }

    /// Called before version files are modified.
    /// Executes any actions returned by the hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn on_pre_bump(&mut self, ctx: &ReleaseContext) -> WasmResult<(HookResult, ActionResults)> {
        let result = self
            .instance
            .unduler_plugin_hook()
            .call_on_pre_bump(&mut self.store, ctx)
            .map_err(|e| WasmError::FunctionCall {
                name: "on_pre_bump".to_string(),
                reason: e.to_string(),
            })?;

        let actions = self.execute_actions(&result.actions);
        Ok((result, actions))
    }

    /// Called after version files are modified.
    /// Executes any actions returned by the hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn on_post_bump(
        &mut self,
        ctx: &ReleaseContext,
    ) -> WasmResult<(HookResult, ActionResults)> {
        let result = self
            .instance
            .unduler_plugin_hook()
            .call_on_post_bump(&mut self.store, ctx)
            .map_err(|e| WasmError::FunctionCall {
                name: "on_post_bump".to_string(),
                reason: e.to_string(),
            })?;

        let actions = self.execute_actions(&result.actions);
        Ok((result, actions))
    }

    /// Called before release commit.
    /// Executes any actions returned by the hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn on_pre_commit(
        &mut self,
        ctx: &ReleaseContext,
    ) -> WasmResult<(HookResult, ActionResults)> {
        let result = self
            .instance
            .unduler_plugin_hook()
            .call_on_pre_commit(&mut self.store, ctx)
            .map_err(|e| WasmError::FunctionCall {
                name: "on_pre_commit".to_string(),
                reason: e.to_string(),
            })?;

        let actions = self.execute_actions(&result.actions);
        Ok((result, actions))
    }

    /// Called before git tag.
    /// Executes any actions returned by the hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn on_pre_tag(&mut self, ctx: &ReleaseContext) -> WasmResult<(HookResult, ActionResults)> {
        let result = self
            .instance
            .unduler_plugin_hook()
            .call_on_pre_tag(&mut self.store, ctx)
            .map_err(|e| WasmError::FunctionCall {
                name: "on_pre_tag".to_string(),
                reason: e.to_string(),
            })?;

        let actions = self.execute_actions(&result.actions);
        Ok((result, actions))
    }

    /// Called after git tag.
    /// Executes any actions returned by the hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn on_post_tag(&mut self, ctx: &ReleaseContext) -> WasmResult<(HookResult, ActionResults)> {
        let result = self
            .instance
            .unduler_plugin_hook()
            .call_on_post_tag(&mut self.store, ctx)
            .map_err(|e| WasmError::FunctionCall {
                name: "on_post_tag".to_string(),
                reason: e.to_string(),
            })?;

        let actions = self.execute_actions(&result.actions);
        Ok((result, actions))
    }

    /// Executes a list of hook actions.
    fn execute_actions(&self, actions: &[HookAction]) -> ActionResults {
        let mut results = ActionResults::default();

        for action in actions {
            match action {
                HookAction::RunCommand(req) => match self.execute_command(req) {
                    Ok(output) => results.command_outputs.push(output),
                    Err(e) => results.errors.push(e),
                },
                HookAction::WriteFile(req) => match self.write_file(req) {
                    Ok(path) => results.files_written.push(path),
                    Err(e) => results.errors.push(e),
                },
                HookAction::LogMessage(req) => {
                    Self::log_message(req);
                }
            }
        }

        results
    }

    /// Executes a command action.
    fn execute_command(&self, req: &CommandRequest) -> Result<CommandOutput, String> {
        // Validate command is whitelisted
        if !ALLOWED_COMMANDS.contains(&req.command.as_str()) {
            return Err(format!(
                "command '{}' is not allowed. Allowed commands: {}",
                req.command,
                ALLOWED_COMMANDS.join(", ")
            ));
        }

        // Determine working directory
        let cwd = match &req.workdir {
            Some(dir) => {
                let path = PathBuf::from(dir);
                if path.is_absolute() {
                    path
                } else {
                    self.workdir.join(path)
                }
            }
            None => self.workdir.clone(),
        };

        tracing::debug!(
            "Running command: {} {:?} in {:?}",
            req.command,
            req.args,
            cwd
        );

        // Execute command
        let output = Command::new(&req.command)
            .args(&req.args)
            .current_dir(&cwd)
            .output()
            .map_err(|e| format!("failed to execute command '{}': {}", req.command, e))?;

        let result = CommandOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        };

        if result.exit_code != 0 {
            tracing::warn!(
                "Command '{}' exited with code {}: {}",
                req.command,
                result.exit_code,
                result.stderr
            );
        }

        Ok(result)
    }

    /// Writes a file action.
    fn write_file(&self, req: &FileWriteRequest) -> Result<PathBuf, String> {
        let path = self.resolve_path(&req.path)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create directory: {e}"))?;
        }

        std::fs::write(&path, &req.content)
            .map_err(|e| format!("failed to write file '{}': {e}", path.display()))?;

        tracing::debug!("Wrote file: {}", path.display());

        Ok(path)
    }

    /// Logs a message from the hook.
    fn log_message(req: &LogRequest) {
        match req.level {
            LogLevel::Trace => tracing::trace!("[plugin] {}", req.message),
            LogLevel::Debug => tracing::debug!("[plugin] {}", req.message),
            LogLevel::Info => tracing::info!("[plugin] {}", req.message),
            LogLevel::Warn => tracing::warn!("[plugin] {}", req.message),
            LogLevel::Error => tracing::error!("[plugin] {}", req.message),
        }
    }

    /// Resolves a path relative to the working directory.
    /// Prevents path traversal attacks.
    fn resolve_path(&self, path: &str) -> Result<PathBuf, String> {
        let path_buf = PathBuf::from(path);

        // If absolute, verify it's within workdir
        let resolved = if path_buf.is_absolute() {
            path_buf
        } else {
            self.workdir.join(&path_buf)
        };

        // For new files, we can't canonicalize yet, so just check the resolved path
        // doesn't try to escape with ..
        let normalized = normalize_path(&resolved);

        let workdir_normalized = normalize_path(&self.workdir);

        if !normalized.starts_with(&workdir_normalized) {
            return Err(format!("path '{path}' is outside the repository"));
        }

        Ok(resolved)
    }
}

/// Normalizes a path by resolving . and .. components without requiring the path to exist.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }

    components.iter().collect()
}

// Re-export generated types
pub use unduler::plugin::types::{
    CommandRequest, FileWriteRequest, HookAction, HookResult, LogLevel, LogRequest, PluginInfo,
    PluginType, ReleaseContext,
};

/// Returns the list of allowed commands for hooks.
#[must_use]
pub fn allowed_commands() -> &'static [&'static str] {
    ALLOWED_COMMANDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_commands() {
        let commands = allowed_commands();
        assert!(commands.contains(&"cargo"));
        assert!(commands.contains(&"npm"));
        assert!(commands.contains(&"yarn"));
        assert!(commands.contains(&"pnpm"));
        assert!(commands.contains(&"gh"));
        assert!(commands.contains(&"git"));
        assert!(!commands.contains(&"rm"));
        assert!(!commands.contains(&"sudo"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/foo/bar/../baz")),
            PathBuf::from("/foo/baz")
        );
        assert_eq!(
            normalize_path(Path::new("/foo/./bar")),
            PathBuf::from("/foo/bar")
        );
        assert_eq!(
            normalize_path(Path::new("/foo/bar/../../baz")),
            PathBuf::from("/baz")
        );
    }
}
