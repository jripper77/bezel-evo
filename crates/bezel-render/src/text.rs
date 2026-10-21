//! Text: shaped with cosmic-text (HarfBuzz-compatible shaping, fallback to
//! other fonts for missing glyphs), rasterized with swash into a coverage
//! mask, then filled with the element's paint so gradients cover the text.
//!
//! Fonts come from the theme's bundled font assets first, then the installed
//! family, then a default sans.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use bezel_core::domain::theme::{AssetRef, BoxF, FontSpec, HAlign, TextStyle, VAlign};
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, SwashCache, SwashContent,
    SwashImage, Weight, Wrap,
};
use tiny_skia::{IntSize, Mask, Pixmap, Rect};

use crate::catalog::FontCatalog;
use crate::composite::premultiply;
use crate::diagnostics::{Diagnostics, Stamp};
use crate::fonts::{SHIPPED_FONTS, is_shipped};
use crate::layer::Layer;
use crate::paint::paint_for;

mod rich;
mod shadow;
pub(crate) use rich::{Piece, RichJob};

/// Whether the renderer may use the fonts installed on the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SystemFonts {
    /// Scan and use the installed fonts and the fonts Bezel ships (the
    /// default for the app).
    #[default]
    Load,
    /// Only the fonts given: reproducible output (tests, golden images).
    Skip,
}

/// Line height as a multiple of the font size.
const LINE_HEIGHT: f32 = 1.2;
/// Largest font size drawn, pixels.
const MAX_FONT_SIZE: f32 = 1024.0;
/// Families tried, in order, for the default sans.
const DEFAULT_FAMILIES: [&str; 10] = [
    "Inter",
    "Noto Sans",
    "DejaVu Sans",
    "Liberation Sans",
    "Open Sans",
    "Roboto",
    "Cantarell",
    "Segoe UI",
    "Arial",
    "Helvetica",
];

struct AssetFont {
    stamp: Stamp,
    ids: Vec<fontdb::ID>,
    family: Option<String>,
    families: HashMap<String, String>,
}

/// A text to draw.
pub(crate) struct TextJob<'a> {
    /// The text (may hold several lines).
    pub text: &'a str,
    /// How it looks.
    pub style: &'a TextStyle,
    /// The element box it is aligned in and clipped to.
    pub area: BoxF,
}

/// Fonts, shaping and glyph rasterization, cached across frames.
pub(crate) struct TextEngine {
    fonts: FontSystem,
    glyphs: SwashCache,
    default_family: Option<String>,
    installed: HashMap<String, Option<String>>,
    asset_fonts: HashMap<AssetRef, AssetFont>,
    /// The faces shipped with Bezel (the `fonts` given to [`TextEngine::new`]).
    bundled: HashSet<fontdb::ID>,
    user: HashSet<fontdb::ID>,
    base_db: Option<fontdb::Database>,
    user_stamp: Vec<(std::path::PathBuf, u64, Option<std::time::SystemTime>)>,
    checked: std::time::Instant,
    /// Digit glyphs and advances per face and size ([`rich`]).
    digits: HashMap<rich::DigitKey, rich::Digits>,
}

fn has_family(db: &fontdb::Database, name: &str) -> Option<String> {
    db.faces()
        .flat_map(|face| face.families.iter())
        .find(|(family, _)| family.eq_ignore_ascii_case(name))
        .map(|(family, _)| family.clone())
}

impl TextEngine {
    /// An engine with `fonts` (font file bytes) and optionally the installed
    /// fonts; with them, also the fonts Bezel ships ([`SHIPPED_FONTS`]),
    /// whatever folder the theme is in. Copies of those in `fonts` are skipped.
    pub fn new(fonts: Vec<Vec<u8>>, system: SystemFonts) -> Self {
        let mut db = fontdb::Database::new();
        let mut bundled = HashSet::new();
        if system == SystemFonts::Load {
            db.load_system_fonts();
            for bytes in SHIPPED_FONTS {
                bundled.extend(db.load_font_source(fontdb::Source::Binary(Arc::new(bytes))));
            }
        }
        for bytes in fonts {
            if system == SystemFonts::Load && is_shipped(&bytes) {
                continue;
            }
            bundled.extend(db.load_font_source(fontdb::Source::Binary(Arc::new(bytes))));
        }
        let default_family = DEFAULT_FAMILIES
            .iter()
            .find_map(|name| has_family(&db, name))
            .or_else(|| {
                db.faces()
                    .find_map(|face| face.families.first().map(|(f, _)| f.clone()))
            });
        if let Some(family) = &default_family {
            db.set_sans_serif_family(family.clone());
        }
        let base_db = (system == SystemFonts::Load).then(|| db.clone());
        let mut engine = Self {
            user: HashSet::new(),
            base_db,
            user_stamp: Vec::new(),
            checked: std::time::Instant::now() - std::time::Duration::from_secs(3),
            fonts: FontSystem::new_with_locale_and_db("en-US".to_string(), db),
            glyphs: SwashCache::new(),
            default_family,
            installed: HashMap::new(),
            asset_fonts: HashMap::new(),
            bundled,
            digits: HashMap::new(),
        };
        engine.refresh_user_fonts();
        engine
    }

    /// Poll only metadata at most every two seconds; rebuild shaping caches on changes.
    pub fn refresh_user_fonts(&mut self) -> bool {
        let Some(dir) = crate::fonts::user_fonts_directory() else {
            return false;
        };
        self.refresh_user_fonts_from(&dir)
    }

    fn refresh_user_fonts_from(&mut self, dir: &std::path::Path) -> bool {
        if self.base_db.is_none() || self.checked.elapsed() < std::time::Duration::from_secs(2) {
            return false;
        }
        self.checked = std::time::Instant::now();
        let stamp = crate::fonts::folder_stamp(dir);
        if stamp == self.user_stamp {
            return false;
        }
        self.user_stamp = stamp;
        let Some(mut db) = self.base_db.clone() else {
            return false;
        };
        self.user.clear();
        for bytes in crate::fonts::font_files(dir) {
            if is_shipped(&bytes) {
                continue;
            }
            let source = fontdb::Source::Binary(Arc::new(bytes));
            self.user.extend(db.load_font_source(source));
        }
        // Prefer the user's family over an installed copy, while keeping embedded faces.
        let names: HashSet<_> = self
            .user
            .iter()
            .filter_map(|id| db.face(*id))
            .flat_map(|f| f.families.iter().map(|(n, _)| n.to_lowercase()))
            .collect();
        let remove: Vec<_> = db
            .faces()
            .filter(|f| {
                !self.user.contains(&f.id)
                    && !self.bundled.contains(&f.id)
                    && f.families
                        .iter()
                        .any(|(n, _)| names.contains(&n.to_lowercase()))
            })
            .map(|f| f.id)
            .collect();
        for id in remove {
            db.remove_face(id);
        }
        self.fonts = FontSystem::new_with_locale_and_db("en-US".into(), db);
        self.glyphs = SwashCache::new();
        self.installed.clear();
        self.asset_fonts.clear();
        self.digits.clear();
        true
    }

    pub fn retain_assets(&mut self, used: &[AssetRef]) -> bool {
        let removed: Vec<_> = self
            .asset_fonts
            .keys()
            .filter(|a| !used.contains(a))
            .cloned()
            .collect();
        if removed.is_empty() {
            return false;
        }
        for asset in removed {
            if let Some(font) = self.asset_fonts.remove(&asset) {
                for id in font.ids {
                    self.fonts.db_mut().remove_face(id);
                }
            }
        }
        self.installed.clear();
        self.glyphs = SwashCache::new();
        self.digits.clear();
        true
    }

    fn installed(&mut self, family: &str) -> Option<String> {
        let key = family.to_lowercase();
        if let Some(hit) = self.installed.get(&key) {
            return hit.clone();
        }
        let found = has_family(self.fonts.db(), family);
        self.installed.insert(key, found.clone());
        found
    }

    fn asset_family(&mut self, asset: &AssetRef, bytes: &[u8], wanted: &str) -> Option<String> {
        if let Some(font) = self.asset_fonts.get_mut(asset)
            && font.stamp.matches(bytes)
        {
            return font
                .families
                .get(&wanted.to_lowercase())
                .cloned()
                .or_else(|| font.family.clone());
        }
        if !crate::fonts::valid_font(bytes) {
            return None;
        }
        let db = self.fonts.db_mut();
        for id in self
            .asset_fonts
            .remove(asset)
            .into_iter()
            .flat_map(|f| f.ids)
        {
            db.remove_face(id);
        }
        let ids: Vec<fontdb::ID> = db
            .load_font_source(fontdb::Source::Binary(Arc::new(bytes.to_vec())))
            .into_iter()
            .collect();
        // Private aliases give assets precedence over installed fonts and other assets.
        let mut families = HashMap::new();
        let mut aliased = Vec::new();
        let mut family = None;
        for id in ids {
            let Some(mut info) = db.face(id).cloned() else {
                continue;
            };
            db.remove_face(id);
            for (name, _) in &mut info.families {
                let alias = format!("bezel-asset-{}:{name}", asset.0);
                families.insert(name.to_lowercase(), alias.clone());
                if family.is_none() {
                    family = Some(alias.clone());
                }
                *name = alias;
            }
            aliased.push(db.push_face_info(info));
        }
        let selected = families
            .get(&wanted.to_lowercase())
            .cloned()
            .or_else(|| family.clone());
        self.installed.clear();
        let font = AssetFont {
            stamp: Stamp::new(bytes),
            ids: aliased,
            families,
            family,
        };
        self.asset_fonts.insert(asset.clone(), font);
        selected
    }

    /// The family to shape with: the bundled asset, the installed family or
    /// the default sans.
    fn family_for(
        &mut self,
        font: &FontSpec,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) -> Option<String> {
        if let Some(asset) = &font.asset {
            let family = assets
                .get(asset)
                .and_then(|b| self.asset_family(asset, b, &font.family));
            if family.is_some() {
                return family;
            }
            diagnostics.warn(format!(
                "font asset {} is missing or unreadable; using {}",
                asset.0, font.family
            ));
        }
        if let Some(family) = self.installed(&font.family) {
            return Some(family);
        }
        let fallback = match &self.default_family {
            Some(family) => format!("{family:?}"),
            None => "no font".to_string(),
        };
        diagnostics.warn(format!(
            "font family {:?} is not available; using {fallback}",
            font.family
        ));
        self.default_family.clone()
    }

    /// Every family of the font database, with its faces, and the family
    /// drawn for a missing one.
    pub fn catalog(&self) -> FontCatalog {
        if let Some(base) = &self.base_db {
            let mut seen = HashSet::new();
            let files = self
                .fonts
                .db()
                .faces()
                .filter(|f| self.user.contains(&f.id))
                .filter_map(|f| {
                    if let fontdb::Source::Binary(bytes) = &f.source {
                        seen.insert(Arc::as_ptr(bytes) as *const () as usize)
                            .then(|| (**bytes).as_ref().to_vec())
                    } else {
                        None
                    }
                })
                .collect();
            crate::catalog::catalog_with_user(
                base,
                &self.bundled,
                &HashSet::new(),
                self.default_family.clone(),
            )
            .with_user_fonts(files)
        } else {
            crate::catalog::catalog_with_user(
                self.fonts.db(),
                &self.bundled,
                &self.user,
                self.default_family.clone(),
            )
        }
    }

    pub fn watches_user_fonts(&self) -> bool {
        self.base_db.is_some()
    }

    /// Draws `job` into `layer`.
    pub fn draw(
        &mut self,
        layer: &mut Layer<'_>,
        job: &TextJob<'_>,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) {
        let size = job.style.size.min(MAX_FONT_SIZE);
        if job.text.is_empty() || size.is_nan() || size < 1.0 {
            return;
        }
        let Some(family) = self.family_for(&job.style.font, assets, diagnostics) else {
            diagnostics.warn("no font is available to draw text".to_string());
            return;
        };
        let buffer = self.shape(job, &family, size);
        let Some((mask, ink, color_glyphs)) = self.rasterize(layer, &buffer, job) else {
            return;
        };
        if let Some(shadow) = job.style.shadow {
            shadow::draw(
                layer,
                std::iter::once(mask.data().to_vec()),
                &color_glyphs,
                shadow,
            );
        }
        if let (Some(paint), Some(ink)) = (paint_for(&job.style.paint, job.area), ink) {
            layer.fill_rect_masked(ink, &paint, &mask);
        }
        for (pixmap, x, y) in color_glyphs {
            layer.blit(pixmap.as_ref(), x, y);
        }
    }

    /// Natural unwrapped dimensions, using the drawing shaper and font resolver.
    pub fn measure(
        &mut self,
        job: &TextJob<'_>,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) -> Option<(f32, f32)> {
        let size = job.style.size.min(MAX_FONT_SIZE);
        if job.text.is_empty() || !size.is_finite() || size < 1.0 {
            return None;
        }
        let family = self.family_for(&job.style.font, assets, diagnostics)?;
        let buffer = self.shape(job, &family, size);
        let width = buffer
            .layout_runs()
            .map(|r| r.line_w)
            .fold(0.0_f32, f32::max);
        let height: f32 = buffer.layout_runs().map(|r| r.line_height).sum();
        Some((width.ceil().max(1.0), height.ceil().max(1.0)))
    }

    fn shape(&mut self, job: &TextJob<'_>, family: &str, size: f32) -> Buffer {
        let style = job.style;
        let metrics = Metrics::new(size, (size * LINE_HEIGHT).ceil());
        let mut buffer = Buffer::new(&mut self.fonts, metrics);
        buffer.set_wrap(Wrap::None);
        buffer.set_size(None, None);
        let mut attrs = Attrs::new()
            .family(Family::Name(family))
            .weight(Weight(style.font.weight.clamp(1, 1000)))
            .style(if style.font.italic {
                Style::Italic
            } else {
                Style::Normal
            });
        if style.letter_spacing.is_finite() && style.letter_spacing != 0.0 {
            attrs = attrs.letter_spacing(style.letter_spacing / size);
        }
        let displayed = style.casing.apply(job.text);
        buffer.set_text(&displayed, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        buffer
    }

    /// Glyph coverage in a mask the size of the layer, plus color glyphs
    /// (emoji) as premultiplied images with their canvas position.
    fn rasterize(
        &mut self,
        layer: &Layer<'_>,
        buffer: &Buffer,
        job: &TextJob<'_>,
    ) -> Option<Rasterized> {
        let (w, h) = (layer.pixmap.width(), layer.pixmap.height());
        let mut coverage = Coverage {
            data: vec![0; w as usize * h as usize],
            width: w as i32,
            height: h as i32,
            ink: None,
        };
        let mut color_glyphs = Vec::new();
        let area = job.area;
        let height: f32 = buffer.layout_runs().map(|run| run.line_height).sum();
        let top = area.y
            + match job.style.valign {
                VAlign::Top => 0.0,
                VAlign::Middle => (area.height - height) / 2.0,
                VAlign::Bottom => area.height - height,
            };
        for run in buffer.layout_runs() {
            let left = area.x
                + match job.style.align {
                    HAlign::Left => 0.0,
                    HAlign::Center => (area.width - run.line_w) / 2.0,
                    HAlign::Right => area.width - run.line_w,
                };
            for glyph in run.glyphs {
                let physical = glyph.physical((left, top + run.line_y), 1.0);
                let Some(image) = self.glyphs.get_image(&mut self.fonts, physical.cache_key) else {
                    continue;
                };
                let x = physical.x + image.placement.left;
                let y = physical.y - image.placement.top;
                match image.content {
                    SwashContent::Mask => coverage.add(image, x - layer.x, y - layer.y),
                    SwashContent::Color => {
                        color_glyphs.extend(color_glyph(image).map(|p| (p, x, y)))
                    }
                    SwashContent::SubpixelMask => {}
                }
            }
        }
        let ink = coverage.ink.and_then(|(l, t, r, b)| {
            let (x, y) = (layer.x + l, layer.y + t);
            Rect::from_xywh(x as f32, y as f32, (r - l) as f32, (b - t) as f32)
        });
        let mask = Mask::from_vec(coverage.data, IntSize::from_wh(w, h)?)?;
        Some((mask, ink, color_glyphs))
    }
}

/// Glyph coverage over the layer, the canvas rectangle that holds ink and
/// the color glyphs with their canvas position.
type Rasterized = (Mask, Option<Rect>, Vec<(Pixmap, i32, i32)>);

/// Glyph coverage accumulated over the layer.
struct Coverage {
    data: Vec<u8>,
    width: i32,
    height: i32,
    /// Layer pixels written, `(left, top, right, bottom)` exclusive.
    ink: Option<(i32, i32, i32, i32)>,
}

impl Coverage {
    /// Adds a glyph mask whose top-left is at layer `(x, y)`.
    fn add(&mut self, image: &SwashImage, x: i32, y: i32) {
        self.add_rows(image, x, y, (i32::MIN, i32::MAX));
    }

    /// Adds a glyph mask whose top-left is at layer `(x, y)`, only on the
    /// layer rows `top..bottom` (a rolling digit inside its line).
    fn add_rows(&mut self, image: &SwashImage, x: i32, y: i32, (top, bottom): (i32, i32)) {
        let gw = image.placement.width as i32;
        let gh = image.placement.height as i32;
        let (c0, c1) = ((-x).max(0), gw.min(self.width - x));
        let (r0, r1) = (
            (-y).max(0).max(top.saturating_sub(y)),
            gh.min(self.height - y).min(bottom.saturating_sub(y)),
        );
        if c0 >= c1 || r0 >= r1 {
            return;
        }
        let (l, t, r, b) = (x + c0, y + r0, x + c1, y + r1);
        self.ink = Some(match self.ink {
            None => (l, t, r, b),
            Some((l0, t0, r0, b0)) => (l0.min(l), t0.min(t), r0.max(r), b0.max(b)),
        });
        for row in r0..r1 {
            let src = (row * gw + c0) as usize..(row * gw + c1) as usize;
            let dst = ((y + row) * self.width + x + c0) as usize;
            let (Some(src), Some(dst)) = (
                image.data.get(src),
                self.data.get_mut(dst..dst + (c1 - c0) as usize),
            ) else {
                return;
            };
            for (d, &s) in dst.iter_mut().zip(src) {
                // Union of coverages: a + b - ab.
                let sum = u32::from(*d) + u32::from(s);
                *d = (sum - (u32::from(*d) * u32::from(s) + 127) / 255) as u8;
            }
        }
    }
}

/// A color glyph as a premultiplied image.
fn color_glyph(image: &SwashImage) -> Option<Pixmap> {
    let size = IntSize::from_wh(image.placement.width, image.placement.height)?;
    let mut data = image.data.clone();
    premultiply(&mut data);
    Pixmap::from_vec(data, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::FONT;

    #[test]
    fn a_collection_keeps_the_requested_family_and_asset_aliases_win_over_installed_names() {
        let mut collection = b"ttcf\x00\x01\x00\x00\x00\x00\x00\x02".to_vec();
        collection.resize(20, 0);
        for (index, font) in [FONT, SHIPPED_FONTS[0]].iter().enumerate() {
            while !collection.len().is_multiple_of(4) {
                collection.push(0);
            }
            let start = collection.len();
            collection[12 + index * 4..16 + index * 4]
                .copy_from_slice(&(start as u32).to_be_bytes());
            collection.extend_from_slice(font);
            let tables = u16::from_be_bytes([font[4], font[5]]) as usize;
            for table in 0..tables {
                let at = start + 12 + table * 16 + 8;
                let offset = u32::from_be_bytes(collection[at..at + 4].try_into().unwrap());
                collection[at..at + 4].copy_from_slice(&(offset + start as u32).to_be_bytes());
            }
        }
        assert!(crate::valid_font(&collection));
        let mut engine = TextEngine::new(
            vec![FONT.to_vec(), SHIPPED_FONTS[0].to_vec()],
            SystemFonts::Skip,
        );
        let asset = AssetRef("assets/collection.ttc".into());
        let inter = engine.asset_family(&asset, &collection, "Inter").unwrap();
        let mono = engine
            .asset_family(&asset, &collection, crate::testkit::FAMILY)
            .unwrap();
        assert_eq!(inter, "bezel-asset-assets/collection.ttc:Inter");
        assert_ne!(inter, mono);
        let catalog = crate::FontCatalog::default().with_user_fonts(vec![collection.clone()]);
        assert_eq!(
            catalog.portable_face(
                "Inter",
                crate::FontFace {
                    weight: 400,
                    italic: false
                }
            ),
            Some(collection)
        );
    }

    #[test]
    fn user_files_reload_and_detached_assets_do_not_pollute_family_matching() {
        let dir = std::env::temp_dir().join(format!("bezel-user-reload-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut engine = TextEngine::new(Vec::new(), SystemFonts::Skip);
        engine.base_db = Some(engine.fonts.db().clone());
        std::fs::write(dir.join("local.ttf"), FONT).unwrap();
        assert!(engine.refresh_user_fonts_from(&dir));
        let catalog = engine.catalog();
        assert_eq!(catalog.families[0].origin, crate::FontOrigin::User);
        assert!(
            catalog
                .portable_face(
                    crate::testkit::FAMILY,
                    crate::FontFace {
                        weight: 400,
                        italic: false
                    }
                )
                .is_some()
        );
        assert!(
            !engine.refresh_user_fonts_from(&dir),
            "metadata polls are throttled"
        );
        std::fs::remove_file(dir.join("local.ttf")).unwrap();
        engine.checked -= std::time::Duration::from_secs(3);
        assert!(engine.refresh_user_fonts_from(&dir));
        assert!(engine.catalog().families.is_empty());
        let asset = AssetRef("assets/local.ttf".into());
        let alias = engine
            .asset_family(&asset, FONT, crate::testkit::FAMILY)
            .unwrap();
        assert!(alias.starts_with("bezel-asset-"));
        assert!(engine.installed(crate::testkit::FAMILY).is_none());
        assert!(engine.retain_assets(&[]));
        assert_eq!(engine.fonts.db().faces().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn families_resolve_to_assets_installed_or_default() {
        let mut engine = TextEngine::new(vec![FONT.to_vec()], SystemFonts::Skip);
        let mut diagnostics = Diagnostics::default();
        let assets = BTreeMap::from([
            (AssetRef("assets/font.ttf".into()), FONT.to_vec()),
            (AssetRef("assets/bad.ttf".into()), b"not a font".to_vec()),
        ]);
        let spec = |family: &str, asset: Option<&str>| FontSpec {
            family: family.into(),
            asset: asset.map(|a| AssetRef(a.into())),
            ..FontSpec::default()
        };
        let default = Some("JetBrains Mono NL".to_string());
        assert_eq!(engine.default_family, default);
        let installed =
            engine.family_for(&spec("jetbrains mono nl", None), &assets, &mut diagnostics);
        assert_eq!(installed, default);
        assert_eq!(diagnostics.count(), 0);
        let bundled = engine.family_for(
            &spec("Nope", Some("assets/font.ttf")),
            &assets,
            &mut diagnostics,
        );
        let alias = Some("bezel-asset-assets/font.ttf:JetBrains Mono NL".to_string());
        assert_eq!(bundled, alias);
        assert_eq!(
            engine.asset_family(
                &AssetRef("assets/font.ttf".into()),
                FONT,
                crate::testkit::FAMILY
            ),
            alias
        );
        let unknown = engine.family_for(
            &spec("Nope", Some("assets/bad.ttf")),
            &assets,
            &mut diagnostics,
        );
        assert_eq!(unknown, default);
        assert!(diagnostics.mentions("unreadable"));
        assert!(diagnostics.mentions("not available"));

        let mut empty = TextEngine::new(Vec::new(), SystemFonts::Skip);
        assert_eq!(
            empty.family_for(&spec("Inter", None), &assets, &mut diagnostics),
            None
        );
    }

    #[test]
    fn the_shipped_families_are_always_there_and_bundled() {
        let copies: Vec<Vec<u8>> = SHIPPED_FONTS.iter().map(|b| b.to_vec()).collect();
        let mut engine = TextEngine::new(copies, SystemFonts::Load);
        let mut diagnostics = Diagnostics::default();
        let catalog = engine.catalog();
        for (family, _) in crate::SHIPPED_FAMILIES {
            let spec = FontSpec {
                family: family.into(),
                ..FontSpec::default()
            };
            let found = engine.family_for(&spec, &BTreeMap::new(), &mut diagnostics);
            assert_eq!(found.as_deref(), Some(family));
            let listed = catalog.families.iter().find(|f| f.name == family).unwrap();
            assert_eq!(listed.origin, crate::FontOrigin::Bundled, "{family}");
        }
        for (name, expected) in [
            (
                "Fraunces 144pt",
                vec![
                    (300, false),
                    (300, true),
                    (400, false),
                    (400, true),
                    (600, false),
                ],
            ),
            (
                "Instrument Sans",
                vec![(400, false), (500, false), (600, false)],
            ),
            (
                "Space Grotesk",
                vec![(400, false), (500, false), (600, false)],
            ),
            (
                "Bricolage Grotesque 96pt",
                vec![(400, false), (500, false), (700, false), (800, false)],
            ),
        ] {
            let family = catalog.families.iter().find(|f| f.name == name).unwrap();
            let faces: Vec<_> = family.faces.iter().map(|f| (f.weight, f.italic)).collect();
            assert_eq!(
                faces, expected,
                "{name}: only the template faces are shipped"
            );
            for face in &family.faces {
                assert!(
                    catalog.bundled_face(name, *face).is_some(),
                    "{name}: face bytes"
                );
            }
        }
        assert_eq!(diagnostics.count(), 0);
        let shipped = engine.bundled.len();
        assert_eq!(shipped, SHIPPED_FONTS.len(), "copies are not loaded twice");
        assert!(
            TextEngine::new(Vec::new(), SystemFonts::Skip)
                .bundled
                .is_empty()
        );
    }

    #[test]
    fn coverage_is_a_clipped_union() {
        let mut coverage = Coverage {
            data: vec![0; 4],
            width: 2,
            height: 2,
            ink: None,
        };
        let mut image = SwashImage::new();
        image.placement.width = 2;
        image.placement.height = 2;
        image.data = vec![255, 128, 128, 0];
        coverage.add(&image, 1, 1);
        assert_eq!(coverage.data, vec![0, 0, 0, 255]);
        assert_eq!(coverage.ink, Some((1, 1, 2, 2)));
        coverage.add(&image, 0, 0);
        coverage.add(&image, 0, 0);
        assert_eq!(coverage.data, vec![255, 192, 192, 255]);
        assert_eq!(coverage.ink, Some((0, 0, 2, 2)));
        coverage.add(&image, 5, 0);
        coverage.add(&image, -2, 0);
        assert_eq!(coverage.data, vec![255, 192, 192, 255]);
        image.content = SwashContent::Color;
        image.data = [255, 0, 0, 128].repeat(4);
        let pixmap = color_glyph(&image).expect("glyph");
        assert_eq!(
            pixmap.pixel(0, 0).map(|p| (p.red(), p.alpha())),
            Some((128, 128))
        );
    }
}
