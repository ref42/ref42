//! The single page handler: it resolves the request path to the home page, a
//! shelf, or a note.
//!
//! Topcoat's router cannot register `/{section_key}/` and `/{*doc_path}`
//! side by side, and a single catch-all is exactly what keeps the URL layout
//! unchanged from the original site.

use topcoat::{
    Result,
    context::Cx,
    router::{StatusCode, page, request::uri},
    view::{BoxView, Unescaped, View, ViewExt, component, view},
};

use crate::icons;
use crate::icons::paths;
use crate::layout;
use crate::site::{self, NOTES, SECTIONS};

/// Every URL on the site, from `/` down to `/toolchain/rust_dev_base_env/`.
///
/// The request path is read from the request context rather than captured as
/// a path parameter, because the static file layer owns Topcoat's single
/// catch-all path parameter.
pub async fn dispatch(cx: &Cx) -> Result<BoxView<'_>> {
    let path = uri(cx).path().to_string();
    let trimmed = path.trim_matches('/');
    let segments: Vec<&str> = if trimmed.is_empty() {
        Vec::new()
    } else {
        trimmed.split('/').collect()
    };

    match segments.as_slice() {
        [] => home(cx).await,
        [section] => match site::section(section) {
            Some(section) => shelf(cx, section).await,
            None => not_found(cx).await,
        },
        _ => {
            let url = format!("/{}/", segments.join("/"));
            match site::note_by_url(&url) {
                Some(note) => crate::note::render(cx, note).await,
                None => not_found(cx).await,
            }
        }
    }
}

#[page("/")]
pub async fn dispatch_root(cx: &Cx) -> Result<BoxView<'_>> {
    dispatch(cx).await
}

#[page("/{*doc_path}")]
pub async fn dispatch_nested(cx: &Cx) -> Result<BoxView<'_>> {
    dispatch(cx).await
}

/// A branded 404 with a matching status code.
async fn not_found<'a>(cx: &'a Cx) -> Result<BoxView<'a>> {
    let doc = layout::not_found_document(cx).await?;
    Ok(view! { cx => (StatusCode::NOT_FOUND) (doc) }.boxed())
}

// ---------------------------------------------------------------------------
// home
// ---------------------------------------------------------------------------

async fn home(cx: &Cx) -> Result<BoxView<'_>> {
    // No hero: the wordmark is already in the header and the tagline is in the
    // footer, so a masthead here only repeats them. The page is the notes.
    let body = view! { cx =>
        feed_block()
    };

    layout::document(
        cx,
        layout::PageMeta::website("ref42 · Development notes", site::SITE_DESCRIPTION, "/"),
        body.boxed(),
    )
    .await
}

/// The whole index: every note in reading order, numbered like a catalogue.
#[component]
async fn feed_block() -> Result<impl View> {
    Ok(view! {
        <section class="block feed-block" aria-label="All notes">
            <div class="shell">
                <div class="feed">
                    for (index, item) in NOTES.iter().enumerate() {
                        <a class="note-row" href=(item.url)>
                            <span class="note-row__node" aria-hidden="true">
                                if index + 1 < 10 { "0" } else { "" }
                                (index + 1)
                            </span>
                            <span class="note-row__body">
                                <strong class="note-row__title">(item.title)</strong>
                                <span class="note-row__desc">(item.description)</span>
                            </span>
                            <span class="note-row__side">
                                (site::section_short(item.section))
                                " · "
                                (item.minutes)
                                " min"
                            </span>
                        </a>
                    }
                </div>
            </div>
        </section>
    })
}

// ---------------------------------------------------------------------------
// shelf
// ---------------------------------------------------------------------------
async fn shelf<'a>(cx: &'a Cx, section: &'static site::Section) -> Result<BoxView<'a>> {
    let notes: Vec<&'static site::Note> = site::notes_in(section.key).collect();

    let body = view! { cx =>
        <div class="shell">
            <header class="shelf-head">
                <p class="eyebrow">
                    "/"
                    (section.key)
                </p>

                <h1>(section.title)</h1>
                <p class="shelf-head__lead">(section.description)</p>
            </header>

            // The shelf intro is the body of the section's `_index.md`. It used
            // to be parsed and thrown away, so authored copy sat in the content
            // directory while the page showed only the frontmatter description.
            if !section.intro.is_empty() {
                <div class="prose shelf-intro">(Unescaped::new_unchecked(section.intro))</div>
            }

            <ol class="route-list">
                for (index, item) in notes.iter().enumerate() {
                    <li>
                        <a class="route-item" href=(item.url)>
                            <span class="route-item__node">
                                if index + 1 < 10 { "0" } else { "" }
                                (index + 1)
                            </span>
                            <span>
                                <strong class="route-item__title">(item.title)</strong>
                                <span class="route-item__desc">(item.description)</span>
                            </span>
                            <span class="route-item__side">
                                (item.minutes)
                                " min"
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    aria-hidden="true"
                                    stroke-width="1.7"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    paths(packed: icons::ARROW_RIGHT)
                                </svg>
                            </span>
                        </a>
                    </li>
                }
            </ol>

            shelf_neighbours(active: section.key)
        </div>
    };

    layout::document(
        cx,
        layout::PageMeta::website(
            format!("{} · ref42", section.title),
            section.description,
            &format!("/{}/", section.key),
        ),
        body.boxed(),
    )
    .await
}

#[component]
async fn shelf_neighbours(active: &str) -> Result<impl View> {
    let index = SECTIONS.iter().position(|s| s.key == active);
    let prev = index
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| SECTIONS.get(i));
    let next = index.and_then(|i| SECTIONS.get(i + 1));

    Ok(view! {
        <nav class="shelf-next" aria-label="Section navigation">
            if let Some(prev) = prev {
                <a href=(format!("/{}/", prev.key))>
                    <span class="dir">"← Previous section"</span>
                    <span class="name">(prev.title)</span>
                </a>
            }
            if let Some(next) = next {
                <a href=(format!("/{}/", next.key)) style="margin-inline-start:auto;text-align:end">
                    <span class="dir">"Next section "</span>
                    <span class="name">(next.title)</span>
                </a>
            }
        </nav>
    })
}
