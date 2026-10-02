//! `ref42 new` — the one thing this binary does that is not serving the site.
//!
//! Authoring runs through here rather than through a shell script so that the
//! rules live where the build's rules live: the same `pipeline` module that
//! parses frontmatter writes it, and the date, the filename and the ordering are
//! the ones `build.rs` will read back. A generator in another language would be
//! a second opinion about what a note is, and the two would drift.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::pipeline::{
    filename_from_title, is_date, levenshtein, next_weight, note_template, split_frontmatter,
    today_date,
};

/// Where the notes live, relative to the repository root.
const CONTENT: &str = "content";

/// Flags that need a value, so `--flag=value` and `--flag value` both work.
/// A slice rather than an array so adding a flag cannot leave a stale count.
const VALUE_FLAGS: &[&str] = &[
    "--section",
    "-s",
    "--weight",
    "-w",
    "--date",
    "-d",
    "--slug",
];

const USAGE: &str = "\
ref42 new — start a note

    ref42 new <title> --section <key> [options]

Options
    -s, --section <key>   folder under content/ to write into (required)
    -w, --weight <n>      position in that section (default: after the last note)
    -d, --date <date>     YYYY-MM-DD (default: today)
        --slug <name>     filename, without .md (default: from the title)
    -n, --dry-run         print the note instead of writing it
    -f, --force           overwrite an existing file, or create a new section
    -h, --help            this

The title becomes the note's `title`, its filename and therefore its URL.
Everything else the build works out: the section it appears in, the feed, the
sitemap and the search index. `description:` is the one field worth filling in
before you commit — the catalogue, search results and social cards show it.";

#[derive(Debug, Default)]
pub struct Options {
    title: Option<String>,
    section: Option<String>,
    weight: Option<i64>,
    date: Option<String>,
    slug: Option<String>,
    dry_run: bool,
    force: bool,
    help: bool,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self::default();
        let mut index = 0;

        while index < args.len() {
            let arg = args[index].as_str();
            index += 1;

            // `--weight=30` in one word; the split keeps the value with its flag.
            let (flag, inline) = match arg.split_once('=') {
                Some((flag, value)) if VALUE_FLAGS.contains(&flag) => {
                    (flag, Some(value.to_string()))
                }
                _ => (arg, None),
            };

            match flag {
                "--section" | "-s" => {
                    options.section = Some(value(args, &mut index, inline, flag)?)
                }
                "--weight" | "-w" => {
                    let raw = value(args, &mut index, inline, flag)?;
                    options.weight = Some(
                        raw.parse()
                            .map_err(|_| format!("{flag} wants a number, got `{raw}`"))?,
                    );
                }
                "--date" | "-d" => options.date = Some(value(args, &mut index, inline, flag)?),
                "--slug" => options.slug = Some(value(args, &mut index, inline, flag)?),
                "--dry-run" | "-n" => options.dry_run = true,
                "--force" | "-f" => options.force = true,
                "--help" | "-h" => options.help = true,
                other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
                _ => {
                    if options.title.is_some() {
                        return Err(format!("`{arg}` is a second title; quote the first one"));
                    }
                    options.title = Some(arg.to_string());
                }
            }
        }

        Ok(options)
    }
}

/// The value for a flag, from `--flag=value` or the next argument.
fn value(
    args: &[String],
    index: &mut usize,
    inline: Option<String>,
    flag: &str,
) -> Result<String, String> {
    if let Some(inline) = inline {
        return Ok(inline);
    }
    let next = args.get(*index).cloned();
    match next {
        Some(next) if !next.starts_with('-') => {
            *index += 1;
            Ok(next)
        }
        _ => Err(format!("{flag} needs a value")),
    }
}

pub fn run(args: &[String]) -> ExitCode {
    let options = match Options::parse(args) {
        Ok(options) => options,
        Err(message) => return usage_error(&message),
    };

    if options.help {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    match create(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ref42 new: {message}");
            ExitCode::FAILURE
        }
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("ref42 new: {message}\n");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn create(options: &Options) -> Result<(), String> {
    let root = Path::new(CONTENT);
    if !root.is_dir() {
        return Err(format!(
            "there is no `{CONTENT}/` directory here — run this from the repository root"
        ));
    }

    let sections = sections(root);

    let title = match &options.title {
        Some(title) => title.clone(),
        None => return Err("a title is required".to_string()),
    };

    let section = match &options.section {
        Some(section) => section.clone(),
        None => {
            return Err(format!(
                "a section is required; {CONTENT}/ has {}",
                sections.join(", ")
            ));
        }
    };

    let slug = options
        .slug
        .clone()
        .unwrap_or_else(|| filename_from_title(&title));
    if slug.is_empty() {
        return Err(format!(
            "`{title}` has no filename in it that survives a URL — pass --slug"
        ));
    }

    // A section that does not exist is created, because a folder of notes is a
    // section now. A *typo* of an existing one is not, because that publishes a
    // stray shelf rather than failing: `--force` is the way to mean it.
    let dir = root.join(&section);
    let is_new_section = !dir.is_dir();
    if is_new_section
        && !options.force
        && let Some(close) = closest(&section, &sections)
    {
        return Err(format!(
            "content/{section}/ does not exist — did you mean `{close}`? \
             (--force to create a new section anyway)"
        ));
    }

    let date = match &options.date {
        Some(date) if is_date(date) => date.clone(),
        Some(date) => return Err(format!("--date wants YYYY-MM-DD, got `{date}`")),
        None => today_date(),
    };

    let inherited = options.weight.is_none();
    let weight = match options.weight {
        Some(weight) => weight,
        None => next_weight(&existing_weights(&dir)),
    };

    let path = dir.join(format!("{slug}.md"));
    let note = note_template(&title, weight, &date);

    if options.dry_run {
        print!("dry run: {} would be written\n\n{note}", path.display());
        return Ok(());
    }

    if path.exists() && !options.force {
        return Err(format!(
            "{} already exists (--force to overwrite it)",
            path.display()
        ));
    }

    fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;
    fs::write(&path, note).map_err(|err| format!("{}: {err}", path.display()))?;

    println!("{}", path.display());
    println!(
        "  weight   {weight}{}",
        if inherited {
            " (after the last note in this section)"
        } else {
            ""
        }
    );
    println!("  date     {date}");
    println!("  URL      {}", note_url(&section, &slug));
    if is_new_section {
        println!(
            "  section  content/{section}/ is new; it will be titled \"{}\". \
             Add _index.md to set the title, the order or an intro",
            crate::pipeline::title_from_key(&section)
        );
    }
    println!();
    println!("Fill in `description:` — the catalogue, search and social cards show it.");
    println!("Then `cargo run` to look at it, and commit.");

    Ok(())
}

/// The URL the note will be published at.
///
/// The trailing slash is not decoration: the export writes
/// `<slug>/index.html`, so the slash is the canonical form and the thing the
/// canonical tag, the sitemap and every internal link use.
fn note_url(section: &str, slug: &str) -> String {
    format!(
        "{}/{section}/{slug}/",
        crate::site::SITE_BASE_URL.trim_end_matches('/')
    )
}

/// Section folders under `content/`, sorted.
fn sections(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    found.sort();
    found
}

/// The closest existing name to `needle`, if one is close enough to be a typo.
///
/// Two edits, the same bar the 404 uses for a mistyped note. Three characters is
/// the floor for the same reason it is there: below that everything is within
/// two edits of everything else.
fn closest<'a>(needle: &str, known: &'a [String]) -> Option<&'a str> {
    if needle.len() < 3 {
        return None;
    }

    known
        .iter()
        .map(|key| {
            (
                levenshtein(&needle.to_lowercase(), &key.to_lowercase()),
                key,
            )
        })
        .filter(|(distance, _)| *distance <= 2)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, key)| key.as_str())
}

/// The `weight` of every note already in a section.
fn existing_weights(dir: &Path) -> Vec<i64> {
    let mut weights = Vec::new();

    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        if path.file_name().is_some_and(|name| name == "_index.md") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            let (fields, _) = split_frontmatter(&raw);
            if let Some(weight) = fields.number("weight") {
                weights.push(weight);
            }
        }
    }

    weights
}

/// Kept for the tests below and for a future `ref42 new --list`.
#[allow(dead_code)]
fn section_paths(root: &Path) -> Vec<PathBuf> {
    sections(root).iter().map(|key| root.join(key)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn a_title_and_a_section_are_read_from_position_or_flag() {
        let options = Options::parse(&args(&["Hello", "--section", "esp32"])).expect("parses");
        assert_eq!(options.title.as_deref(), Some("Hello"));
        assert_eq!(options.section.as_deref(), Some("esp32"));

        let options = Options::parse(&args(&["Hello", "-s", "esp32"])).expect("parses");
        assert_eq!(options.section.as_deref(), Some("esp32"));

        // The other spelling, which is the one that is easy to forget.
        let options = Options::parse(&args(&["Hello", "--section=esp32"])).expect("parses");
        assert_eq!(options.section.as_deref(), Some("esp32"));
    }

    #[test]
    fn a_title_with_spaces_is_one_argument() {
        let options = Options::parse(&args(&["Flash an ESP32-C6", "-s", "esp32"])).expect("parses");
        assert_eq!(options.title.as_deref(), Some("Flash an ESP32-C6"));

        // Unquoted, it looks like three arguments, and saying so beats writing a
        // note called "an".
        let error = Options::parse(&args(&["Flash", "an", "ESP32-C6"])).expect_err("rejects");
        assert!(error.contains("second title"), "{error}");
    }

    #[test]
    fn numbers_and_flags_are_validated() {
        let options =
            Options::parse(&args(&["T", "-s", "esp32", "-w", "30", "--dry-run"])).expect("parses");
        assert_eq!(options.weight, Some(30));
        assert!(options.dry_run);

        let error = Options::parse(&args(&["T", "-s", "esp32", "-w", "heavy"])).expect_err("bad");
        assert!(error.contains("wants a number"), "{error}");

        let error = Options::parse(&args(&["T", "-s"])).expect_err("missing value");
        assert!(error.contains("needs a value"), "{error}");

        let error = Options::parse(&args(&["T", "--wat"])).expect_err("unknown");
        assert!(error.contains("unknown option"), "{error}");
    }

    #[test]
    fn the_reported_url_is_the_canonical_one() {
        let url = note_url("esp32", "flash_an_esp32_c6");
        assert!(url.ends_with("/esp32/flash_an_esp32_c6/"), "{url}");
        // The site's own base URL, which the build already resolved, and no
        // doubled slash when it carries a trailing one.
        assert!(url.starts_with(crate::site::SITE_BASE_URL.trim_end_matches('/')));
        assert!(!url.contains("//esp32"), "{url}");
    }

    #[test]
    fn typos_in_a_section_name_are_caught() {
        let known = args(&["esp32", "stm32", "toolchain", "utils"]);
        assert_eq!(closest("toolchian", &known), Some("toolchain"));
        // An exact name comes back as itself, and the comparison is
        // case-insensitive: that is what catches `--section ESP32` on a
        // filesystem where the spelling has to match the directory.
        assert_eq!(closest("esp32", &known), Some("esp32"));
        assert_eq!(closest("ESP32", &known), Some("esp32"));
        assert_eq!(closest("esp8266", &known), None, "too far to guess");
        assert_eq!(closest("u", &known), None, "too short to guess");
    }
}
