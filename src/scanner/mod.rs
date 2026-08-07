//! Directory walking, filtering, and file-combining logic.

mod collect;
mod filter;
mod lang;
mod process;
mod scan;
mod structure;
mod utils;
mod walker;

pub use scan::scan;
