//! Build pipeline for the ref42 Topcoat site.
//!
//! Reads `content/**/*.md`, renders CommonMark to syntax-highlighted HTML, and
//! emits `generated.rs` into `OUT_DIR` so the binary ships every note as static
//! data. Also concatenates and minifies `styles/*.css` into `assets/main.css`,
//! mirrors `static/` into `assets/`, and writes `assets/search-index.json`.
//!
//! The string transforms themselves live in `src/pipeline.rs`, because a build
//! script's own `#[cfg(test)]` modules never run under `cargo test`; that file
//! is included here by path and covered by tests in the binary target.

#[path = "src/pipeline.rs"]
mod pipeline;

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Options, markdown_to_html_with_plugins};

use pipeline::{
    Frontmatter, ImageInfo, TocEntry, add_heading_anchors, age_in_days, count_words,
    image_dimensions, images_without_alt, is_date, join_multiline_tags, json_string, minify_css,
    rewrite_images, short_hash, split_frontmatter, strip_bom, strip_html, wrap_code_blocks,
    wrap_tables, xml_escape,
};

/// Where the site is published, unless a deploy overrides it.
///
/// Only used to build absolute URLs (canonical links, Open Graph, the feed and
/// the sitemap), which crawlers and social cards require to be absolute.
const DEFAULT_BASE_URL: &str = "https://www.ref42.com";

/// How old a note may get before it is flagged as possibly out of date.
///
/// Six months. These are toolchain guides, and esp-hal, espflash, embassy and
/// probe-rs all cut releases faster than that, so the badge is a prompt to
/// re-verify rather than a claim that the note is wrong.
const STALE_AFTER_DAYS: i64 = 180;

/// Files `build.rs` writes into `assets/` itself. They have no `static/` source,
/// so the mirror must never overwrite them and the purge must never delete them.
const GENERATED_ASSETS: &[&str] = &[
    "main.css",
    "app.js",
    "search-index.json",
    "feed.xml",
    "sitemap.xml",
    "robots.txt",
];

/// Replaced with the font content hash inside the concatenated stylesheet.
///
/// A token rather than a hardcoded `?v=iosevka-2`: the old value was edited by
/// hand, and forgetting to bump it served a stale font for as long as the
/// browser's cache lived. An unreplaced token would be visible in the URL, so
/// this fails loudly rather than silently.
const FONT_VERSION_TOKEN: &str = "__FONT_VERSION__";

struct Section {
    key: String,
    title: String,
    short: String,
    description: String,
    order: i64,
    /// Rendered body of the section's `_index.md`.
    intro: String,
}

struct Note {
    section: String,
    slug: String,
    url: String,
    title: String,
    description: String,
    html: String,
    plain: String,
    toc: Vec<TocEntry>,
    words: usize,
    minutes: usize,
    /// Sort key inside its shelf. Lower first, matching Zola's `weight`.
    weight: i64,
    date: String,
    updated: String,
    /// Days between `updated` and the build, so the page can say how long ago
    /// the note was last checked.
    age_days: i64,
    /// True once the note is old enough that its toolchain advice may have moved.
    stale: bool,
    tags: Vec<String>,
    tested_with: Vec<String>,
}

/// Fallback shelf metadata, used only when a section's `_index.md` does not
/// carry `order`/`short` itself.
///
/// This used to be the *only* place a shelf could be declared, so adding one
/// meant editing Rust and a directory that was not listed here was skipped in
/// silence. Now the frontmatter wins and a directory that has neither is
/// reported at build time.
/// `(order, short label, fallback description)`.
fn section_meta(key: &str) -> Option<(i64, &'static str, &'static str)> {
    match key {
        "toolchain" => Some((
            0,
            "Toolchain",
            "Build a Rust embedded development environment from scratch with rustup, targets, flashing, and debugging tools.",
        )),
        "esp32" => Some((
            1,
            "ESP32",
            "Rust development guides, template projects, and sensor experiments for ESP32-C3/C6/S3.",
        )),
        "stm32" => Some((
            2,
            "STM32",
            "Rust development, Embassy, probe-rs, and register-level experiments for STM32F4/H7.",
        )),
        "utils" => Some((
            3,
            "Tools",
            "Ref42 tools for EDA library export, desktop workflows, and embedded development.",
        )),
        _ => None,
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    for tracked in ["content", "styles", "scripts", "static", "build.rs"] {
        println!("cargo::rerun-if-changed={tracked}");
    }
    println!("cargo::rerun-if-env-changed=SITE_BASE_URL");

    let base_url = std::env::var("SITE_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string();

    let content_root = manifest.join("content");
    let images = load_images(&manifest.join("static"));

    // Days since the epoch, so `age_in_days` needs no calendar conversion. This
    // is the one place the build is deliberately time-dependent: the staleness
    // badge is a claim about the present, and a site nobody has rebuilt should
    // be able to say so rather than keep insisting everything is fresh.
    let today_days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| (since.as_secs() / 86_400) as i64);

    let (sections, notes) = load_content(&content_root, &images, today_days);
    let (site_title, site_description) = load_site_meta(&content_root);

    // The font is versioned separately from the rest of the assets on purpose:
    // it is ~1 MB, so a change to `app.js` must not invalidate it in every
    // returning visitor's cache.
    let font_version =
        short_hash_of(&[manifest.join("static/fonts/iosevka-latin-400-normal.woff2")]);
    let asset_version = asset_version(&manifest, &font_version);

    build_assets(&manifest, &font_version);
    write_search_index(&manifest, &sections, &notes);

    let meta = SiteMeta {
        title: site_title,
        description: site_description,
        base_url,
        asset_version,
        font_version,
    };
    write_crawler_files(&manifest, &sections, &notes, &meta);
    let generated = render_rust(&sections, &notes, &meta);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("generated.rs"), generated).expect("write generated.rs");
}

/// Site-wide identity and build metadata, emitted into `generated.rs`.
struct SiteMeta {
    title: String,
    description: String,
    base_url: String,
    asset_version: String,
    font_version: String,
}

/// The site title and description, from `content/_index.md`.
///
/// These were constants in `src/site.rs` *and* frontmatter here, and the two
/// had already drifted apart 閳?two different descriptions of the same site.
/// Content wins, and the duplicate constants are gone.
fn load_site_meta(root: &Path) -> (String, String) {
    let src = std::fs::read_to_string(root.join("_index.md")).unwrap_or_default();
    let (fm, _) = split_frontmatter(&src);
    (
        fm.get("title").unwrap_or("ref42").to_string(),
        fm.get("description").unwrap_or_default().to_string(),
    )
}

// ---------------------------------------------------------------------------
// content loading
// ---------------------------------------------------------------------------

fn load_content(
    root: &Path,
    images: &HashMap<String, ImageInfo>,
    today_days: i64,
) -> (Vec<Section>, Vec<Note>) {
    let options = options();
    let plugins = plugins();

    let mut sections: Vec<Section> = Vec::new();
    let mut notes = Vec::new();

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .expect("read content dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    for dir in dirs {
        let key = dir
            .file_name()
            .expect("section name")
            .to_string_lossy()
            .to_string();

        let index_path = dir.join("_index.md");
        let index_src = std::fs::read_to_string(&index_path).unwrap_or_default();
        let (fm, index_body) = split_frontmatter(&index_src);
        let fallback = section_meta(&key);

        if !index_path.exists() && fallback.is_none() {
            println!(
                "cargo::warning=content/{key}/ has no _index.md and no entry in section_meta, so its notes are not published"
            );
            continue;
        }

        let short = fm
            .get("short")
            .map(str::to_string)
            .or_else(|| fallback.map(|f| f.1.to_string()))
            .unwrap_or_else(|| key.clone());
        let title = fm
            .get("title")
            .map(str::to_string)
            .unwrap_or_else(|| short.clone());
        let description = fm
            .get("description")
            .map(str::to_string)
            .or_else(|| fallback.map(|f| f.2.to_string()))
            .unwrap_or_default();
        let order = match fm.number("order").or_else(|| fallback.map(|f| f.0)) {
            Some(order) => order,
            None => {
                println!(
                    "cargo::warning=content/{key}/_index.md has no `order`, so the shelf sorts last"
                );
                i64::MAX
            }
        };

        // The intro is real authored copy 閳?the shelves lost it entirely when
        // the body was dropped during the port, and two of them still carry
        // links that only exist here.
        let intro = if index_body.trim().is_empty() {
            String::new()
        } else {
            let rendered = markdown_to_html_with_plugins(&index_body, &options, &plugins);
            let (with_ids, _) = add_heading_anchors(&rendered);
            let html = rewrite_images(&wrap_tables(&wrap_code_blocks(&with_ids)), |src| {
                images.get(src).cloned()
            });
            warn_missing_alt(&html, &format!("/{key}/"));
            html
        };

        sections.push(Section {
            key: key.clone(),
            title,
            short,
            description,
            order,
            intro,
        });

        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .expect("read section dir")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .filter(|p| p.file_name().is_some_and(|n| n != "_index.md"))
            .collect();
        files.sort();

        for file in files {
            let raw = std::fs::read_to_string(&file).expect("read note");
            let (fm, body) = split_frontmatter(&raw);
            let body = join_multiline_tags(&body);
            let stem = file
                .file_stem()
                .expect("stem")
                .to_string_lossy()
                .to_string();
            let slug = fm
                .get("slug")
                .map(str::to_string)
                .unwrap_or_else(|| stem.clone());
            let url = format!("/{key}/{slug}/");

            let rendered = markdown_to_html_with_plugins(&body, &options, &plugins);
            let (with_ids, toc) = add_heading_anchors(&rendered);
            let html = rewrite_images(&wrap_tables(&wrap_code_blocks(&with_ids)), |src| {
                images.get(src).cloned()
            });
            warn_missing_alt(&html, &url);

            let plain = strip_html(&html);
            let words = count_words(&plain);

            let date = read_date(&fm, "date", &url);
            // A note with only a `date` is treated as last verified then, so
            // the feed and the page never have to say "unknown".
            let updated = read_date(&fm, "updated", &url);
            let updated = if updated.is_empty() {
                date.clone()
            } else {
                updated
            };

            // How long ago this was last checked. The badge is a claim about the
            // present, so it is computed against the build's own clock rather
            // than stored in the content: a site nobody has rebuilt for a year
            // should say so, not keep insisting everything is fresh.
            let age_days = age_in_days(&updated, today_days).unwrap_or(0);

            notes.push(Note {
                section: key.clone(),
                slug,
                url,
                title: fm
                    .get("title")
                    .map(str::to_string)
                    .unwrap_or_else(|| stem.clone()),
                description: fm
                    .get("description")
                    .map(str::to_string)
                    .unwrap_or_default(),
                html,
                toc,
                plain,
                words,
                minutes: (words as f64 / 350.0).ceil().max(1.0) as usize,
                // Lower sorts first. A note without a weight sorts last rather
                // than first, so adding one never silently jumps the queue.
                weight: fm.number("weight").unwrap_or(i64::MAX),
                date,
                updated,
                age_days,
                stale: age_days >= STALE_AFTER_DAYS,
                tags: clean_list(fm.list("tags")),
                tested_with: fm.list("tested_with"),
            });
        }
    }

    let order_of = |key: &str| -> i64 {
        sections
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.order)
            .unwrap_or(i64::MAX)
    };
    let weight_of = |note: &Note| -> (i64, i64, String) {
        (order_of(&note.section), note.weight, note.slug.clone())
    };
    // Without the weight, notes sorted alphabetically by filename and the
    // intended reading order was silently replaced 閳?the "five minute setup"
    // note that every shelf means to open with ended up third.
    notes.sort_by_key(weight_of);
    sections.sort_by_key(|s| (s.order, s.key.clone()));

    (sections, notes)
}

/// Reads a `YYYY-MM-DD` field, warning instead of silently ignoring a typo.
fn read_date(fm: &Frontmatter, key: &str, url: &str) -> String {
    let Some(value) = fm.get(key) else {
        return String::new();
    };
    if is_date(value) {
        value.trim().to_string()
    } else {
        println!("cargo::warning={url}: `{key}: {value}` is not a YYYY-MM-DD date and was ignored");
        String::new()
    }
}

/// Normalises a tag list: trimmed, lowercased, de-duplicated, order preserved.
fn clean_list(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let tag = item.trim().to_lowercase();
        if !tag.is_empty() && !out.contains(&tag) {
            out.push(tag);
        }
    }
    out
}

fn warn_missing_alt(html: &str, where_: &str) {
    for src in images_without_alt(html) {
        println!("cargo::warning={where_}: <img src=\"{src}\"> has no alt attribute");
    }
}

/// Every image under `static/`, keyed by the URL the site serves it at, so the
/// renderer can add intrinsic dimensions and offer a WebP source.
fn load_images(root: &Path) -> HashMap<String, ImageInfo> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(root, &mut files);

    // WebP siblings are matched to their PNG by stem, and only offered when the
    // WebP is genuinely smaller: one flat KiCad diagram re-encoded at quality
    // 82 came out 5% *larger* than the PNG it came from, and offering it would
    // have cost bandwidth on exactly the image least able to afford it.
    let mut webp_by_stem: HashMap<String, PathBuf> = HashMap::new();
    for path in files.iter().filter(|p| has_extension(p, "webp")) {
        let Some(url) = url_of(root, path) else {
            continue;
        };
        let Some((stem, _)) = url.rsplit_once('.') else {
            continue;
        };
        webp_by_stem.insert(stem.to_string(), path.clone());
    }

    let mut images = HashMap::new();
    for path in files.iter().filter(|p| {
        ["png", "jpg", "jpeg", "gif"]
            .iter()
            .any(|ext| has_extension(p, ext))
    }) {
        let Some(url) = url_of(root, path) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let Some((width, height)) = image_dimensions(&bytes) else {
            continue;
        };

        let webp = url.rsplit_once('.').and_then(|(stem, _)| {
            let candidate = webp_by_stem.get(stem)?;
            let webp_len = std::fs::metadata(candidate).ok()?.len();
            let source_len = std::fs::metadata(path).ok()?.len();
            (webp_len < source_len).then(|| format!("{stem}.webp"))
        });

        images.insert(
            url,
            ImageInfo {
                width,
                height,
                webp,
            },
        );
    }

    images
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
    out.sort();
}

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

/// `/utils/seex/1.png` for `static/utils/seex/1.png`.
fn url_of(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let mut url = String::from("/");
    for component in relative.components() {
        let name = component.as_os_str().to_string_lossy();
        url.push_str(&name);
        url.push('/');
    }
    url.pop();
    Some(url)
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    // Raw HTML in the notes (hand-written image pairs, centered logos) must be
    // passed through verbatim. comrak 0.55 gates this on `render.unsafe`; the
    // older `render.escape` flag means the opposite and *escapes* raw HTML,
    // which silently mangled every hand-written tag.
    options.render.r#unsafe = true;
    options.render.escape = false;
    // `github_pre_lang` must stay off so the syntect adapter can emit its own
    // highlighted markup instead of comrak's escaped passthrough.
    options.render.github_pre_lang = false;
    options
}

fn plugins() -> Plugins<'static> {
    let mut plugins = Plugins::default();
    // `SyntectAdapterBuilder` always indexes syntect's *bundled* theme set, so
    // the theme must be one of the names it ships. `base16-mocha.dark` is the
    // closest match to the `catppuccin-mocha` the old Zola config asked for,
    // and `styles/note.css` layers the Catppuccin surface colour over it.
    let adapter: &'static SyntectAdapter = Box::leak(Box::new(
        SyntectAdapterBuilder::new().theme(HIGHLIGHT_THEME).build(),
    ));
    plugins.render.codefence_syntax_highlighter = Some(adapter);
    plugins
}

/// A syntect theme bundled with the crate.
const HIGHLIGHT_THEME: &str = "base16-mocha.dark";

// ---------------------------------------------------------------------------
// code generation
// ---------------------------------------------------------------------------

/// Escapes a value as a Rust string literal.
///
/// Control characters other than the four common ones are escaped too: a stray
/// form feed or NUL in a note would otherwise produce generated source that
/// does not compile, and the error would point at generated code.
fn rust_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 || ch == '\u{7f}' => {
                let _ = write!(out, "\\u{{{:x}}}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn render_rust(sections: &[Section], notes: &[Note], meta: &SiteMeta) -> String {
    let mut out = String::new();
    out.push_str("// @generated by build.rs -- do not edit.\n\n");

    let _ = writeln!(
        out,
        "/// Site name, from `content/_index.md`.\npub const SITE_TITLE: &str = {};\n",
        rust_str(&meta.title)
    );
    let _ = writeln!(
        out,
        "/// Site description, from `content/_index.md`.\npub const SITE_DESCRIPTION: &str = {};\n",
        rust_str(&meta.description)
    );
    let _ = writeln!(
        out,
        "/// The absolute URL this build is published at, for canonical links,\n/// Open Graph, the feed and the sitemap.\npub const SITE_BASE_URL: &str = {};\n",
        rust_str(&meta.base_url)
    );
    let _ = writeln!(
        out,
        "/// Content hash of the stylesheet and client script, for cache busting.\n/// `static_files.rs` treats a matching `?v=` as immutable.\npub const ASSET_VERSION: &str = {};\n",
        rust_str(&meta.asset_version)
    );
    let _ = writeln!(
        out,
        "/// Content hash of the webfont, versioned separately so a change to the\n/// client script does not invalidate a 1 MB download.\npub const FONT_VERSION: &str = {};\n",
        rust_str(&meta.font_version)
    );

    out.push_str(
        "pub struct Section {\n    pub key: &'static str,\n    pub title: &'static str,\n    pub short: &'static str,\n    pub description: &'static str,\n    /// Rendered body of the shelf's `_index.md`.\n    pub intro: &'static str,\n}\n\n",
    );
    // `plain` is deliberately absent: it is only needed while generating the
    // search index, so the plain-text body of every note stays in the build
    // script instead of being compiled into the binary.
    out.push_str(
        "pub struct Note {\n    pub section: &'static str,\n    pub url: &'static str,\n    pub title: &'static str,\n    pub description: &'static str,\n    pub html: &'static str,\n    pub toc: &'static [TocEntry],\n    pub minutes: usize,\n    pub words: usize,\n    /// `YYYY-MM-DD`, empty when the note does not say.\n    pub date: &'static str,\n    /// Last verified date, falling back to `date`.\n    pub updated: &'static str,\n    /// Days between the last verification and the build.\n    pub age_days: i64,\n    /// True once the note is old enough that its toolchain advice may have moved.\n    pub stale: bool,\n    pub tags: &'static [&'static str],\n    /// Tool versions this note was written against.\n    pub tested_with: &'static [&'static str],\n}\n\n",
    );
    out.push_str("pub struct TocEntry {\n    pub level: u32,\n    pub id: &'static str,\n    pub title: &'static str,\n}\n\n");

    out.push_str("pub static SECTIONS: &[Section] = &[\n");
    for section in sections {
        let _ = writeln!(
            out,
            "    Section {{ key: {}, title: {}, short: {}, description: {}, intro: {} }},",
            rust_str(&section.key),
            rust_str(&section.title),
            rust_str(&section.short),
            rust_str(&section.description),
            rust_str(&section.intro),
        );
    }
    out.push_str("];\n\n");

    out.push_str("pub static NOTES: &[Note] = &[\n");
    for note in notes {
        let ident = format!("TOC_{}", sanitize_ident(&note.section, &note.slug));
        // The table of contents is a block-scoped static, so the array
        // initializer stays a plain list of `Note` values.
        out.push_str("    {\n");
        let _ = writeln!(out, "        static {ident}: &[TocEntry] = &[");
        for entry in &note.toc {
            let _ = writeln!(
                out,
                "            TocEntry {{ level: {}, id: {}, title: {} }},",
                entry.level,
                rust_str(&entry.id),
                rust_str(&entry.title),
            );
        }
        out.push_str("        ];\n");
        let _ = writeln!(
            out,
            "        Note {{ section: {}, url: {}, title: {}, description: {}, html: {}, toc: {}, minutes: {}, words: {}, date: {}, updated: {}, age_days: {}, stale: {}, tags: &[{}], tested_with: &[{}] }}",
            rust_str(&note.section),
            rust_str(&note.url),
            rust_str(&note.title),
            rust_str(&note.description),
            rust_str(&note.html),
            ident,
            note.minutes,
            note.words,
            rust_str(&note.date),
            rust_str(&note.updated),
            note.age_days,
            note.stale,
            quoted_list(&note.tags),
            quoted_list(&note.tested_with),
        );
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out
}

/// `"a", "b"` 閳?the body of a `&[&str]` literal.
fn quoted_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| rust_str(item))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A Rust identifier for the block-scoped TOC static.
///
/// Anything that is not alphanumeric becomes an underscore, so a slug with a
/// space or a `+` cannot produce generated source that does not compile. A
/// collision between two slugs would be a compile error rather than silent
/// wrongness, which is the right way round.
fn sanitize_ident(section: &str, slug: &str) -> String {
    let mut out = String::from("TOC_");
    for ch in section
        .chars()
        .chain(std::iter::once('_'))
        .chain(slug.chars())
    {
        if ch.is_alphanumeric() {
            out.extend(ch.to_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// assets
// ---------------------------------------------------------------------------

/// Concatenated in this order; later files may override earlier ones.
///
/// `fonts.css` is generated by the font fetch and is already minified, so it is
/// appended verbatim (see `CSS_VERBATIM`).
const CSS_SOURCES: &[&str] = &[
    "styles/tokens.css",
    "styles/base.css",
    "styles/shell.css",
    "styles/home.css",
    "styles/shelf.css",
    "styles/note.css",
    "styles/extras.css",
    "styles/fonts.css",
];

/// Sources appended without minification.
const CSS_VERBATIM: &[&str] = &["styles/fonts.css"];

/// Hash of everything the versioned URLs point at, so any edit invalidates
/// them without anybody having to remember to bump a query string.
fn asset_version(manifest: &Path, font_version: &str) -> String {
    let mut bytes = Vec::new();
    for rel in CSS_SOURCES {
        if let Ok(css) = std::fs::read(manifest.join(rel)) {
            bytes.extend_from_slice(&css);
        }
    }
    if let Ok(js) = std::fs::read(manifest.join("scripts/app.js")) {
        bytes.extend_from_slice(&js);
    }
    bytes.extend_from_slice(font_version.as_bytes());
    short_hash(&bytes)
}

/// Hash of the concatenated contents of `paths`, skipping any that are absent.
fn short_hash_of(paths: &[PathBuf]) -> String {
    let mut bytes = Vec::new();
    for path in paths {
        if let Ok(contents) = std::fs::read(path) {
            bytes.extend_from_slice(&contents);
        }
    }
    short_hash(&bytes)
}

fn build_assets(manifest: &Path, font_version: &str) {
    build_css(manifest, font_version);

    // The client script is served as-is; no transform, no bundler.
    let js_src = manifest.join("scripts/app.js");
    let js_dest = manifest.join("assets/app.js");
    match std::fs::read(&js_src) {
        Ok(bytes) => {
            if let Err(err) = std::fs::write(&js_dest, bytes) {
                println!("cargo::warning=failed to write assets/app.js: {err}");
            }
        }
        Err(_) => println!("cargo::warning=scripts/app.js missing"),
    }

    copy_static(manifest);
}

fn build_css(manifest: &Path, font_version: &str) {
    let mut combined = String::new();
    let mut missing = Vec::new();

    for rel in CSS_SOURCES {
        match std::fs::read_to_string(manifest.join(rel)) {
            Ok(css) => {
                combined.push_str("/* ");
                combined.push_str(rel);
                combined.push_str(" */\n");
                if CSS_VERBATIM.contains(rel) {
                    // These sources are copied as-is, so a UTF-8 BOM in the
                    // file would land mid-stylesheet. A stray U+FEFF there
                    // makes the browser drop whatever rule follows it 閳?                    // which silently killed the first @font-face.
                    combined.push_str(&strip_bom(&css));
                } else {
                    combined.push_str(&minify_css(&css));
                }
                combined.push('\n');
            }
            Err(_) => missing.push(*rel),
        }
    }

    if !missing.is_empty() {
        println!("cargo::warning=missing css sources: {}", missing.join(", "));
    }
    if combined.is_empty() {
        return;
    }

    if combined.contains(FONT_VERSION_TOKEN) {
        combined = combined.replace(FONT_VERSION_TOKEN, font_version);
    }

    let dest = manifest.join("assets/main.css");
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(err) = std::fs::write(&dest, &combined) {
        println!("cargo::warning=failed to write assets/main.css: {err}");
    }
}

/// The client-side search index, as a real JSON file.
///
/// It used to be inlined into every response 閳?16 KB of JavaScript repeated on
/// every page, and truncated to the first 1600 characters of each note, so most
/// of a long note was unsearchable. One cached file is smaller, complete, and
/// only fetched when someone actually searches.
fn write_search_index(manifest: &Path, sections: &[Section], notes: &[Note]) {
    let short_of: HashMap<&str, &str> = sections
        .iter()
        .map(|s| (s.key.as_str(), s.short.as_str()))
        .collect();

    let mut json = String::from("{\"v\":1,\"docs\":[");
    for (index, note) in notes.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }

        // The haystack is folded to lowercase here. Search compares it against
        // a lowercased query on every keystroke, and the browser should not
        // have to redo that work for the same text on every visit. Headings and
        // tags are included so a search can match a section title or a tag,
        // not just the body prose.
        let mut haystack = String::new();
        for part in [
            note.title.as_str(),
            note.description.as_str(),
            &note.tags.join(" "),
            &note.tested_with.join(" "),
        ] {
            haystack.push_str(part);
            haystack.push(' ');
        }
        for entry in &note.toc {
            haystack.push_str(&entry.title);
            haystack.push(' ');
        }
        haystack.push_str(&note.plain);

        let _ = write!(
            json,
            "[{},{},{},{}]",
            json_string(&note.url),
            json_string(
                short_of
                    .get(note.section.as_str())
                    .copied()
                    .unwrap_or("Note")
            ),
            json_string(&note.title),
            json_string(&haystack.to_lowercase()),
        );
    }
    json.push_str("]}");

    let dest = manifest.join("assets/search-index.json");
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(err) = std::fs::write(&dest, json) {
        println!("cargo::warning=failed to write assets/search-index.json: {err}");
    }
}

/// The Atom feed, the XML sitemap and `robots.txt`.
///
/// Written as files rather than served from routes: the site's single catch-all
/// route makes a second root-level route a conflict waiting to happen, a static
/// export gets all three for free, and everything they need 閳?the absolute base
/// URL, the dates, the note list 閳?is already known here.
fn write_crawler_files(manifest: &Path, sections: &[Section], notes: &[Note], meta: &SiteMeta) {
    let dest = manifest.join("assets");
    let _ = std::fs::create_dir_all(&dest);

    write_file(&dest.join("feed.xml"), &atom_feed(notes, meta));
    write_file(&dest.join("sitemap.xml"), &sitemap(sections, notes, meta));
    write_file(&dest.join("robots.txt"), &robots(meta));
}

fn write_file(path: &Path, contents: &str) {
    if let Err(err) = std::fs::write(path, contents) {
        println!("cargo::warning=failed to write {}: {err}", path.display());
    }
}

/// Used when no note carries a date at all. Atom requires a feed-level
/// `updated`, and a clock reading would rewrite the feed on every build, so the
/// fallback is a fixed value plus a warning.
const DEFAULT_DATE: &str = "1970-01-01";

/// An Atom 1.0 feed of every dated note, newest first.
///
/// The old Zola site had `generate_feeds = true`; the port dropped it, which
/// silently removed the only way a returning reader could follow the notes.
fn atom_feed(notes: &[Note], meta: &SiteMeta) -> String {
    let mut ordered: Vec<&Note> = notes.iter().filter(|n| !n.updated.is_empty()).collect();
    ordered.sort_by(|a, b| b.updated.cmp(&a.updated).then_with(|| a.url.cmp(&b.url)));

    if ordered.is_empty() {
        println!("cargo::warning=no note has a `date`, so feed.xml lists no entries");
    }
    let feed_updated = ordered.first().map_or(DEFAULT_DATE, |n| n.updated.as_str());
    let base = &meta.base_url;

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    let _ = writeln!(out, "  <title>{}</title>", xml_escape(&meta.title));
    let _ = writeln!(
        out,
        "  <subtitle>{}</subtitle>",
        xml_escape(&meta.description)
    );
    let _ = writeln!(out, "  <id>{}/</id>", xml_escape(base));
    let _ = writeln!(
        out,
        "  <link href=\"{}/feed.xml\" rel=\"self\"/>",
        xml_escape(base)
    );
    let _ = writeln!(out, "  <link href=\"{}/\"/>", xml_escape(base));
    let _ = writeln!(out, "  <updated>{}</updated>", rfc3339(feed_updated));
    out.push_str("  <author><name>ref42</name></author>\n");

    for note in ordered {
        let url = xml_escape(&format!("{base}{}", note.url));
        out.push_str("  <entry>\n");
        let _ = writeln!(out, "    <title>{}</title>", xml_escape(&note.title));
        let _ = writeln!(out, "    <id>{url}</id>");
        let _ = writeln!(out, "    <link href=\"{url}\"/>");
        let _ = writeln!(out, "    <updated>{}</updated>", rfc3339(&note.updated));
        if !note.date.is_empty() {
            let _ = writeln!(out, "    <published>{}</published>", rfc3339(&note.date));
        }
        if !note.description.is_empty() {
            let _ = writeln!(
                out,
                "    <summary>{}</summary>",
                xml_escape(&note.description)
            );
        }
        for tag in &note.tags {
            let _ = writeln!(out, "    <category term=\"{}\"/>", xml_escape(tag));
        }
        out.push_str("  </entry>\n");
    }

    out.push_str("</feed>\n");
    out
}

/// `YYYY-MM-DD` as the RFC 3339 timestamp Atom and sitemaps expect.
fn rfc3339(date: &str) -> String {
    format!("{date}T00:00:00Z")
}

fn sitemap(sections: &[Section], notes: &[Note], meta: &SiteMeta) -> String {
    let base = &meta.base_url;
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    push_url(&mut out, &format!("{base}/"), "");
    for section in sections {
        // A shelf is as fresh as its freshest note.
        let lastmod = notes
            .iter()
            .filter(|n| n.section == section.key)
            .map(|n| n.updated.as_str())
            .max()
            .unwrap_or_default();
        push_url(&mut out, &format!("{base}/{}/", section.key), lastmod);
    }
    for note in notes {
        push_url(&mut out, &format!("{base}{}", note.url), &note.updated);
    }

    out.push_str("</urlset>\n");
    out
}

fn push_url(out: &mut String, loc: &str, lastmod: &str) {
    if lastmod.is_empty() {
        let _ = writeln!(out, "  <url><loc>{}</loc></url>", xml_escape(loc));
    } else {
        let _ = writeln!(
            out,
            "  <url><loc>{}</loc><lastmod>{lastmod}</lastmod></url>",
            xml_escape(loc)
        );
    }
}

fn robots(meta: &SiteMeta) -> String {
    format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
        meta.base_url
    )
}

/// Mirrors `static/` into `assets/`, which the server serves at the same URLs
/// the old site used.
///
/// `assets/` is a pure function of `static/` and the generated assets, so
/// anything in it that `static/` no longer contains is purged afterwards.
/// Without that, deleting a file under `static/` left its `assets/` copy being
/// served forever 閳?which is exactly how a stale `search.js` and a superseded
/// `styles/tokens.css` survived their deletions.
fn copy_static(manifest: &Path) {
    let from = manifest.join("static");
    let to = manifest.join("assets");
    if !from.is_dir() {
        return;
    }
    let mut wanted = Vec::new();
    if let Err(err) = copy_tree(&from, &to, GENERATED_ASSETS, &mut wanted) {
        // Bail out without purging: a partial copy must never be treated as
        // the authoritative list of files, or the purge would delete assets
        // that are still referenced.
        println!("cargo::warning=failed to mirror static/: {err}");
        return;
    }
    if let Err(err) = purge_stale(&to, &wanted, GENERATED_ASSETS) {
        println!("cargo::warning=failed to purge stale assets: {err}");
    }
}

fn copy_tree(
    from: &Path,
    to: &Path,
    skip: &[&str],
    wanted: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let src = entry.path();
        let dst = to.join(&name);

        if src.is_dir() {
            copy_tree(&src, &dst, skip, wanted)?;
        } else if src.is_file() {
            if skip.contains(&name.as_str()) {
                continue;
            }
            std::fs::copy(&src, &dst)?;
            wanted.push(dst);
        }
    }
    Ok(())
}

/// Removes regular files under `to` that are not in `wanted` and not skipped.
///
/// Only files are removed 閳?directories are left alone, so an empty leftover
/// directory is harmless and a symlinked tree cannot be followed out of
/// `assets/`. Generated assets are exempt because they have no `static/` source.
fn purge_stale(to: &Path, wanted: &[PathBuf], keep: &[&str]) -> std::io::Result<()> {
    for entry in std::fs::read_dir(to)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            purge_stale(&path, wanted, keep)?;
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if keep.contains(&name.as_str()) || wanted.contains(&path) {
            continue;
        }
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
