// Copyright (C) 2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! A test verifying that log/trace messages are actually emitted.
//!
//! Target functions (marked `#[ignore]`) emit known marker strings and
//! are invoked by rerunning the binary with the target test name.

#![cfg(any(feature = "log", feature = "trace"))]

use std::env;
use std::process::Command;
use std::process::Output;


#[cfg(feature = "log")]
const LOG_INFO: &str = "MARKER_LOG_INFO_abc123";
#[cfg(feature = "log")]
const LOG_ERROR: &str = "MARKER_LOG_ERROR_stu901";
#[cfg(feature = "log")]
const LOG_WARN: &str = "MARKER_LOG_WARN_vwx234";
#[cfg(feature = "log")]
const LOG_INFO_LEVELS: &str = "MARKER_LOG_INFO_yza567";
#[cfg(feature = "log")]
const LOG_DEBUG: &str = "MARKER_LOG_DEBUG_bcd890";

#[cfg(feature = "trace")]
const TRACE_INFO: &str = "MARKER_TRACE_INFO_def456";
#[cfg(feature = "trace")]
const TRACE_ERROR: &str = "MARKER_TRACE_ERROR_ghi789";
#[cfg(feature = "trace")]
const TRACE_WARN: &str = "MARKER_TRACE_WARN_jkl012";
#[cfg(feature = "trace")]
const TRACE_INFO_LEVELS: &str = "MARKER_TRACE_INFO_mno345";
#[cfg(feature = "trace")]
const TRACE_DEBUG: &str = "MARKER_TRACE_DEBUG_pqr678";


/// Run the ignored test with name `name` and capture its stderr output.
fn run_target(name: &str, extra_env: &[(&str, &str)]) -> String {
  let exe = env::current_exe().expect("failed to determine test binary path");
  let mut cmd = Command::new(exe);
  cmd
    .args(["--ignored", "--exact", name, "--nocapture"])
    .env("NO_COLOR", "1")
    .env_remove("RUST_LOG_SPAN_EVENTS");

  if !extra_env.iter().any(|(k, _)| *k == "RUST_LOG") {
    cmd.env("RUST_LOG", "info");
  }
  for (k, v) in extra_env {
    cmd.env(k, v);
  }

  let Output {
    status,
    stdout: _,
    stderr,
  } = cmd.output().expect("failed to execute test binary");

  let stderr = String::from_utf8(stderr).expect("stderr is not UTF-8");
  assert!(status.success(), "subprocess `{name}` failed:\n{stderr}",);
  stderr
}


#[cfg(feature = "log")]
#[ignore = "target for output verification"]
#[test_log::test]
fn emit_log_info() {
  logging::info!("{LOG_INFO}");
}

#[cfg(feature = "log")]
#[ignore = "target for output verification"]
#[test_log::test]
fn emit_log_levels() {
  logging::error!("{LOG_ERROR}");
  logging::warn!("{LOG_WARN}");
  logging::info!("{LOG_INFO_LEVELS}");
  logging::debug!("{LOG_DEBUG}");
}

#[cfg(feature = "trace")]
#[ignore = "target for output verification"]
#[test_log::test]
fn emit_trace_info() {
  tracing::info!("{TRACE_INFO}");
}

#[cfg(feature = "trace")]
#[ignore = "target for output verification"]
#[test_log::test]
fn emit_trace_levels() {
  tracing::error!("{TRACE_ERROR}");
  tracing::warn!("{TRACE_WARN}");
  tracing::info!("{TRACE_INFO_LEVELS}");
  tracing::debug!("{TRACE_DEBUG}");
}


/// Check that the `log` backend emits log messages.
#[cfg(feature = "log")]
#[test]
fn log_emits_output() {
  let output = run_target("emit_log_info", &[]);
  assert!(
    output.contains(LOG_INFO),
    "expected log output not found in stderr:\n{output}",
  );
}

/// Check that the `tracing` backend emits log messages.
#[cfg(feature = "trace")]
#[test]
fn trace_emits_output() {
  let output = run_target("emit_trace_info", &[]);
  assert!(
    output.contains(TRACE_INFO),
    "expected trace output not found in stderr:\n{output}",
  );
}

/// Verify that log levels are honored as expected.
#[cfg(feature = "log")]
#[test]
fn log_level_filtering() {
  let stderr = run_target("emit_log_levels", &[("RUST_LOG", "info")]);
  assert!(stderr.contains(LOG_ERROR), "missing error");
  assert!(stderr.contains(LOG_WARN), "missing warn");
  assert!(stderr.contains(LOG_INFO_LEVELS), "missing info");
  assert!(!stderr.contains(LOG_DEBUG), "debug should be filtered out");

  let output = run_target("emit_log_levels", &[("RUST_LOG", "debug")]);
  assert!(output.contains(LOG_ERROR), "missing error");
  assert!(output.contains(LOG_WARN), "missing warn");
  assert!(output.contains(LOG_INFO_LEVELS), "missing info");
  assert!(output.contains(LOG_DEBUG), "missing debug");
}

/// Test that log levels are honored for the `tracing` backend.
#[cfg(feature = "trace")]
#[test]
fn trace_level_filtering() {
  let output = run_target("emit_trace_levels", &[("RUST_LOG", "info")]);
  assert!(output.contains(TRACE_ERROR), "missing error");
  assert!(output.contains(TRACE_WARN), "missing warn");
  assert!(output.contains(TRACE_INFO_LEVELS), "missing info");
  assert!(
    !output.contains(TRACE_DEBUG),
    "debug should be filtered out"
  );

  let stderr = run_target("emit_trace_levels", &[("RUST_LOG", "debug")]);
  assert!(stderr.contains(TRACE_ERROR), "missing error");
  assert!(stderr.contains(TRACE_WARN), "missing warn");
  assert!(stderr.contains(TRACE_INFO_LEVELS), "missing info");
  assert!(stderr.contains(TRACE_DEBUG), "missing debug");
}
