//! Support for tracing

use std::env::var_os;
use tracing_subscriber::fmt::format::FmtSpan;

/// Initialize the tracing
pub fn init(env_filter: impl Into<tracing_subscriber::EnvFilter>) {
  let event_filter = eval_event_filter();

  let _ = tracing_subscriber::FmtSubscriber::builder()
    .with_env_filter(env_filter)
    .with_span_events(event_filter)
    .with_test_writer()
    .try_init();
}

fn eval_event_filter() -> FmtSpan {
  match var_os("RUST_LOG_SPAN_EVENTS") {
    Some(mut value) => {
      value.make_ascii_lowercase();
      let value = value
        .to_str()
        .expect("test-log: RUST_LOG_SPAN_EVENTS must be valid UTF-8");
      value
        .split(",")
        .map(|filter| match filter.trim() {
          "new" => FmtSpan::NEW,
          "enter" => FmtSpan::ENTER,
          "exit" => FmtSpan::EXIT,
          "close" => FmtSpan::CLOSE,
          "active" => FmtSpan::ACTIVE,
          "full" => FmtSpan::FULL,
          _ => panic!(
            "test-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                  For example: `active` or `new,close`\n\t\
                  Supported filters: new, enter, exit, close, active, full\n\t\
                  Got: {}",
            value
          ),
        })
        .fold(FmtSpan::NONE, |acc, filter| filter | acc)
    },
    None => FmtSpan::NONE,
  }
}
