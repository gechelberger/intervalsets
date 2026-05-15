//! UI tests for compile-error paths. Gated behind the
//! `INTERVALSETS_UI_TESTS` env var so rustc-diagnostic drift doesn't
//! break CI runs on toolchains other than the pinned one.
//!
//! Run from the workspace root (or use the `just test-ui` recipe):
//!
//! ```ignore
//! INTERVALSETS_UI_TESTS=1 cargo test -p intervalsets-macros --test ui
//! ```
//!
//! Regenerate snapshots after intentional message changes:
//!
//! ```ignore
//! INTERVALSETS_UI_TESTS=1 TRYBUILD=overwrite \
//!     cargo test -p intervalsets-macros --test ui
//! ```

#[test]
fn ui() {
    if std::env::var_os("INTERVALSETS_UI_TESTS").is_none() {
        eprintln!(
            "skipping ui tests: set INTERVALSETS_UI_TESTS=1 to enable \
             (pinned to a single toolchain to avoid stderr drift)"
        );
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
