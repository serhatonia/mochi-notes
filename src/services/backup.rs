//! Local, file-based backups of the SQLite database.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub modified: DateTime<Local>,
    pub size: u64,
}

/// Copies the database file into `backups_dir` and keeps only `keep` backups.
pub fn create_backup(db_path: &Path, backups_dir: &Path, keep: usize) -> Result<PathBuf, String> {
    fs::create_dir_all(backups_dir).map_err(|error| error.to_string())?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let mut target = backups_dir.join(format!("mochi-{stamp}.db"));
    let mut attempt = 2;
    while target.exists() {
        target = backups_dir.join(format!("mochi-{stamp}-{attempt}.db"));
        attempt += 1;
    }

    fs::copy(db_path, &target).map_err(|error| error.to_string())?;
    prune(backups_dir, keep);

    Ok(target)
}

pub fn list_backups(backups_dir: &Path) -> Vec<BackupInfo> {
    let Ok(entries) = fs::read_dir(backups_dir) else {
        return Vec::new();
    };

    let mut backups: Vec<BackupInfo> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("db") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            let modified = metadata
                .modified()
                .ok()
                .map(DateTime::<Local>::from)
                .unwrap_or_else(Local::now);

            Some(BackupInfo {
                path,
                modified,
                size: metadata.len(),
            })
        })
        .collect();

    backups.sort_by(|a, b| b.modified.cmp(&a.modified));
    backups
}

/// Removes the oldest backups, keeping `keep` files.
pub fn prune(backups_dir: &Path, keep: usize) {
    let backups = list_backups(backups_dir);
    for backup in backups.into_iter().skip(keep) {
        let _ = fs::remove_file(backup.path);
    }
}

pub fn latest_backup(backups_dir: &Path) -> Option<BackupInfo> {
    list_backups(backups_dir).into_iter().next()
}
