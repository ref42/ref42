# Writing a note

Everything the site publishes comes from `content/`. There is nothing to
register and no build step to remember: **a markdown file becomes a page.**

Put a file at `content/<anything>/<name>.md` and it is published at
`/<anything>/<name>/`. A folder becomes a section on its own; the filename
becomes the URL.

## Start a note with the command

```powershell
cargo new-note "Flash an ESP32-C6 over USB" -s esp32
```

That writes `content/esp32/flash_an_esp32_c6_over_usb.md` — the title becomes the
filename and therefore the URL — with today's date and a `weight` that puts it
after the last note in that section. It prints the path it wrote, the URL it will
be published at, and the one field worth filling in before you commit.

| | |
|---|---|
| `-s, --section <key>` | which folder to write into (required) |
| `-w, --weight <n>` | position in the section, instead of last |
| `-d, --date <date>` | `YYYY-MM-DD`, instead of today |
| `--slug <name>` | the filename, if the title makes a poor one |
| `-n, --dry-run` | print the note instead of writing it |
| `-f, --force` | overwrite an existing note, or create a strange new section |

Two things it refuses to do quietly. A section name that is one or two edits from
an existing one — `--section toochain` — stops and asks whether you meant
`toolchain`, because the alternative is publishing a stray shelf. And a title it
cannot turn into a filename (`"!!!"`) asks for `--slug` rather than writing a file
called `.md`.

A section that does not exist yet is created, because a folder of notes is a
section (see [Sections](#sections)).

The alias is in `.cargo/config.toml`; without it the same command is
`cargo run --quiet -- new "…" -s esp32`, which is also what the deployed binary
offers as `ref42 new`.

## A note by hand

```markdown
---
title: 'Flash an ESP32-C6 over USB'
description: 'One line for the catalogue, search results and social cards.'
weight: 20
date: "2026-09-30"
tags: [esp32, espflash]
---

Intro paragraph.

## First heading

Text, `inline code`, [a link](/toolchain/cortex-m/).

```bash
espflash flash --monitor
```

![What the screenshot shows](/esp32/c6-flash.png)
```

See it:

```powershell
cargo run          # http://127.0.0.1:3000/
```

Publish it:

```powershell
git add -A
git commit -m "Add ESP32-C6 flashing note"
git push
```

Pushing to `master` deploys. The note goes live about two minutes later, with
its canonical URL, sitemap entry, feed entry, search entry, related notes and
previous/next links worked out for you.

## Frontmatter

All of it is optional — a file with no frontmatter at all still publishes, titled
from its filename. What each field does when it is missing is the part worth
knowing:

| field | default if missing | what it does |
|---|---|---|
| `title` | the filename | the heading, `<title>`, catalogue row, search result |
| `description` | empty | catalogue blurb, search result, social card. One good sentence |
| `weight` | sorts **last** | order within the section. Use 10, 20, 30… |
| `date` | none | `"YYYY-MM-DD"`. **Without it the note never appears in the feed** |
| `updated` | the `date` | drives the "Verified 3 months ago" chip |
| `tags` | none | drives "Related notes" (notes sharing a tag) |
| `tested_with` | none | e.g. `["esp-hal 0.16", "espflash 4.1"]`, shown in the note's rail |
| `slug` | the filename | only needed for a URL that differs from the file |
| `draft` | off | `draft: true` keeps the note out of the site entirely |

Two of those bite quietly. **`weight`**: forget it and the note sorts last in its
section rather than erroring. **`date`**: forget it and the note is on the site
and in the sitemap but absent from `feed.xml`.

`draft: true` means the file stays in `content/` and is published nowhere — no
page, no feed, no sitemap, no search entry. It is the way to keep a half-written
note in place. `draft: no` (or a typo like `draft: ture`) publishes it, because
guessing "published" is the recoverable mistake.

## Markdown

GitHub-flavoured: tables, task lists, strikethrough, autolinks and footnotes are
all on. Code fences are highlighted with syntect's `base16-mocha.dark` — chosen
because the adapter only indexes syntect's bundled themes and it was the closest
match to the Catppuccin Mocha the old Zola config asked for.

Every heading gets an anchor and appears in the "On this page" rail.

## Images

Put the file anywhere under `static/` and reference it from the site root:

```
static/esp32/c6-flash.png   →   ![alt text](/esp32/c6-flash.png)
```

The build reads the real pixel dimensions and adds `width`, `height` and
`loading="lazy"` for you, so the layout does not jump while the image loads.

To get WebP — about half the bytes for a screenshot — run:

```powershell
python scripts/optimize-images.py     # needs Pillow
```

It converts the images your notes actually reference, keeps the PNG as the
fallback, and refuses a conversion that would come out *bigger*. Commit the
`.webp` files it writes; the build offers them automatically. It also reports any
reference pointing at a file that is not there — worth having, because the build
does **not** warn about a missing image: it renders a broken `<img>` and says
nothing.

A missing `alt` is a different matter: **that fails the content check in CI.**

## Sections

`content/` currently has four — `toolchain`, `esp32`, `stm32`, `utils` — and each
one holds an `_index.md`:

```markdown
---
title: "Robotics"
short: "Robotics"
order: 4
description: "One line for the section's shelf page."
---

The body of this file becomes the section's intro paragraph.
```

A **new folder needs none of that to work.** Drop notes in `content/robotics/`
and the folder becomes a section titled from its own name (`robotics` →
"Robotics", `dev-tools` → "Dev Tools"), sorting after the declared sections.
Add the `_index.md` when you want to control the title, the order, the
description, or to write an intro.

## What runs before it goes live

Every push is checked by `.github/workflows/rust.yml`: `cargo fmt`, `clippy`
with warnings denied, the unit tests, and a content lint that fails on any
build warning — an `alt` that is missing, or a `date` that is not
`YYYY-MM-DD`. The deploy is a separate workflow, so a failed check reports a
problem without taking the site down.

## Where things are

| path | what it is |
|---|---|
| `content/**/*.md` | the notes. The only thing you normally touch |
| `static/` | images, fonts, favicons — copied to the site root as-is |
| `styles/` | the CSS sources, compiled and minified by the build |
| `scripts/app.js` | search, keyboard shortcuts, the 404's "did you mean" |
| `src/new_note.rs` | `ref42 new`, the command that writes a note |
| `src/` | the Rust that renders the pages |
| `build.rs` | turns `content/` into notes: frontmatter, ordering, feed, sitemap, search index |
