use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub(crate) fn site_fingerprint(root: &Path) -> io::Result<u64> {
    let mut paths = Vec::new();
    for name in ["pages", "layout", "assets"] {
        let path = root.join(name);
        if path.exists() {
            collect_paths(&path, &mut paths)?;
        }
    }
    paths.sort();

    let mut hasher = DefaultHasher::new();
    for path in paths {
        path.strip_prefix(root).unwrap_or(&path).hash(&mut hasher);
        let metadata = fs::symlink_metadata(&path)?;
        metadata.is_dir().hash(&mut hasher);
        metadata.is_file().hash(&mut hasher);
        metadata.len().hash(&mut hasher);
        metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .hash(&mut hasher);
    }

    Ok(hasher.finish())
}

fn collect_paths(path: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    paths.push(path.to_owned());
    if fs::symlink_metadata(path)?.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_paths(&entry?.path(), paths)?;
        }
    }
    Ok(())
}
