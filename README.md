# Mochi Notes

A cozy, pastel note-taking app for Windows — built entirely in Rust with Iced, SQLite, serde, uuid and chrono.

- Markdown editor with autosave and live preview
- Folders, tags, favorites, pinning, archive, trash and search
- Six pastel themes, keyboard shortcuts and drag & drop
- Local backups, Markdown export and a clickable mascot
- Everything stays on your machine in `%LOCALAPPDATA%\MochiNotes`

## Build

Requires the Rust MSVC toolchain and Visual Studio Build Tools.

```powershell
cargo build --release
```

The executable is written to `target\release\mochi-notes.exe`.

## Website

The landing page lives in [`site/`](site/index.html) — plain HTML + CSS, no JavaScript.

Open `site/index.html` in a browser to preview it. The download button points at
`site/downloads/MochiNotes.exe`, so copy a fresh build there before publishing:

```powershell
Copy-Item target\release\mochi-notes.exe site\downloads\MochiNotes.exe
```

## Publish with GitHub Pages

1. Create a public repository named `mochi-notes` on GitHub.
2. Initialize and push this folder:

   ```powershell
   git init
   git add .
   git commit -m "Mochi Notes v1.0"
   git branch -M main
   git remote add origin https://github.com/serhatonia/mochi-notes.git
   git push -u origin main
   ```

3. In the repository, go to **Settings → Pages**.
4. Set **Source** to `Deploy from a branch`, choose `main` and the `/docs`-style
   folder `site` (select **/site** from the folder dropdown), then save.
5. Your site will be live at `https://serhatonia.github.io/mochi-notes/`.

Keep `site/downloads/MochiNotes.exe` in sync whenever you ship a new build.

## Credits

Made with love by **Serhat** — [github/serhatonia](https://github.com/serhatonia) · [x/serhatonia](https://x.com/serhatonia)
