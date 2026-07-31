use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn core(command: &str) -> Command {
    let mut process = Command::new(env!("CARGO_BIN_EXE_cliptown-core"));
    process
        .env("CLIPTOWN_COMMAND", command)
        .env("CLIPTOWN_POSITIONALS", "[]")
        .env("CLIPTOWN_ENDPOINT", "http://localhost:3000")
        .env_remove("CLIPTOWN_FILE")
        .env_remove("CLIPTOWN_FROM_CLIPBOARD")
        .env_remove("CLIPTOWN_OUTPUT_JSON")
        .env_remove("CLIPTOWN_PIN_CLIP")
        .env_remove("CLIPTOWN_STDIN");
    process
}

fn run_with_stdin(process: &mut Command, input: &str) -> Output {
    let mut child = process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cliptown-core");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(input.as_bytes())
        .expect("write child stdin");
    child.wait_with_output().expect("collect command output")
}

#[test]
fn stdin_payload_is_not_repeated_in_machine_output() {
    let sensitive = "private clipboard payload";
    let output = run_with_stdin(
        core("clip add")
            .env("CLIPTOWN_STDIN", "true")
            .env("CLIPTOWN_OUTPUT_JSON", "true"),
        sensitive,
    );

    assert!(output.status.success());
    let stdout: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON success envelope");
    assert_eq!(stdout["schema_version"], 1);
    assert_eq!(stdout["ok"], true);
    assert_eq!(stdout["result"]["command"], "clip.add");
    assert_eq!(stdout["result"]["source"], "stdin");
    assert_eq!(stdout["result"]["byte_count"], sensitive.len());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(sensitive));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(sensitive));
}

#[test]
fn file_payload_is_not_repeated_in_output() {
    let sensitive = "file-only clipboard payload";
    let mut file = tempfile::NamedTempFile::new().expect("temporary clip file");
    file.write_all(sensitive.as_bytes())
        .expect("write clip file");

    let output = core("clip add")
        .env("CLIPTOWN_FILE", file.path())
        .output()
        .expect("run file input");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(&sensitive.len().to_string()));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(sensitive));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(sensitive));
}

#[test]
fn multiple_sources_use_stable_json_error_and_usage_exit_code() {
    let output = run_with_stdin(
        core("clip add")
            .env("CLIPTOWN_STDIN", "true")
            .env("CLIPTOWN_FILE", "unused.txt")
            .env("CLIPTOWN_OUTPUT_JSON", "true"),
        "must not be read",
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error envelope");
    assert_eq!(stderr["schema_version"], 1);
    assert_eq!(stderr["ok"], false);
    assert_eq!(stderr["error"]["code"], "invalid_arguments");
}

#[test]
fn configuration_errors_have_a_distinct_exit_code() {
    let output = core("doctor")
        .env("CLIPTOWN_ENDPOINT", "http://api.example.test")
        .env("CLIPTOWN_OUTPUT_JSON", "true")
        .output()
        .expect("run invalid configuration");

    assert_eq!(output.status.code(), Some(3));
    let stderr: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error envelope");
    assert_eq!(stderr["error"]["code"], "invalid_configuration");
}

#[test]
fn missing_files_have_a_distinct_io_exit_code() {
    let output = core("clip add")
        .env("CLIPTOWN_FILE", "this-file-does-not-exist")
        .env("CLIPTOWN_OUTPUT_JSON", "true")
        .output()
        .expect("run missing file input");

    assert_eq!(output.status.code(), Some(5));
    let stderr: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error envelope");
    assert_eq!(stderr["error"]["code"], "io_error");
}

#[test]
fn flags_contract_exposes_stdin_but_not_inline_clip_text() {
    let contract = include_str!("../.cli-flags.toml");
    assert!(contract.contains("[commands.clip.commands.add.flags.stdin]"));
    assert!(!contract.contains("[commands.clip.commands.add.flags.text]"));
    assert!(!contract.contains("aliases = [\"text\"]"));
}

#[test]
fn json_envelope_schema_is_versioned_and_enumerates_error_codes() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/cli-envelope.schema.json"))
            .expect("valid CLI envelope JSON Schema");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["properties"]["schema_version"]["const"], 1);
    assert_eq!(schema["properties"]["result"]["$ref"], "#/$defs/result");
    assert_eq!(schema["properties"]["error"]["$ref"], "#/$defs/error");
    assert_eq!(
        schema["$defs"]["error"]["properties"]["code"]["enum"],
        serde_json::json!([
            "invalid_arguments",
            "invalid_configuration",
            "clipboard_unavailable",
            "io_error",
            "client_error"
        ])
    );
}

#[test]
fn placeholder_commands_do_not_repeat_private_arguments() {
    let sensitive = "private search phrase";
    let output = core("clip search")
        .env("CLIPTOWN_QUERY", sensitive)
        .output()
        .expect("run search placeholder");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(sensitive));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(sensitive));
}
