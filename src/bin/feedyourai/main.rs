//! `feedyourai` binary entry point. See `app.rs` for the actual CLI logic,
//! which is shared with the `fyai` binary via `#[path]`.

#[path = "app.rs"]
mod app;

fn main() -> color_eyre::eyre::Result<()> {
    app::run()
}
