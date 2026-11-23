use std::path::PathBuf;

/// All locations used by Mochi Notes on disk.
#[derive(Debug, Clone)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub backups_dir: PathBuf,
    pub exports_dir: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let base = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        let data_dir = base.join("MochiNotes");

        Self {
            db_path: data_dir.join("mochi.db"),
            backups_dir: data_dir.join("backups"),
            exports_dir: data_dir.join("exports"),
            data_dir,
        }
    }
}
