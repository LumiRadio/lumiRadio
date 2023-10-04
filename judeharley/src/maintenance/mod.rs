use crate::prelude::*;
use std::path::{Path, PathBuf};

pub mod indexing;

pub fn rewrite_music_path(path: &Path, music_path: &Path) -> Result<PathBuf> {
    Ok(Path::new("/music").join(path.strip_prefix(music_path)?))
}
