//! Pure text plumbing shared by the build script and the unit tests.
//!
//! `build.rs` is not a test target: `cargo test` never runs `#[cfg(test)]`
//! modules inside a build script (checked, not assumed). So the transforms that
//! rewrite content live here, where `src/main.rs` can include them under
//! `#[cfg(test)]` and `build.rs` can include the same file with `#[path]`.
//!
//! Nothing here may depend on a crate. The build script's dependencies
//! (comrak, syntect) are not available to the binary, and the binary's
//! (topcoat) is not available to the build script, so this is standard library
//! only.
//!
//! Every function below exists because something was wrong with the obvious
//! implementation. The comments say which — that is the point of the tests.

#![allow(dead_code)] // Included by two targets; each uses a different subset.

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// frontmatter
// ---------------------------------------------------------------------------

/// A frontmatter value: a scalar, or a list written inline or as `- item` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Scalar(String),
    List(Vec<String>),
}

impl Value {
    fn as_scalar(&self) -> Option<&str> {
        match self {
            Value::Scalar(value) => Some(value),
            // A one-element list is a scalar as far as `get` is concerned.
            Value::List(items) => items.first().map(String::as_str),
        }
    }
}

/// The parsed frontmatter block, keyed by field name.
#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    entries: BTreeMap<String, Value>,
}

impl Frontmatter {
    /// A scalar field. For a list field this returns the first item.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).and_then(Value::as_scalar)
    }

    /// A list field. A scalar is treated as a one-element list, so `tags: rust`
    /// and `tags: [rust]` behave the same.
    pub fn list(&self, key: &str) -> Vec<String> {
        match self.entries.get(key) {
            Some(Value::List(items)) => items.clone(),
            Some(Value::Scalar(value)) if !value.trim().is_empty() => vec![value.clone()],
            _ => Vec::new(),
        }
    }

    /// A `usize` field, for `weight` and `order`.
    pub fn number(&self, key: &str) -> Option<i64> {
        self.get(key)?.trim().parse().ok()
    }

    pub fn has(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.entries.insert(key.to_string(), value);
    }
}

/// Splits `---` frontmatter from the body.
///
/// Line endings are normalised first. `content/` used to be checked out with
/// CRLF while some files were written with LF, so a fence line could read as
/// "```\r" and silently defeat every later scanner — including the code-fence
/// detection in [`join_multiline_tags`]. `.gitattributes` now pins the tree to
/// LF; the normalisation stays because content can arrive from anywhere.
pub fn split_frontmatter(src: &str) -> (Frontmatter, String) {
    let normalised = src.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalised.trim_start_matches('\u{feff}');

    let mut frontmatter = Frontmatter::default();

    if trimmed.lines().next().map(str::trim) != Some("---") {
        return (frontmatter, trimmed.to_string());
    }

    // Everything after the opening fence line. That line is not necessarily
    // exactly `---\n` (it can carry trailing spaces), so its end is located
    // rather than assumed — assuming a fixed four bytes silently shifted the
    // body by the difference.
    let Some(newline) = trimmed.find('\n') else {
        return (frontmatter, String::new());
    };
    let after_open = &trimmed[newline + 1..];

    let mut body_start = None;
    let mut cursor = 0usize;
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut block_list: Option<(String, Vec<String>)> = None;

    for line in after_open.lines() {
        cursor += line.len() + 1; // + the newline `lines()` dropped
        let trimmed_line = line.trim();

        // The closing fence is a line that is nothing but `---`. Searching for
        // a leading "\n---" instead would also split on a value that happens to
        // start with a dash trio.
        if trimmed_line == "---" {
            body_start = Some(cursor);
            break;
        }

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        // A block list item belongs to the key above it.
        if let Some(item) = trimmed_line.strip_prefix("- ") {
            if let Some((_, items)) = block_list.as_mut() {
                items.push(unquote(item.trim()));
            }
            continue;
        }

        let Some((key, value)) = trimmed_line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value.trim();

        // Flush a block list when a new key starts.
        if let Some((list_key, items)) = block_list.take() {
            fields.push((list_key, format!("[{}]", items.join(", "))));
        }

        if value.is_empty() {
            // `key:` with nothing after it starts a block list.
            block_list = Some((key, Vec::new()));
        } else {
            fields.push((key, value.to_string()));
        }
    }

    if let Some((list_key, items)) = block_list.take() {
        fields.push((list_key, format!("[{}]", items.join(", "))));
    }

    for (key, raw) in fields {
        frontmatter.entries.insert(key, parse_value(&raw));
    }

    let body = body_start
        .and_then(|offset| after_open.get(offset..))
        .unwrap_or_default()
        .to_string();

    (frontmatter, body)
}

/// Parses a raw value: an inline list, or a scalar with optional quotes.
fn parse_value(raw: &str) -> Value {
    let raw = raw.trim();

    if let Some(inner) = raw
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        return Value::List(split_inline_list(inner));
    }

    Value::Scalar(unquote(raw))
}

/// Splits `a, b, "c, d"` on commas that are not inside quotes.
fn split_inline_list(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in inner.chars() {
        match quote {
            Some(q) if ch == q => {
                quote = None;
                current.push(ch);
            }
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            None if ch == ',' => {
                items.push(unquote(current.trim()));
                current.clear();
            }
            None => current.push(ch),
        }
    }
    items.push(unquote(current.trim()));

    items.retain(|item| !item.is_empty());
    items
}

/// Strips one layer of matching single or double quotes.
fn unquote(value: &str) -> String {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

/// True for a `YYYY-MM-DD` date, which is the only form the site accepts.
pub fn is_date(value: &str) -> bool {
    let value = value.trim();
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// True for the values a frontmatter flag is written as: `true`, `yes`, `on`, `1`.
///
/// Frontmatter here is a map of strings rather than parsed YAML, so there is no
/// boolean type to read. This is what decides that `draft: true` is a draft and
/// `draft: no` — or a typo like `draft: ture` — is not, which matters because
/// the failure mode of guessing wrong is publishing something unfinished.
pub fn is_truthy(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "true" | "yes" | "on" | "1"
        )
    })
}

/// A human title for a section that is named only by its folder:
/// `dev-tools` and `dev_tools` both become `Dev Tools`.
///
/// Used when a directory of notes has no `_index.md`. Authoring a note should
/// not require knowing that a folder needs a metadata file first, so a folder
/// with notes in it is published under its own name instead of being skipped.
pub fn title_from_key(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    let mut start_of_word = true;

    for ch in key.chars() {
        if ch == '-' || ch == '_' || ch.is_whitespace() {
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            start_of_word = true;
            continue;
        }
        if start_of_word {
            out.extend(ch.to_uppercase());
            start_of_word = false;
        } else {
            out.push(ch);
        }
    }

    out.trim().to_string()
}

// ---------------------------------------------------------------------------
// authoring: what `ref42 new` writes

/// A filename from a title: `Flash an ESP32-C6!` -> `flash_an_esp32_c6`.
///
/// Deliberately not `slugify`, which is the heading-anchor rule a few hundred
/// lines down. That one keeps CJK characters, because an anchor a reader can
/// read beats a transliterated one, and separates with hyphens. A filename is
/// different: it becomes a path on someone's disk and a URL, so this is ASCII
/// only, and it uses underscores because the notes with long titles already do
/// (`rust_dev_base_env`, `esp32c3_dht11`). It can come back empty, which is the
/// command's cue to ask for `--slug` rather than write a file called `.md`.
pub fn filename_from_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }

    out.trim_end_matches('_').to_string()
}

/// The weight that puts a new note after everything already in its section.
///
/// Existing notes use 10, 20, 30, so this keeps the gaps: the next note gets
/// max + 10, and inserting one between two others is a matter of typing 15
/// rather than renumbering the section.
pub fn next_weight(existing: &[i64]) -> i64 {
    existing.iter().copied().max().unwrap_or(0) + 10
}

/// A frontmatter value, quoted so the parser hands back exactly what went in.
///
/// Single quotes unless the value contains one, because that is what the notes
/// already look like and this parser only strips a matching pair. Falling back
/// to double quotes matters: `Don't panic` in single quotes round-trips here but
/// would not survive a real YAML parser, and swapping to double quotes keeps the
/// file valid for both.
pub fn quote_value(value: &str) -> String {
    if value.contains('\'') && !value.contains('"') {
        format!("\"{value}\"")
    } else {
        format!("'{value}'")
    }
}

/// The whole file a new note starts as.
///
/// Here rather than in the command so the shape of a note is in one place and
/// can be asserted: the fields are the ones that change what the site does
/// (`weight` orders it, `date` puts it in the feed, `description` is what the
/// catalogue and search show), and the body is short enough to be obviously
/// disposable.
pub fn note_template(title: &str, weight: i64, date: &str) -> String {
    format!(
        "---\n\
         title: {title}\n\
         description: ''\n\
         weight: {weight}\n\
         date: \"{date}\"\n\
         ---\n\
         \n\
         One or two sentences on what this note covers, and why.\n\
         \n\
         ## First heading\n\
         \n\
         Text.\n",
        title = quote_value(title),
    )
}

// ---------------------------------------------------------------------------
// markdown source preparation
// ---------------------------------------------------------------------------

/// Collapses multi-line raw-HTML tags in the markdown source onto one line.
///
/// comrak only treats a raw-HTML tag as raw when it sits on a single line. A
/// tag broken across lines — which is how the notes format `<img … />` — is
/// escaped instead, dropping its quotes so it renders as `src= /path` with the
/// attribute mangled. Joining the tag first keeps the original attributes.
pub fn join_multiline_tags(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut chars = md.char_indices().peekable();
    let mut in_fence = false;
    let mut line_start = true;

    while let Some((_, ch)) = chars.next() {
        if line_start {
            // Fenced code blocks: never rewrite their contents.
            let fence = ch == '`' || ch == '~';
            if fence {
                in_fence = !in_fence;
            }
        }

        if ch == '<' && !in_fence && !inside_code_span(&out) {
            let mut tag = String::from('<');
            let mut closed = false;
            let mut quote: Option<char> = None;

            while let Some(&(_, next)) = chars.peek() {
                chars.next();
                tag.push(next);
                match quote {
                    Some(q) if next == q => quote = None,
                    Some(_) => {}
                    None if next == '"' || next == '\'' => quote = Some(next),
                    None if next == '>' => {
                        closed = true;
                        break;
                    }
                    None if next == '\n' && tag.len() > 4000 => break,
                    None => {}
                }
            }

            if closed {
                // Join onto one line, keeping a single space between tokens.
                let joined = tag.split_whitespace().collect::<Vec<_>>().join(" ");
                out.push_str(&joined);
            } else {
                out.push_str(&tag);
            }
            line_start = false;
            continue;
        }

        line_start = ch == '\n';
        out.push(ch);
    }

    out
}

/// True when the text written so far ends inside an unterminated inline code
/// span, where `<` is literal text rather than markup.
fn inside_code_span(out: &str) -> bool {
    let last_line = out.rsplit('\n').next().unwrap_or_default();
    last_line.matches('`').count() % 2 == 1
}

// ---------------------------------------------------------------------------
// HTML post-processing
// ---------------------------------------------------------------------------

/// One entry of a note's table of contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub level: u32,
    pub id: String,
    pub title: String,
}

/// `<pre><code class="language-x">..</code></pre>` becomes
/// `<pre data-lang="x"><code class="language-x">..</code></pre>`; the code
/// toolbar itself is built by `app.js` so the copy button stays a progressive
/// enhancement.
pub fn wrap_code_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;

    while let Some(pos) = rest.find("<pre") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];

        // The next characters are either `>` or ` data-lang=...>`.
        let Some(gt) = after.find('>') else {
            out.push_str(after);
            return out;
        };
        let open = &after[..=gt];

        // The language comes from the `language-x` class the highlighter
        // writes on the `<code>` element just after the opening `<pre>`.
        let lang = if open.contains("data-lang") {
            None
        } else {
            after[gt..]
                .find("language-")
                .filter(|rel| *rel < 400)
                .map(|rel| {
                    let start = gt + rel + "language-".len();
                    let end = after[start..]
                        .find(|c: char| {
                            !c.is_ascii_alphanumeric() && c != '+' && c != '#' && c != '-'
                        })
                        .map(|e| start + e)
                        .unwrap_or(after.len());
                    after[start..end].replace('"', "")
                })
        };

        match lang {
            Some(lang) if !lang.is_empty() => {
                out.push_str("<pre data-lang=\"");
                out.push_str(&lang);
                out.push_str("\">");
            }
            _ => out.push_str(open),
        }
        rest = &after[gt + 1..];
    }

    out.push_str(rest);
    out
}

/// Gives every `<h2>`/`<h3>`/`<h4>` a stable `id` plus a hover anchor link.
///
/// comrak 0.55 no longer writes heading ids itself, and the table of contents
/// is derived from those ids, so the build owns both sides and they always
/// agree.
pub fn add_heading_anchors(html: &str) -> (String, Vec<TocEntry>) {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    let mut toc = Vec::new();

    while let Some(pos) = rest.find("<h") {
        let after = &rest[pos..];
        let level = match after.as_bytes().get(2) {
            Some(b'2') => 2u8,
            Some(b'3') => 3,
            Some(b'4') => 4,
            // Not a heading we anchor. `<hr>` and `<html>` land here too.
            _ => {
                out.push_str(&rest[..pos + 2]);
                rest = &after[2..];
                continue;
            }
        };

        // The opening tag is `<hN>` or `<hN ...>`.
        let Some(open_end) = after.find('>') else {
            break;
        };
        let open = &after[..open_end];
        let after_open = &after[open_end + 1..];
        // Locate the whole `</hN>` closing tag rather than advancing a fixed
        // byte count, which left a stray `>` after every heading.
        let close_pat = format!("</h{level}>");
        let Some(close_pos) = after_open.find(&close_pat) else {
            break;
        };
        let inner = &after_open[..close_pos];

        let title = strip_html(inner);
        let base = slugify(&title, level);
        let id = unique_slug(&base, &mut used);

        out.push_str(&rest[..pos]);
        if open.contains(" id=") {
            out.push_str(open);
        } else {
            out.push_str(&format!("<h{level} id=\"{id}\""));
            // Preserve any attributes comrak did emit.
            let attrs = open[format!("<h{level}").len()..].trim();
            if !attrs.is_empty() {
                out.push(' ');
                out.push_str(attrs);
            }
        }
        out.push('>');
        out.push_str(inner);
        out.push_str(&format!(
            "<a class=\"anchor\" href=\"#{id}\" aria-label=\"Link to this section\">#</a>"
        ));
        out.push_str(&format!("</h{level}>"));

        if !title.is_empty() {
            toc.push(TocEntry {
                level: level as u32,
                id,
                title,
            });
        }

        // Resume right after `</hN>`.
        rest = &after_open[close_pos + close_pat.len()..];
    }

    out.push_str(rest);
    (out, toc)
}

/// A URL-safe heading slug that keeps CJK characters, so Chinese headings get
/// readable anchors rather than empty ones.
pub fn slugify(title: &str, level: u8) -> String {
    let mut slug = String::with_capacity(title.len() + 2);
    let mut last_dash = false;

    for ch in title.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                slug.push(lower);
            }
            last_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        // Everything else (punctuation, emoji) is dropped.
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        slug = format!("section-{level}");
    }
    slug
}

/// Appends `-1`, `-2`, ... when the same heading appears twice on a page.
pub fn unique_slug(base: &str, used: &mut BTreeMap<String, usize>) -> String {
    let entry = used.entry(base.to_string()).or_insert(0);
    let slug = if *entry == 0 {
        base.to_string()
    } else {
        format!("{base}-{entry}")
    };
    *entry += 1;
    slug
}

/// comrak emits bare `<table>`; the stylesheet wants a rounded, scrollable
/// container.
///
/// The table is wrapped in a focusable scroll container so a keyboard user can
/// actually reach the overflow — a `div` with `overflow-x: auto` is otherwise
/// unreachable without a pointer.
pub fn wrap_tables(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;

    while let Some(pos) = rest.find("<table>") {
        let after = &rest[pos..];
        let Some(end) = after.find("</table>") else {
            break;
        };
        out.push_str(&rest[..pos]);
        out.push_str("<div class=\"table-wrap\" role=\"region\" tabindex=\"0\">");
        out.push_str(&after[..end + "</table>".len()]);
        out.push_str("</div>");
        rest = &after[end + "</table>".len()..];
    }

    out.push_str(rest);
    out
}

/// Strips tags and decodes the handful of entities comrak emits, for search
/// text and reading-time counts.
pub fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut entity = String::new();
    let mut in_entity = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            '&' if !in_tag => {
                in_entity = true;
                entity.clear();
            }
            ';' if in_entity => {
                in_entity = false;
                out.push(match entity.as_str() {
                    "amp" => '&',
                    "lt" => '<',
                    "gt" => '>',
                    "quot" => '"',
                    "apos" => '\'',
                    "nbsp" => ' ',
                    _ => ' ',
                });
            }
            _ if in_tag => {}
            _ if in_entity => {
                if entity.len() < 8 {
                    entity.push(ch);
                } else {
                    in_entity = false;
                    out.push(' ');
                }
            }
            _ => out.push(ch),
        }
    }

    let mut collapsed = String::with_capacity(out.len());
    let mut last_space = false;
    for ch in out.chars() {
        if ch.is_whitespace() {
            if !last_space {
                collapsed.push(' ');
            }
            last_space = true;
        } else {
            collapsed.push(ch);
            last_space = false;
        }
    }
    collapsed.trim().to_string()
}

/// Counts latin words plus one per CJK codepoint, so Chinese notes get a
/// meaningful reading time.
pub fn count_words(text: &str) -> usize {
    let mut latin = 0usize;
    let mut cjk = 0usize;
    let mut in_word = false;

    for ch in text.chars() {
        let is_cjk = matches!(ch as u32,
            0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0xAC00..=0xD7AF);
        if is_cjk {
            cjk += 1;
            in_word = false;
        } else if ch.is_alphanumeric() {
            if !in_word {
                latin += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }

    latin + cjk
}

// ---------------------------------------------------------------------------
// images
// ---------------------------------------------------------------------------

/// What the build knows about an image file it found under `static/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    /// URL of the WebP sibling, when one sits next to the source file.
    pub webp: Option<String>,
}

/// Pixel dimensions of a PNG, JPEG or GIF, or `None` for anything else.
///
/// The build reads the header itself rather than pulling in an image crate: it
/// needs two integers, and the alternative is a dependency that compiles a
/// decoder the site never runs.
pub fn image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        // IHDR is required to be the first chunk: 8 signature, 4 length,
        // 4 type, then width and height, both big-endian.
        let width = be_u32(bytes.get(16..20)?)?;
        let height = be_u32(bytes.get(20..24)?)?;
        return Some((width, height));
    }

    if bytes.starts_with(&[0xFF, 0xD8]) {
        return jpeg_dimensions(bytes);
    }

    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        let width = u16::from_le_bytes([*bytes.get(6)?, *bytes.get(7)?]) as u32;
        let height = u16::from_le_bytes([*bytes.get(8)?, *bytes.get(9)?]) as u32;
        return Some((width, height));
    }

    None
}

fn be_u32(slice: &[u8]) -> Option<u32> {
    let slice: [u8; 4] = slice.try_into().ok()?;
    Some(u32::from_be_bytes(slice))
}

/// Walks JPEG segments to the frame header, which is the only place the
/// dimensions live.
fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2usize;

    while i + 3 < bytes.len() {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];

        // Standalone markers carry no length.
        if marker == 0xFF || marker == 0x01 || (0xD0..=0xD9).contains(&marker) {
            i += 2;
            continue;
        }

        let length = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        // SOF0..SOF15, excluding DHT (C4), JPG (C8) and DAC (CC).
        let is_sof =
            (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC;
        if is_sof {
            let height = u16::from_be_bytes([*bytes.get(i + 5)?, *bytes.get(i + 6)?]) as u32;
            let width = u16::from_be_bytes([*bytes.get(i + 7)?, *bytes.get(i + 8)?]) as u32;
            return Some((width, height));
        }

        if length < 2 {
            return None;
        }
        i += 2 + length;
    }

    None
}

/// One attribute of an HTML start tag.
struct Attr {
    name: String,
    /// `None` for a bare attribute such as `hidden`.
    value: Option<String>,
}

/// Rewrites every `<img>` in a rendered note:
///
/// * `width`/`height` from the real file, so the browser reserves the box and
///   the page does not jump as screenshots arrive (the site had no intrinsic
///   sizes at all, so every image was a layout shift),
/// * `loading="lazy" decoding="async"` on everything except the first image,
///   which is usually the one above the fold and would hurt LCP if deferred,
/// * a `<picture>` with a WebP `<source>` when a `.webp` sibling exists.
///
/// `lookup` maps a URL path such as `/utils/seex/1.png` to what the build knows
/// about that file. Images it does not recognise (remote URLs, files that are
/// not there) are left exactly as they were.
pub fn rewrite_images(html: &str, lookup: impl Fn(&str) -> Option<ImageInfo>) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;
    let mut seen = 0usize;

    while let Some(pos) = rest.find("<img") {
        // Only treat it as a tag if the next byte ends the name: `<img` in
        // prose would otherwise be rewritten. Escaped code is `&lt;img`, so it
        // never reaches here.
        let after = &rest[pos..];
        if !matches!(
            after.as_bytes().get(4),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'/') | Some(b'>')
        ) {
            out.push_str(&rest[..pos + 4]);
            rest = &after[4..];
            continue;
        }

        let Some(gt) = find_tag_end(after) else {
            out.push_str(&rest[..pos]);
            rest = after;
            break;
        };

        out.push_str(&rest[..pos]);
        let inner = &after[4..gt];
        let attrs = parse_attrs(inner);

        let src = attrs
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("src"))
            .and_then(|a| a.value.as_deref());
        let info = src.and_then(&lookup);

        let has = |name: &str| attrs.iter().any(|a| a.name.eq_ignore_ascii_case(name));

        let mut extra: Vec<Attr> = Vec::new();
        let is_first = seen == 0;
        seen += 1;

        if let Some(info) = &info {
            // A hand-written width (the logos set one) wins, but a missing
            // height is still filled in from the file's aspect ratio.
            if !has("width") && !has("height") {
                extra.push(Attr {
                    name: "width".into(),
                    value: Some(info.width.to_string()),
                });
                extra.push(Attr {
                    name: "height".into(),
                    value: Some(info.height.to_string()),
                });
            }
        }

        if !has("decoding") {
            extra.push(Attr {
                name: "decoding".into(),
                value: Some("async".into()),
            });
        }
        if !has("loading") && !is_first {
            extra.push(Attr {
                name: "loading".into(),
                value: Some("lazy".into()),
            });
        }

        let mut tag = String::from("<img");
        for attr in attrs.iter().chain(extra.iter()) {
            tag.push(' ');
            tag.push_str(&attr.name);
            if let Some(value) = &attr.value {
                tag.push_str("=\"");
                tag.push_str(&escape_attr(value));
                tag.push('"');
            }
        }
        tag.push('>');

        match info.and_then(|info| info.webp) {
            Some(webp) => {
                out.push_str("<picture><source type=\"image/webp\" srcset=\"");
                out.push_str(&escape_attr(&webp));
                out.push_str("\">");
                out.push_str(&tag);
                out.push_str("</picture>");
            }
            None => out.push_str(&tag),
        }

        rest = &after[gt + 1..];
    }

    out.push_str(rest);
    out
}

/// The `src` of every `<img>` that has no `alt` attribute, for a build warning.
///
/// A missing `alt` is invisible until someone uses a screen reader, so it is
/// worth failing loudly at build time rather than shipping it.
pub fn images_without_alt(html: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let mut rest = html;

    while let Some(pos) = rest.find("<img") {
        let after = &rest[pos..];
        let Some(gt) = find_tag_end(after) else {
            break;
        };
        let attrs = parse_attrs(&after[4..gt]);
        if !attrs.iter().any(|a| a.name.eq_ignore_ascii_case("alt")) {
            let src = attrs
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case("src"))
                .and_then(|a| a.value.clone())
                .unwrap_or_else(|| "(no src)".to_string());
            missing.push(src);
        }
        rest = &after[gt + 1..];
    }

    missing
}

/// Index of the `>` that closes a start tag, ignoring any inside quotes.
fn find_tag_end(tag: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, ch) in tag.char_indices() {
        match quote {
            Some(q) if ch == q => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '>' => return Some(i),
            None => {}
        }
    }
    None
}

/// Parses the inside of a start tag into attributes, preserving order.
fn parse_attrs(inner: &str) -> Vec<Attr> {
    let mut attrs = Vec::new();
    let bytes: Vec<char> = inner.chars().collect();
    let mut i = 0usize;

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        let mut name = String::new();
        while i < bytes.len() && !bytes[i].is_whitespace() && bytes[i] != '=' {
            name.push(bytes[i]);
            i += 1;
        }

        while i < bytes.len() && bytes[i].is_whitespace() {
            i += 1;
        }

        let mut value = None;
        if i < bytes.len() && bytes[i] == '=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_whitespace() {
                i += 1;
            }
            let mut parsed = String::new();
            if i < bytes.len() && (bytes[i] == '"' || bytes[i] == '\'') {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    parsed.push(bytes[i]);
                    i += 1;
                }
                i += 1; // closing quote
            } else {
                while i < bytes.len() && !bytes[i].is_whitespace() {
                    parsed.push(bytes[i]);
                    i += 1;
                }
            }
            value = Some(parsed);
        }

        if !name.is_empty() {
            attrs.push(Attr { name, value });
        }
    }

    attrs
}

/// Escapes a value for a double-quoted HTML attribute.
fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ---------------------------------------------------------------------------
// JSON, for the search index
// ---------------------------------------------------------------------------

/// Escapes a string as a JSON string literal, including the surrounding quotes.
///
/// The search index used to be written by hand into a JavaScript string, where
/// the only defence was stripping `"` and `\`. A real `.json` file has to be
/// valid JSON, so control characters are escaped properly and non-ASCII is left
/// as UTF-8, which JSON allows and the browser decodes back.
pub fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            ch if (ch as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// Escapes a string as a JSON literal that is also safe to inline in a
/// `<script>` element.
///
/// [`json_string`] is correct for a `.json` file, but an inline
/// `application/ld+json` block ends at the first `</script>` — and a note title
/// is allowed to contain that. Escaping `<`, `>` and `&` as `\uXXXX` leaves the
/// parsed value identical while making the byte sequence impossible.
pub fn json_string_for_script(value: &str) -> String {
    json_string(value)
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

/// Escapes text for an XML text node or attribute value.
///
/// The feed and the sitemap are XML: one unescaped `&` in a note title makes the
/// whole document unparseable, and a feed reader then shows nothing at all
/// rather than something slightly wrong.
pub fn xml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            ch => out.push(ch),
        }
    }
    out
}

/// A short, stable content hash for cache-busting asset URLs.
///
/// FNV-1a rather than a cryptographic digest: this only has to change when the
/// bytes change, and it keeps the build script dependency-free. It is not used
/// for anything security-sensitive, and the comment saying so matters — if this
/// ever guards a signature, it has to be replaced.
pub fn short_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Masked to 48 bits so the hex form is always exactly twelve characters:
    // `{:012x}` is a *minimum* width, and a full 64-bit value prints sixteen.
    format!("{:012x}", hash & 0x0000_FFFF_FFFF_FFFF)
}

// ---------------------------------------------------------------------------
// css
// ---------------------------------------------------------------------------

/// Drops a leading UTF-8 BOM (and any stray zero-width no-break space).
///
/// `tools/fetch-fonts.ps1` ran under Windows PowerShell 5.1, whose
/// `Set-Content -Encoding UTF8` writes a BOM. Concatenating that file into
/// `main.css` put a U+FEFF in the middle of the stylesheet, where it makes the
/// browser drop whatever rule follows it — which silently killed the first
/// `@font-face`.
pub fn strip_bom(css: &str) -> String {
    css.trim_start_matches('\u{feff}').replace('\u{feff}', "")
}

/// Small dependency-free CSS minifier: strips comments and collapses runs of
/// whitespace down to the minimum that keeps the stylesheet meaning-preserving.
///
/// Whitespace is significant in two places: it can be the descendant
/// combinator in a selector (`.a .b`), and it can be part of a value
/// (`width: 100% - 2rem`). It is *not* significant directly after `}`, `;`,
/// `{`, `,` or `>`, and dropping it there is what actually shrinks the file.
pub fn minify_css(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    let mut pending_space = false;
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        if let Some(q) = quote {
            out.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if ch == q {
                quote = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            out.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = '\0';
            for c in chars.by_ref() {
                if prev == '*' && c == '/' {
                    break;
                }
                prev = c;
            }
            pending_space = true;
            continue;
        }

        if ch.is_whitespace() {
            pending_space = true;
            continue;
        }

        if pending_space {
            let last = out.chars().last();
            // Whitespace directly before these tokens never changes meaning.
            //
            // `:` is deliberately absent even though a declaration rarely needs
            // a space after it: in a selector the space *before* a colon is the
            // descendant combinator, so collapsing `a :hover` to `a:hover`
            // would silently change which element the rule matches. A few
            // hundred bytes are not worth that risk.
            let drop = matches!(ch, '}' | ')' | ']' | ',' | ';' | '{' | '>')
                // ...and neither does whitespace directly after them.
                || matches!(last, None | Some('{') | Some('}') | Some(';') | Some(','));
            if !drop {
                out.push(' ');
            }
            pending_space = false;
        }

        out.push(ch);
    }

    out
}

// ---------------------------------------------------------------------------
// dates and fuzzy matching
// ---------------------------------------------------------------------------

/// Days from 1970-01-01 to a civil date, proleptic Gregorian.
///
/// Howard Hinnant's `days_from_civil`: era arithmetic keeps leap years correct
/// without a table. It lives here, with tests, because the staleness badge hangs
/// off it and an off-by-one-day date bug is exactly the kind that survives
/// review.
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (month + 9) % 12; // March = 0
    let doy = (153 * mp as i64 + 2) / 5 + day as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Parses the site's only accepted date form, `YYYY-MM-DD`.
pub fn parse_date(value: &str) -> Option<(i64, u32, u32)> {
    let value = value.trim();
    if !is_date(value) {
        return None;
    }
    Some((
        value[0..4].parse().ok()?,
        value[5..7].parse().ok()?,
        value[8..10].parse().ok()?,
    ))
}

/// Whole days from `date` to `today_days`, where the latter is days since the
/// epoch — how the build passes "now" without a calendar conversion.
pub fn age_in_days(date: &str, today_days: i64) -> Option<i64> {
    let (year, month, day) = parse_date(date)?;
    Some(today_days - days_from_civil(year, month, day))
}

/// The inverse of `days_from_civil`: days since the epoch back to a calendar date.
///
/// Howard Hinnant's `civil_from_days`, the same algorithm in the other
/// direction. It exists so a note can be dated today without pulling a date
/// crate into a site that needs exactly one date format.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365; // [0, 399]
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]
    let shifted_month = (5 * day_of_year + 2) / 153; // [0, 11], March-based
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32; // [1, 31]
    let month = (if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    }) as u32; // [1, 12]

    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Today, as `YYYY-MM-DD`, from the system clock.
///
/// The clock only has to be right to the day, so this reads seconds since the
/// epoch rather than dragging in timezone handling — a note written at 00:30
/// will be dated by UTC, which is the same trade `build.rs` already makes when
/// it computes how stale a note is.
pub fn today_date() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// "today", "yesterday", "3 days ago", "2 months ago", "a year ago" — how long
/// since a note was last verified, which is the one fact a toolchain guide has
/// to be honest about, and the reason the badge exists.
///
/// Deliberately coarse: "2 months ago" is what a reader needs, and a day count
/// on old content reads as false precision.
pub fn humanize_age(days: i64) -> String {
    if days <= 0 {
        return "today".to_string();
    }
    match days {
        1 => "yesterday".to_string(),
        2..=29 => format!("{days} days ago"),
        30..=59 => "a month ago".to_string(),
        60..=364 => format!("{} months ago", (days as f64 / 30.44).round() as i64),
        365..=729 => "a year ago".to_string(),
        _ => format!("{} years ago", (days as f64 / 365.25).round() as i64),
    }
}

/// Edit distance between two strings.
///
/// Two rows of the dynamic-programming table rather than the full matrix: the
/// inputs are single words, and callers skip pairs whose lengths differ by more
/// than the threshold before paying for this at all.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[b.len()]
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- frontmatter ----------------------------------------------------

    #[test]
    fn frontmatter_splits_scalars() {
        let src =
            "---\ntitle: 'SeEx: desktop EDA library exporter'\nweight: 20\n---\n\nBody here.\n";
        let (fm, body) = split_frontmatter(src);

        assert_eq!(fm.get("title"), Some("SeEx: desktop EDA library exporter"));
        assert_eq!(fm.number("weight"), Some(20));
        assert_eq!(body.trim(), "Body here.");
    }

    #[test]
    fn frontmatter_survives_crlf_and_bom() {
        let src = "\u{feff}---\r\ntitle: \"A\"\r\n---\r\nBody\r\n";
        let (fm, body) = split_frontmatter(src);

        assert_eq!(fm.get("title"), Some("A"));
        assert_eq!(body.trim(), "Body");
        // No carriage return may survive into the body: comrak would treat it
        // as part of the text and the code-fence scanner would miss fences.
        assert!(!body.contains('\r'));
    }

    #[test]
    fn frontmatter_without_fence_is_all_body() {
        let (fm, body) = split_frontmatter("# Just markdown\n");
        assert!(fm.get("title").is_none());
        assert_eq!(body, "# Just markdown\n");
    }

    #[test]
    fn frontmatter_does_not_split_on_dashes_inside_a_value() {
        let src = "---\ntitle: \"a---b\"\nweight: 1\n---\nBody\n";
        let (fm, body) = split_frontmatter(src);

        assert_eq!(fm.get("title"), Some("a---b"));
        assert_eq!(fm.number("weight"), Some(1));
        assert_eq!(body.trim(), "Body");
    }

    #[test]
    fn frontmatter_reads_inline_lists() {
        let src = "---\ntags: [esp32, \"rust embedded\"]\n---\n";
        let (fm, _) = split_frontmatter(src);

        assert_eq!(fm.list("tags"), vec!["esp32", "rust embedded"]);
    }

    #[test]
    fn frontmatter_reads_block_lists() {
        let src = "---\ntags:\n  - esp32\n  - sensor\ntitle: \"X\"\n---\n";
        let (fm, _) = split_frontmatter(src);

        assert_eq!(fm.list("tags"), vec!["esp32", "sensor"]);
        assert_eq!(fm.get("title"), Some("X"));
    }

    #[test]
    fn frontmatter_treats_a_lone_scalar_as_a_one_item_list() {
        let (fm, _) = split_frontmatter("---\ntags: rust\n---\n");
        assert_eq!(fm.list("tags"), vec!["rust"]);
        assert_eq!(fm.list("missing"), Vec::<String>::new());
    }

    #[test]
    fn section_titles_come_from_folder_names() {
        assert_eq!(title_from_key("messy"), "Messy");
        assert_eq!(title_from_key("dev-tools"), "Dev Tools");
        assert_eq!(title_from_key("dev_tools"), "Dev Tools");
        assert_eq!(title_from_key("ESP32"), "ESP32");
        assert_eq!(title_from_key("cortex-m"), "Cortex M");
        // A name that is only separators must not produce a blank title.
        assert_eq!(title_from_key("--"), "");
    }

    #[test]
    fn titles_become_filenames() {
        assert_eq!(
            filename_from_title("Flash an ESP32-C6!"),
            "flash_an_esp32_c6"
        );
        assert_eq!(filename_from_title("Cortex-M + Rust"), "cortex_m_rust");
        assert_eq!(
            filename_from_title("npnp: EDA export CLI"),
            "npnp_eda_export_cli"
        );
        assert_eq!(filename_from_title("  spaced  out  "), "spaced_out");
        assert_eq!(filename_from_title("already_slugged"), "already_slugged");
        // Nothing usable is left, and the command has to notice rather than
        // write a file called `.md`.
        assert_eq!(filename_from_title("!!!"), "");
        assert_eq!(filename_from_title(""), "");
        // No doubled or trailing separators.
        assert!(!filename_from_title("a -- b").contains("__"));
        assert!(!filename_from_title("trailing!").ends_with('_'));
        // Unlike a heading anchor, a filename stays ASCII: the URL is shared and
        // the file has to survive every filesystem it lands on.
        assert_eq!(filename_from_title("嵌入式 Rust"), "rust");
    }

    #[test]
    fn a_new_note_sorts_after_the_others() {
        assert_eq!(next_weight(&[]), 10);
        assert_eq!(next_weight(&[10]), 20);
        assert_eq!(next_weight(&[10, 40, 30]), 50);
        // Unsorted input is the normal case: notes are collected in file order.
        assert_eq!(next_weight(&[30, 10, 20]), 40);
    }

    #[test]
    fn values_are_quoted_for_the_parser_that_reads_them() {
        assert_eq!(quote_value("Plain title"), "'Plain title'");
        // An apostrophe would end a single-quoted value.
        assert_eq!(quote_value("Don't panic"), "\"Don't panic\"");
        // A double quote inside is fine either way, so the usual style wins.
        assert_eq!(quote_value("The \"best\" way"), "'The \"best\" way'");
        // Both: single quotes, which this parser strips correctly, and which is
        // the least bad option a YAML parser would also accept.
        assert_eq!(quote_value("it's a \"test\""), "'it's a \"test\"'");
    }

    #[test]
    fn a_new_note_is_a_valid_note() {
        let note = note_template("Don't panic", 30, "2026-10-02");

        // The frontmatter has to parse back to what went in.
        let (fields, body) = split_frontmatter(&note);
        assert_eq!(fields.get("title"), Some("Don't panic"));
        assert_eq!(fields.get("weight"), Some("30"));
        assert_eq!(fields.get("date"), Some("2026-10-02"));
        assert_eq!(fields.get("description"), Some(""));
        assert!(is_date(fields.get("date").unwrap_or_default()));
        // `date` present is what puts a note in the feed; `draft` absent is what
        // keeps it published.
        assert!(!is_truthy(fields.get("draft")));
        // And it is not an empty body: a note with a heading is a note.
        assert!(body.contains("## First heading"));
    }

    #[test]
    fn dates_convert_both_ways() {
        // The epoch, which is what `today_date` counts from.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(1970, 1, 1), 0);

        for date in [
            "2026-10-02",
            "2026-01-01",
            "2026-12-31",
            "2024-02-29", // a leap day
            "2000-03-01",
            "1999-12-31",
            "2100-06-15",
        ] {
            let (year, month, day) = parse_date(date).expect("test date parses");
            assert_eq!(
                civil_from_days(days_from_civil(year, month, day)),
                (year, month, day),
                "round trip failed for {date}"
            );
        }

        // And what the command writes today is a date the build accepts.
        assert!(is_date(&today_date()), "today_date: {}", today_date());
    }

    #[test]
    fn flags_are_read_as_booleans() {
        // Anything else — including a typo — leaves the note published, because
        // the failure mode of guessing "draft" is more forgiving than the
        // failure mode of guessing "published".
        assert!(is_truthy(Some("true")));
        assert!(is_truthy(Some("TRUE")));
        assert!(is_truthy(Some(" yes ")));
        assert!(is_truthy(Some("on")));
        assert!(is_truthy(Some("1")));

        assert!(!is_truthy(None));
        assert!(!is_truthy(Some("false")));
        assert!(!is_truthy(Some("no")));
        assert!(!is_truthy(Some("")));
        assert!(!is_truthy(Some("ture")));
    }

    #[test]
    fn dates_are_validated() {
        assert!(is_date("2026-09-30"));
        assert!(is_date(" 2026-01-05 "));
        assert!(!is_date("2026-9-30"));
        assert!(!is_date("yesterday"));
        assert!(!is_date("2026-09-30T10:00:00Z"));
    }

    #[test]
    fn civil_dates_convert_to_epoch_days() {
        // Known values: the epoch itself, the 2000 leap day, and a date after it.
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1), 10957);
        assert_eq!(days_from_civil(2000, 3, 1), 11017);
        assert_eq!(days_from_civil(1999, 12, 31), 10956);
    }

    #[test]
    fn leap_days_do_not_drift() {
        // A year apart across a leap day is 366 days, not 365.
        let before = days_from_civil(2024, 2, 28);
        let after = days_from_civil(2024, 3, 1);
        assert_eq!(after - before, 2);
    }

    #[test]
    fn ages_are_measured_from_a_known_today() {
        let today = days_from_civil(2026, 9, 30);

        assert_eq!(age_in_days("2026-09-30", today), Some(0));
        assert_eq!(age_in_days("2026-09-29", today), Some(1));
        assert_eq!(age_in_days("2026-07-11", today), Some(81));
        // A future date is not an error, just negative; the label clamps it.
        assert_eq!(age_in_days("2027-01-01", today), Some(-93));
        assert_eq!(age_in_days("not a date", today), None);
    }

    #[test]
    fn ages_read_as_english() {
        assert_eq!(humanize_age(0), "today");
        assert_eq!(humanize_age(-5), "today");
        assert_eq!(humanize_age(1), "yesterday");
        assert_eq!(humanize_age(9), "9 days ago");
        assert_eq!(humanize_age(45), "a month ago");
        assert_eq!(humanize_age(81), "3 months ago");
        assert_eq!(humanize_age(400), "a year ago");
        assert_eq!(humanize_age(1000), "3 years ago");
    }

    #[test]
    fn edit_distance_is_symmetric_and_bounded() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("cortex", "cortx"), 1);
        assert_eq!(levenshtein("cortex", "cotrex"), 2); // transposition = 2 edits
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("a", "b"), 1);
    }

    // ---- markdown source ------------------------------------------------

    #[test]
    fn multiline_tags_are_joined() {
        let md = "<img src=\"/a.png\"\n     alt=\"A\"\n     width=\"220\">";
        let joined = join_multiline_tags(md);

        assert_eq!(joined, "<img src=\"/a.png\" alt=\"A\" width=\"220\">");
        assert!(!joined.contains('\n'));
    }

    #[test]
    fn multiline_tags_are_left_alone_inside_fences() {
        let md = "```html\n<img src=\"/a.png\"\n  alt=\"A\">\n```\n";
        assert_eq!(join_multiline_tags(md), md);
    }

    #[test]
    fn angle_brackets_in_code_spans_are_left_alone() {
        let md = "Use `<img src>` in a template.\n";
        assert_eq!(join_multiline_tags(md), md);
    }

    #[test]
    fn a_quoted_angle_bracket_does_not_end_a_tag_early() {
        let joined = join_multiline_tags("<img alt=\"a > b\"\n src=\"/a.png\">");
        assert_eq!(joined, "<img alt=\"a > b\" src=\"/a.png\">");
    }

    // ---- html post-processing -------------------------------------------

    #[test]
    fn headings_get_ids_and_anchors() {
        let (html, toc) = add_heading_anchors("<h2>Hello World!</h2><p>x</p>");

        assert!(html.contains("<h2 id=\"hello-world\">"));
        assert!(html.contains("href=\"#hello-world\""));
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].id, "hello-world");
        assert_eq!(toc[0].title, "Hello World!");
        assert_eq!(toc[0].level, 2);
    }

    #[test]
    fn repeated_headings_get_unique_ids() {
        let (_, toc) = add_heading_anchors("<h2>Same</h2><h2>Same</h2><h2>Same</h2>");

        assert_eq!(
            toc.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec!["same", "same-1", "same-2"]
        );
    }

    #[test]
    fn heading_anchors_do_not_swallow_the_next_heading() {
        // A fixed-byte-count advance used to leave a stray `>` behind.
        let (html, toc) = add_heading_anchors("<h2>A</h2><h3>B</h3>");

        assert_eq!(toc.len(), 2);
        assert!(!html.contains("</h2>>"));
        assert!(html.contains("<h3 id=\"b\">"));
    }

    #[test]
    fn horizontal_rules_are_not_headings() {
        let (html, toc) = add_heading_anchors("<hr><p>x</p>");

        assert!(toc.is_empty());
        assert_eq!(html, "<hr><p>x</p>");
    }

    #[test]
    fn cjk_headings_keep_their_characters() {
        let (_, toc) = add_heading_anchors("<h2>环境搭建</h2>");
        assert_eq!(toc[0].id, "环境搭建");
    }

    #[test]
    fn tables_are_wrapped_for_keyboard_scrolling() {
        let wrapped = wrap_tables("<table><tr><td>x</td></tr></table>");
        assert!(wrapped.contains("class=\"table-wrap\""));
        assert!(wrapped.contains("tabindex=\"0\""));
        assert!(wrapped.ends_with("</div>"));
    }

    #[test]
    fn code_blocks_gain_their_language_attribute() {
        let wrapped = wrap_code_blocks("<pre><code class=\"language-rust\">fn x() {}</code></pre>");

        assert!(wrapped.starts_with("<pre data-lang=\"rust\">"));
        // The wrapper must not swallow what follows it.
        assert!(wrapped.ends_with("</code></pre>"));
    }

    #[test]
    fn strip_html_decodes_entities_and_collapses_space() {
        assert_eq!(strip_html("<p>a &amp; b</p>\n<p>c</p>"), "a & b c");
        assert_eq!(strip_html("&lt;img&gt;"), "<img>");
    }

    #[test]
    fn word_count_handles_latin_and_cjk() {
        assert_eq!(count_words("one two three"), 3);
        // Each CJK codepoint counts as a word, so a Chinese note still gets a
        // meaningful reading time.
        assert_eq!(count_words("环境搭建"), 4);
        assert_eq!(count_words("rust 环境 rust"), 4);
    }

    // ---- images ---------------------------------------------------------

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn png_dimensions_come_from_the_header() {
        assert_eq!(image_dimensions(&png_bytes(1351, 1164)), Some((1351, 1164)));
    }

    #[test]
    fn jpeg_dimensions_come_from_the_frame_header() {
        let mut bytes = vec![0xFF, 0xD8];
        // A segment before the frame header, to prove the walk skips it.
        bytes.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00]);
        bytes.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
        bytes.extend_from_slice(&964u16.to_be_bytes());
        bytes.extend_from_slice(&1275u16.to_be_bytes());

        assert_eq!(image_dimensions(&bytes), Some((1275, 964)));
    }

    #[test]
    fn unknown_image_types_have_no_dimensions() {
        assert_eq!(image_dimensions(b"<svg></svg>"), None);
    }

    #[test]
    fn images_gain_intrinsic_size_and_lazy_loading() {
        let html = "<p><img src=\"/a.png\" alt=\"A\"></p><p><img src=\"/b.png\" alt=\"B\"></p>";
        let out = rewrite_images(html, |src| {
            Some(ImageInfo {
                width: if src == "/a.png" { 100 } else { 200 },
                height: 50,
                webp: None,
            })
        });

        // The first image is the likely LCP candidate, so it stays eager.
        assert!(out.contains(
            "<img src=\"/a.png\" alt=\"A\" width=\"100\" height=\"50\" decoding=\"async\">"
        ));
        assert!(out.contains("loading=\"lazy\""));
        assert_eq!(out.matches("decoding=\"async\"").count(), 2);
        assert_eq!(out.matches("loading=\"lazy\"").count(), 1);
    }

    #[test]
    fn a_webp_sibling_becomes_a_picture_source() {
        let out = rewrite_images("<img src=\"/a.png\" alt=\"A\">", |_| {
            Some(ImageInfo {
                width: 10,
                height: 20,
                webp: Some("/a.webp".into()),
            })
        });

        assert!(out.starts_with("<picture><source type=\"image/webp\" srcset=\"/a.webp\">"));
        assert!(out.contains("src=\"/a.png\""));
        assert!(out.ends_with("</picture>"));
    }

    #[test]
    fn hand_written_dimensions_are_not_overwritten() {
        let out = rewrite_images("<img src=\"/logo.png\" alt=\"\" width=\"260\">", |_| {
            Some(ImageInfo {
                width: 999,
                height: 999,
                webp: None,
            })
        });

        assert!(out.contains("width=\"260\""));
        assert!(!out.contains("999"));
    }

    #[test]
    fn unknown_images_are_left_untouched() {
        let html = "<img src=\"https://example.com/x.png\" alt=\"X\">";
        let out = rewrite_images(html, |_| None);

        // Still gains decoding, but no invented dimensions and no picture.
        assert!(!out.contains("width="));
        assert!(!out.contains("<picture>"));
        assert!(out.contains("src=\"https://example.com/x.png\""));
    }

    #[test]
    fn rewrite_preserves_unquoted_attributes() {
        let out = rewrite_images("<img src=/a.png alt=A>", |_| None);
        assert!(out.contains("src=\"/a.png\""));
        assert!(out.contains("alt=\"A\""));
    }

    #[test]
    fn images_without_alt_are_reported() {
        let found = images_without_alt("<img src=\"/a.png\"><img src=\"/b.png\" alt=\"B\">");
        assert_eq!(found, vec!["/a.png"]);
    }

    // ---- json + hashing -------------------------------------------------

    #[test]
    fn json_strings_escape_correctly() {
        assert_eq!(json_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(json_string("line\nbreak\ttab"), "\"line\\nbreak\\ttab\"");
        assert_eq!(json_string("\u{1}"), "\"\\u0001\"");
        // Non-ASCII is valid inside a JSON string and must not be mangled.
        assert_eq!(json_string("环境"), "\"环境\"");
    }

    #[test]
    fn json_for_scripts_cannot_close_the_script_element() {
        let out = json_string_for_script("</script><script>alert(1)</script>");

        assert!(!out.contains('<'));
        assert!(!out.contains('>'));
        // The escapes still decode to the original text.
        assert!(out.contains("\\u003c/script\\u003e"));
    }

    #[test]
    fn xml_escaping_protects_the_feed() {
        assert_eq!(
            xml_escape("a & b <c> \"d\""),
            "a &amp; b &lt;c&gt; &quot;d&quot;"
        );
        // Ampersands must not be escaped twice over.
        assert_eq!(xml_escape("&amp;"), "&amp;amp;");
    }

    #[test]
    fn hashes_are_stable_and_content_sensitive() {
        assert_eq!(short_hash(b"abc"), short_hash(b"abc"));
        assert_ne!(short_hash(b"abc"), short_hash(b"abd"));
        assert_eq!(short_hash(b"abc").len(), 12);
    }

    // ---- css ------------------------------------------------------------

    #[test]
    fn minifier_removes_comments_and_spare_space() {
        let css = "/* a comment */\n.a {\n  color: red;\n}\n";
        assert_eq!(minify_css(css), ".a{color: red;}");
    }

    #[test]
    fn minifier_keeps_descendant_combinators() {
        assert_eq!(minify_css(".a .b { color: red }"), ".a .b{color: red}");
    }

    #[test]
    fn minifier_keeps_a_descendant_pseudo_class() {
        // `a :hover` matches a descendant; `a:hover` matches the link itself.
        // The original minifier dropped whitespace before `:`, which rewrote
        // `.prose :not(pre) > code` as a compound selector — so inline code
        // inside lists and blockquotes silently lost its styling. Both
        // selectors below are real, and live in `styles/note.css`.
        assert!(minify_css("a :hover { color: red }").starts_with("a :hover"));
        // Whitespace around `>` may be dropped; the space that is the
        // descendant combinator may not. This is the assertion that matters.
        assert!(
            minify_css(".prose :not(pre) > code { color: red }").starts_with(".prose :not(pre)")
        );
    }

    #[test]
    fn minifier_keeps_space_inside_values() {
        assert_eq!(
            minify_css(".a { width: calc(100% - 2rem) }"),
            ".a{width: calc(100% - 2rem)}"
        );
    }

    #[test]
    fn minifier_respects_quoted_braces_and_comments() {
        let css = ".a { content: \"} /* not a comment */\" }";
        let out = minify_css(css);
        assert!(out.contains("} /* not a comment */"));
    }

    #[test]
    fn bom_is_stripped_from_mid_stylesheet() {
        assert_eq!(strip_bom("\u{feff}.a{}"), ".a{}");
    }
}
