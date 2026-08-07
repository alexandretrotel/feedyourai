//! `fyai` binary entry point: a short-name alias for `feedyourai`. Reuses
//! `feedyourai`'s `app.rs` via `#[path]` so the two binaries share one
//! implementation with no runtime indirection.

#[path = "../feedyourai/app.rs"]
mod app;

fn main() -> color_eyre::eyre::Result<()> {
    app::run()
}
