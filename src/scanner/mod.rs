//! Directory walking, filtering, and file-combining logic.
//!
//! Performance-sensitive by design: the directory is walked exactly once,
//! in parallel (via [`ignore::WalkParallel`]); every file is then read and
//! UTF-8-validated in parallel too (`rayon` + `simdutf8`); and everything is
//! written through one [`std::io::BufWriter`]. See each submodule's docs
//! for the reasoning behind individual choices (e.g. why the output-file
//! identity check in `filter` avoids `canonicalize`).

mod collect;
mod filter;
mod lang;
mod process;
mod scan;
mod structure;
mod utils;
mod walker;

pub use scan::scan;
