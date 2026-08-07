//! Directory walking, filtering, and file-combining logic.

mod filter;
mod lang;
mod process;
mod structure;
mod utils;
mod walker;

pub use process::process_files;
pub use structure::get_directory_structure;
