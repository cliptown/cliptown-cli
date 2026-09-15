use std::io;
use std::sync::OnceLock;

use ores_clis_core::{
    CliPolicy, ColorRole, EmitDisposition, EnvironmentHints, LogLevel, ProtocolEmitter,
    RuntimePolicy, StreamRole, TerminalState, paint, top_level_io,
};

use crate::error::CliError;

static RUNTIME: OnceLock<RuntimePolicy> = OnceLock::new();

pub fn install(runtime: RuntimePolicy) {
    let _ = RUNTIME.set(runtime);
}

#[must_use]
pub fn current() -> RuntimePolicy {
    RUNTIME.get().copied().unwrap_or_else(|| {
        CliPolicy::default().resolve(TerminalState::detect(), EnvironmentHints::detect())
    })
}

pub fn emit_result(json: bool, result: serde_json::Value, plain: String) -> Result<(), CliError> {
    let runtime = current();
    let stdout = io::stdout();
    let mut emitter = ProtocolEmitter::new(stdout.lock(), StreamRole::Primary);
    let write = if json {
        let record = serde_json::to_string(&serde_json::json!({
            "schema_version": 1,
            "ok": true,
            "result": result,
        }))
        .map_err(|error| CliError::Configuration(format!("cannot serialize output: {error}")))?;
        emitter.emit_primary_machine_record(&record)
    } else {
        emitter.emit_primary_human_line(&paint(runtime.color_stdout(), ColorRole::Info, plain))
    };
    match top_level_io(write)? {
        EmitDisposition::Written | EmitDisposition::ConsumerClosed => Ok(()),
    }
}

pub fn emit_error(error: &CliError, json: bool) {
    let runtime = current();
    if !runtime.allows_log(LogLevel::Error) {
        return;
    }
    let stderr = io::stderr();
    let mut emitter = ProtocolEmitter::new(stderr.lock(), StreamRole::Diagnostics);
    let line = if json {
        serde_json::json!({
            "schema_version": 1,
            "ok": false,
            "error": {
                "code": error.code(),
                "message": error.to_string(),
            }
        })
        .to_string()
    } else {
        paint(runtime.color_stderr(), ColorRole::Error, format!("cliptown: {error}"))
    };
    let _ = top_level_io(emitter.emit_diagnostic_line(&line));
}
