Unreleased
----------
- Introduced support for `RUST_LOG_SPAN_EVENTS` environment variable
  that can be used to configure emitting of synthetic trace events
- Updated documentation to include wrapping of other attributes
- Bumped minimum supported Rust version to `1.45`


0.2.5
-----
- Eliminated emitting of `-> ()` constructs in test function signatures


0.2.4
-----
- Eliminated need for emitting of `::f` test function
- Excluded unnecessary files from being contained in release bundle


0.2.3
-----
- Initialize `tracing` globally instead of individually for the run time
  of each test
- Bumped minimum supported Rust version to `1.42`


0.2.2
-----
- Added support for initializing `tracing` infrastructure
  - Introduced `log` (enabled by default) and `trace` features (disabled
    by default)
- Dropped `env_logger` dependency


0.2.1
-----
- Relicensed project under terms of `Apache-2.0 OR MIT`


0.2.0
-----
- Added support for providing inner `#[test]` attribute
- Bumped minimum required Rust version to `1.39.0`


0.1.1
-----
- Updated `README.md` with instructions on how to retrieve test output
  and change log level
- Bumped `env_logger` dependency to `0.7`
- Bumped `syn` dependency to `1.0`
- Bumped `quote` dependency to `1.0`
- Bumped `proc-macro` dependency to `1.0`


0.1.0
-----
- Initial release
