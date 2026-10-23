//! The fonts a vendor theme names, found by the names inside the font files
//! of the vendor app's `fonts` folder: a `.turtheme` stores only a family
//! name (`HelveticaNeueLT Pro 55 Roman`), which is rarely the file's name.

use std::fs;
use std::path::{Path, PathBuf};

use super::read_limited;
use super::turzx::FoundFont;

/// Largest font file read.
const MAX_FONT: u64 = 32 * 1024 * 1024;
/// Most files indexed in one folder.
const MAX_FILES: usize = 1024;
/// Most faces examined in a font collection.
const MAX_FACES: u32 = 256;
/// Font file extensions.
const EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];

/// The vendor app's font folders for the theme at `theme_file`:
/// `<app>/fonts` for `<app>/theme/<resolution>/<theme>.turtheme`, and a
/// `fonts` folder next to the theme or one level up.
pub(crate) fn vendor_font_folders(theme_file: &Path) -> Vec<PathBuf> {
    let dir = theme_file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut folders = Vec::new();
    for base in [Some(dir), dir.parent(), dir.parent().and_then(Path::parent)]
        .into_iter()
        .flatten()
    {
        let folder = base.join("fonts");
        if folder.is_dir() && !folders.contains(&folder) {
            folders.push(folder);
        }
    }
    folders
}

/// The font files of some folders and the names inside them, read the first
/// time a family is looked up.
pub(crate) struct FontFolders {
    folders: Vec<PathBuf>,
    index: Option<Vec<(PathBuf, Vec<String>)>>,
}

impl FontFolders {
    pub(crate) fn new(folders: Vec<PathBuf>) -> Self {
        Self {
            folders,
            index: None,
        }
    }

    /// The file whose font is named `family`: exactly, then ignoring case,
    /// spaces, hyphens and underscores.
    pub(crate) fn find(&mut self, family: &str) -> Option<FoundFont> {
        let folders = &self.folders;
        let index = self.index.get_or_insert_with(|| index(folders));
        let wanted = family.trim().to_lowercase();
        let compact = squeeze(&wanted);
        let path = index
            .iter()
            .find(|(_, names)| names.contains(&wanted))
            .or_else(|| {
                index
                    .iter()
                    .find(|(_, names)| names.iter().any(|n| squeeze(n) == compact))
            })
            .map(|(path, _)| path.clone())?;
        let bytes = read_limited(&path, MAX_FONT).ok()?;
        let file_name = path.file_name()?.to_string_lossy().into_owned();
        Some(FoundFont { file_name, bytes })
    }
}

fn squeeze(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .collect()
}

fn index(folders: &[PathBuf]) -> Vec<(PathBuf, Vec<String>)> {
    let mut index = Vec::new();
    for folder in folders {
        let Ok(entries) = fs::read_dir(folder) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
            })
            .take(MAX_FILES)
            .collect();
        files.sort();
        for path in files {
            if let Ok(bytes) = read_limited(&path, MAX_FONT) {
                let names = font_names(&bytes);
                if !names.is_empty() {
                    index.push((path, names));
                }
            }
        }
    }
    index
}

/// The lowercase names a font answers to, in every language it carries:
/// its family, full name and typographic family, and each family with its
/// style (`Bahnschrift` + `SemiBold`).
pub(crate) fn font_names(bytes: &[u8]) -> Vec<String> {
    use ttf_parser::name_id;
    let count = ttf_parser::fonts_in_collection(bytes)
        .unwrap_or(1)
        .min(MAX_FACES);
    let mut names = Vec::new();
    for face in (0..count).filter_map(|i| ttf_parser::Face::parse(bytes, i).ok()) {
        let of = |id: u16| -> Vec<String> {
            face.names()
                .into_iter()
                .filter(|n| n.name_id == id)
                .filter_map(|n| n.to_string())
                .map(|n| n.trim().to_lowercase())
                .filter(|n| !n.is_empty())
                .collect()
        };
        let families = of(name_id::FAMILY);
        let typographic = of(name_id::TYPOGRAPHIC_FAMILY);
        for (family, styles) in [
            (&families, of(name_id::SUBFAMILY)),
            (&typographic, of(name_id::TYPOGRAPHIC_SUBFAMILY)),
        ] {
            for family in family {
                for style in &styles {
                    names.push(format!("{family} {style}"));
                }
            }
        }
        names.extend(families);
        names.extend(typographic);
        names.extend(of(name_id::FULL_NAME));
    }
    names.sort();
    names.dedup();
    names
}
#[cfg(test)]
mod tests {
    use super::*;

    const FONT: &[u8] =
        include_bytes!("../../../bezel-render/tests/fonts/JetBrainsMonoNL-Regular.ttf");

    #[test]
    fn finds_fonts_by_internal_names_in_vendor_layout() {
        let scratch = tempfile::tempdir().expect("scratch");
        let app = scratch.path();
        let theme_dir = app.join("theme/4801920");
        fs::create_dir_all(&theme_dir).expect("theme directory");
        fs::create_dir(app.join("fonts")).expect("fonts directory");
        fs::write(app.join("fonts/unrelated-file-name.ttf"), FONT).expect("font");
        fs::write(app.join("fonts/broken.otf"), b"invalid").expect("broken font");
        let mut folders = FontFolders::new(vendor_font_folders(&theme_dir.join("test.turtheme")));
        for name in [
            "JetBrains Mono NL",
            "JETBRAINS MONO NL",
            "JetBrainsMonoNL-Regular",
        ] {
            let found = folders.find(name).expect("font found by internal name");
            assert_eq!(found.bytes, FONT);
            assert_eq!(found.file_name, "unrelated-file-name.ttf");
        }
        assert!(folders.find("Missing Font").is_none());
        assert!(font_names(b"invalid").is_empty());
    }
}
