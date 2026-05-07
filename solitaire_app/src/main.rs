//! Desktop entry point for `solitaire_app`.
//!
//! The body of the app lives in `lib.rs` so cargo-apk can package the
//! same code into an Android `cdylib`. This shim is the desktop /
//! `cargo run` path — it just delegates to [`solitaire_app::run`].

fn main() {
    solitaire_app::run();
}
