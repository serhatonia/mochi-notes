//! Markdown export: a single note through a native save dialog, or the whole
//! library into a timestamped folder.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::Local;
use uuid::Uuid;

use crate::models::{Folder, Note, Tag};
use crate::utils::text;

fn tag_names(tag_ids: &[Uuid], tags: &[Tag]) -> Vec<String> {
    let lookup: HashMap<Uuid, &Tag> = tags.iter().map(|tag| (tag.id, tag)).collect();
    tag_ids
        .iter()
        .filter_map(|id| lookup.get(id).map(|tag| tag.name.clone()))
        .collect()
}

fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Renders a note as a self-describing Markdown document.
pub fn note_to_markdown(note: &Note, folder: Option<&Folder>, tags: &[Tag]) -> String {
    let title = note.display_title();
    let names = tag_names(&note.tags, tags);
    let mut output = String::new();

    output.push_str("---\n");
    output.push_str(&format!("title: \"{}\"\n", yaml_escape(&title)));
    output.push_str(&format!(
        "created: {}\n",
        note.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    ));
    output.push_str(&format!(
        "updated: {}\n",
        note.updated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    ));
    if let Some(folder) = folder {
        output.push_str(&format!("folder: \"{}\"\n", yaml_escape(&folder.name)));
    }
    if !names.is_empty() {
        output.push_str(&format!(
            "tags: [{}]\n",
            names
                .iter()
                .map(|name| format!("\"{}\"", yaml_escape(name)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    output.push_str(&format!("favorite: {}\n", note.favorite));
    output.push_str("---\n\n");
    output.push_str(&format!("# {title}\n\n"));

    let body = note.content.trim();
    if body.is_empty() {
        output.push_str("_This note is waiting for its first words._\n");
    } else {
        output.push_str(body);
        output.push('\n');
    }

    output
}

/// Opens a native "Save as…" dialog and writes the note as Markdown.
pub fn save_note_dialog(
    note: &Note,
    folder: Option<&Folder>,
    tags: &[Tag],
    exports_dir: &std::path::Path,
) -> Result<PathBuf, String> {
    let markdown = note_to_markdown(note, folder, tags);
    let filename = format!("{}.md", text::sanitize_filename(&note.display_title()));

    let _ = fs::create_dir_all(exports_dir);

    let file = rfd::FileDialog::new()
        .set_title("Export note as Markdown")
        .set_directory(exports_dir)
        .set_file_name(&filename)
        .add_filter("Markdown", &["md"])
        .save_file()
        .ok_or_else(|| "Export cancelled".to_string())?;

    fs::write(&file, markdown).map_err(|error| error.to_string())?;
    Ok(file)
}

/// Exports every active note into a timestamped folder.
pub fn export_all(
    notes: &[Note],
    folders: &[Folder],
    tags: &[Tag],
    exports_dir: &std::path::Path,
) -> Result<PathBuf, String> {
    let stamp = Local::now().format("%Y-%m-%d %H%M");
    let directory = exports_dir.join(format!("Mochi Export {stamp}"));
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let folder_lookup: HashMap<Uuid, &Folder> =
        folders.iter().map(|folder| (folder.id, folder)).collect();

    let mut used_names: HashMap<String, usize> = HashMap::new();
    let mut index = String::from("# Mochi Notes export\n\n");
    index.push_str(&format!(
        "Exported {} on {}.\n\n",
        text::plural(notes.len(), "note", "notes"),
        Local::now().format("%B %-d, %Y at %H:%M")
    ));

    let mut written = 0;
    for note in notes.iter().filter(|note| !note.trashed) {
        let mut name = text::sanitize_filename(&note.display_title());
        let count = used_names.entry(name.to_lowercase()).or_insert(0);
        *count += 1;
        if *count > 1 {
            name = format!("{name} ({count})");
        }

        let folder = note.folder_id.and_then(|id| folder_lookup.get(&id).copied());
        let markdown = note_to_markdown(note, folder, tags);
        let path = directory.join(format!("{name}.md"));

        if fs::write(&path, markdown).is_ok() {
            written += 1;
            index.push_str(&format!("- [{}]({}.md)\n", note.display_title(), name));
        }
    }

    let _ = fs::write(directory.join("index.md"), index);

    if written == 0 {
        return Err("There were no notes to export".into());
    }

    Ok(directory)
}
