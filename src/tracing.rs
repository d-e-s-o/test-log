//! Support for tracing

use std::env::var_os;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Default)]
pub struct TracingGuard {
  #[cfg(feature = "tracing-flame")]
  _flame: Option<tracing_flame::FlushGuard<BufWriter<File>>>,
}

/// Initialize the tracing
pub fn init(name: &str, env_filter: impl Into<tracing_subscriber::EnvFilter>) -> TracingGuard {
  let env_filter = env_filter.into();
  let event_filter = eval_event_filter();

  let fmt = tracing_subscriber::fmt::layer()
    .with_ansi(true)
    .with_span_events(event_filter)
    .with_level(true)
    .with_test_writer()
    .compact();

  let layered = tracing_subscriber::registry().with(env_filter).with(fmt);

  #[cfg(feature = "tracing-flame")]
  {
    return match std::env::var("TEST_LOG_FLAMES").ok() {
      Some(base) => {
        let path = format!("{base}/{name}.folded");
        let path = Path::new(&path);

        // ensure we have the parent dir
        if let Some(parent) = path.parent() {
          let _ = std::fs::create_dir_all(parent);
        }

        let (flame, guard) =
          tracing_flame::FlameLayer::with_file(path).expect("Unable to initialize tracing-flame");

        let _ = layered.with(flame).try_init();

        TracingGuard {
          _flame: Some(guard),
        }
      },
      None => {
        let _ = layered.try_init();
        TracingGuard::default()
      },
    };
  }

  #[cfg(not(feature = "tracing-flame"))]
  {
    let layered = layered.with(env_filter).with(fmt).try_init();
    TracingGuard::default()
  }
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
