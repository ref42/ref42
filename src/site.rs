//! Site data: the shelves and notes generated from `content/` at build time,
//! plus the handful of hand-written constants that are not content.

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

// `SITE_TITLE` and `SITE_DESCRIPTION` are generated from `content/_index.md`.
// They used to be constants here as well, and the two copies had already drifted
// into two different descriptions of the same site.

pub const GITHUB: &str = "https://github.com/ref42";
pub const BILIBILI: &str = "https://space.bilibili.com/3493142393260061";

pub fn section(key: &str) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.key == key)
}

pub fn section_title(key: &str) -> &'static str {
    section(key).map(|s| s.title).unwrap_or("Note")
}

pub fn section_short(key: &str) -> &'static str {
    section(key).map(|s| s.short).unwrap_or("Note")
}

pub fn notes_in(key: &str) -> impl Iterator<Item = &'static Note> {
    NOTES.iter().filter(move |n| n.section == key)
}

pub fn note_by_url(url: &str) -> Option<&'static Note> {
    NOTES.iter().find(|n| n.url == url)
}

/// Notes in reading order, with their position on the shelf.
pub fn note_position(url: &str) -> Option<(usize, usize)> {
    let note = note_by_url(url)?;
    let index = notes_in(note.section).position(|n| n.url == url)?;
    let total = notes_in(note.section).count();
    Some((index + 1, total))
}

/// Neighbours in the same shelf, for the previous/next footer.
pub fn siblings(url: &str) -> (Option<&'static Note>, Option<&'static Note>) {
    let Some(note) = note_by_url(url) else {
        return (None, None);
    };
    let shelf: Vec<&'static Note> = notes_in(note.section).collect();
    let Some(pos) = shelf.iter().position(|n| n.url == url) else {
        return (None, None);
    };

    let prev = if pos > 0 { Some(shelf[pos - 1]) } else { None };
    let next = shelf.get(pos + 1).copied();
    (prev, next)
}

/// Notes that share at least one tag with `note`, most-shared first.
///
/// Ties keep the shelf reading order, because `NOTES` is already sorted and
/// `sort_by_key` is stable. Notes with no tags have no related notes, which is
/// honest: inventing "related" from word overlap would surface nonsense.
pub fn related_notes(note: &'static Note) -> Vec<&'static Note> {
    if note.tags.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(usize, &'static Note)> = NOTES
        .iter()
        .filter(|other| other.url != note.url)
        .filter_map(|other| {
            let shared = other
                .tags
                .iter()
                .filter(|tag| note.tags.contains(tag))
                .count();
            (shared > 0).then_some((shared, other))
        })
        .collect();
    scored.sort_by_key(|(shared, _)| std::cmp::Reverse(*shared));

    scored.into_iter().map(|(_, other)| other).collect()
}

/// The note a mistyped URL most likely meant.
///
/// Deliberately conservative — a wrong "did you mean" is worse than none — so a
/// candidate has to clear a real bar: an exact slug match anywhere on the site
/// (by far the most common mistake is a note reached without its section
/// prefix), a prefix match either way, or an edit distance of at most two.
pub fn closest_note(trimmed: &str) -> Option<&'static Note> {
    let last = trimmed
        .trim_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let needle = normalize(last);
    // Below three characters almost everything is within two edits of
    // everything else, and a "did you mean" that offers a random note is worse
    // than offering nothing.
    if needle.chars().count() < 3 {
        return None;
    }

    let mut best: Option<(usize, &'static Note)> = None;
    for note in NOTES {
        // The slug is the last segment of the note's URL; the runtime `Note` does
        // not carry it separately, and deriving it here keeps the generated data
        // lean.
        let slug = normalize(
            note.url
                .trim_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or_default(),
        );
        let score = if slug == needle {
            100
        } else if slug.starts_with(&needle) || needle.starts_with(&slug) {
            70
        } else {
            match crate::pipeline::levenshtein(&slug, &needle) {
                1 => 45,
                2 => 40,
                _ => 0,
            }
        };

        if score > 0 && best.is_none_or(|(best_score, _)| score > best_score) {
            best = Some((score, note));
        }
    }

    best.map(|(_, note)| note)
}

/// Lowercase, alphanumerics only, so `rust_dev_base_env`, `rust-dev-base-env`
/// and `Rust Dev Base Env` all compare equal.
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a trimmed request path names a page this site renders rather than a
/// file on disk. Used by the static file layer to step aside.
pub fn is_content_path(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return true;
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    match segments.as_slice() {
        [section_key] => section(section_key).is_some(),
        _ => note_by_url(&format!("/{trimmed}/")).is_some(),
    }
}
