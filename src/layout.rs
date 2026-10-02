//! The document shell: `<head>`, header, navigation, footer, and the
//! not-found fallback.

use topcoat::{
    Result,
    context::Cx,
    router::request::uri,
    view::{BoxView, Unescaped, View, ViewExt, component, view},
};

use crate::icons;
use crate::icons::paths;
use crate::pipeline::json_string_for_script;
use crate::site::{self, GITHUB, SECTIONS, SITE_BASE_URL};

/// Which shelf (if any) the current URL belongs to, for the active nav state.
fn active_shelf(path: &str) -> String {
    let first = path.trim_matches('/').split('/').next().unwrap_or_default();
    if first.is_empty() {
        return "home".to_string();
    }
    if SECTIONS.iter().any(|s| s.key == first) {
        first.to_string()
    } else {
        "home".to_string()
    }
}

/// The canonical form of a content path.
///
/// The dispatcher trims slashes, so `/toolchain` and `/toolchain/` both return
/// 200. Without this the two would each declare a different canonical URL, and
/// a crawler would index the same page twice.
fn canonical_path(path: &str) -> String {
    if path == "/" || path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    }
}

/// An absolute URL for an application path.
///
/// Canonical links, Open Graph and structured data all require absolute URLs;
/// a root-relative `href="/toolchain/"` is not a valid canonical and is ignored.
fn absolute(path: &str) -> String {
    format!("{SITE_BASE_URL}{path}")
}

/// Everything the `<head>` needs beyond the page body.
pub struct PageMeta {
    pub title: String,
    pub description: String,
    /// Root-relative path of this page. `None` for a 404, which must not claim
    /// a canonical URL and must not be indexed.
    pub canonical: Option<String>,
    /// Open Graph type: `article` for a note, `website` for everything else.
    pub og_type: &'static str,
    /// `YYYY-MM-DD`, empty when the note does not say.
    pub published: String,
    pub updated: String,
    pub tags: &'static [&'static str],
}

impl PageMeta {
    pub fn website(title: impl Into<String>, description: impl Into<String>, path: &str) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            canonical: Some(canonical_path(path)),
            og_type: "website",
            published: String::new(),
            updated: String::new(),
            tags: &[],
        }
    }

    pub fn article(note: &'static site::Note) -> Self {
        Self {
            title: format!("{} · ref42", note.title),
            description: note.description.to_string(),
            canonical: Some(note.url.to_string()),
            og_type: "article",
            published: note.date.to_string(),
            updated: note.updated.to_string(),
            tags: note.tags,
        }
    }

    pub fn not_found() -> Self {
        Self {
            title: "404 · ref42".to_string(),
            description: "This note does not exist.".to_string(),
            canonical: None,
            og_type: "website",
            published: String::new(),
            updated: String::new(),
            tags: &[],
        }
    }
}

/// The document shell around a page's content.
///
/// The body is boxed and the result is too, so handlers can combine this with
/// an error branch without their opaque view types having to agree. The
/// lifetime is that of the values the views borrow (the request context and
/// the static site data), never `'static`.
pub async fn document<'a>(cx: &'a Cx, meta: PageMeta, body: BoxView<'a>) -> Result<BoxView<'a>> {
    let path = uri(cx).path().to_string();
    let active = active_shelf(&path);

    let doc = view! { cx =>
        <!DOCTYPE html>
            <html lang="en" data-theme="dark">
            head(meta: meta)

            <body>
                <div class="progress" aria-hidden="true">
                    <div class="progress__bar"></div>
                </div>

                <a class="skip-link" href="#main">"Skip to content"</a>

                header(active: active.clone())

                <main class="site-main" id="main">
                    (body)
                </main>

                footer()

                <button class="to-top" type="button" aria-label="Back to top" title="Back to top">
                    <svg
                        viewBox="0 0 24 24"
                        fill="none"
                        aria-hidden="true"
                        stroke-width="1.7"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        paths(packed: icons::ARROW_UP)
                    </svg>
                </button>

                <script src=(format!("/app.js?v={}", site::ASSET_VERSION)) defer=""></script>
            </body>
        </html>
    };

    Ok(doc.boxed())
}

/// The 404 document, used whenever a handler cannot resolve a page.
pub async fn not_found_document<'a>(cx: &'a Cx) -> Result<BoxView<'a>> {
    let body = crate::notfound::page(cx).await?.boxed();
    document(cx, PageMeta::not_found(), body).await
}

#[component]
async fn head(meta: PageMeta) -> Result<impl View> {
    let canonical = meta.canonical.as_deref().map(absolute);
    let og_image = absolute("/logo/ref42-og.png");
    let structured_data = article_json_ld(&meta);

    Ok(view! {
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <meta name="color-scheme" content="dark">
            <meta name="theme-color" content="#000000">
            <title>(meta.title.clone())</title>
            <meta name="description" content=(meta.description.clone())>
            <meta name="generator" content="Topcoat">

            if let Some(canonical) = canonical {
                <link rel="canonical" href=(canonical.clone())>
                <meta property="og:url" content=(canonical)>
            } else {
                // A 404 has no canonical URL, and asking a crawler to index it
                // would put a "route not found" page in the results.
                <meta name="robots" content="noindex">
            }

            <meta property="og:site_name" content=(site::SITE_TITLE)>
            <meta property="og:title" content=(meta.title.clone())>
            <meta property="og:description" content=(meta.description.clone())>
            <meta property="og:type" content=(meta.og_type)>
            <meta property="og:image" content=(og_image.clone())>
            <meta property="og:image:width" content="1200">
            <meta property="og:image:height" content="630">
            <meta name="twitter:card" content="summary_large_image">
            <meta name="twitter:image" content=(og_image)>

            if !meta.published.is_empty() {
                <meta property="article:published_time" content=(meta.published.clone())>
            }
            if !meta.updated.is_empty() {
                <meta property="article:modified_time" content=(meta.updated.clone())>
            }
            for tag in meta.tags {
                <meta property="article:tag" content=(*tag)>
            }

            if let Some(json) = structured_data {
                <script type="application/ld+json">(Unescaped::new_unchecked(json))</script>
            }

            <link id="site-favicon" rel="icon" type="image/png" href="/logo/ref42-favicon.png">
            <link rel="apple-touch-icon" href="/logo/apple-touch-icon.png">
            // A link in the footer is for people; this one is for a feed reader,
            // which otherwise cannot discover that the site has a feed at all.
            <link
                rel="alternate"
                type="application/atom+xml"
                title=(site::SITE_TITLE)
                href="/feed.xml"
            >

            // Preloaded because the font is the largest asset on the site and
            // `font-display: swap` makes the text paint in a fallback and then
            // reflow when it arrives. `crossorigin` is required even though the
            // font is same-origin: a font is always fetched in CORS mode, and
            // without it the preload is discarded and the file fetched twice.
            <link
                rel="preload"
                as="font"
                type="font/woff2"
                href=(format!("/fonts/iosevka-latin-400-normal.woff2?v={}", site::FONT_VERSION))
                crossorigin="anonymous"
            >
            <link rel="stylesheet" href=(format!("/main.css?v={}", site::ASSET_VERSION))>
        </head>
    })
}

/// `schema.org` structured data for a note, or nothing for any other page.
///
/// Search engines use this for article metadata; every field already exists on
/// the note, so it costs nothing but a few hundred bytes.
fn article_json_ld(meta: &PageMeta) -> Option<String> {
    if meta.og_type != "article" {
        return None;
    }
    let canonical = meta.canonical.as_deref()?;

    let mut out = String::from("{\"@context\":\"https://schema.org\",\"@type\":\"TechArticle\"");
    out.push_str(",\"headline\":");
    out.push_str(&json_string_for_script(&meta.title));
    out.push_str(",\"description\":");
    out.push_str(&json_string_for_script(&meta.description));
    out.push_str(",\"url\":");
    out.push_str(&json_string_for_script(&absolute(canonical)));
    out.push_str(",\"mainEntityOfPage\":");
    out.push_str(&json_string_for_script(&absolute(canonical)));
    if !meta.published.is_empty() {
        out.push_str(",\"datePublished\":");
        out.push_str(&json_string_for_script(&meta.published));
    }
    if !meta.updated.is_empty() {
        out.push_str(",\"dateModified\":");
        out.push_str(&json_string_for_script(&meta.updated));
    }
    if !meta.tags.is_empty() {
        out.push_str(",\"keywords\":");
        out.push_str(&json_string_for_script(&meta.tags.join(", ")));
    }
    out.push_str(",\"author\":{\"@type\":\"Person\",\"name\":\"ref42\"}");
    out.push_str(",\"publisher\":{\"@type\":\"Organization\",\"name\":\"ref42\"}");
    out.push('}');

    Some(out)
}

#[component]
async fn header(active: String) -> Result<impl View> {
    Ok(view! {
        <header class="site-header">
            <div class="shell site-header__inner">
                <a class="brand" href="/" aria-label="ref42 home">
                    <img class="brand__mark" src="/logo/ref42.png" alt="" width="44" height="44">
                    <span class="brand__text">
                        <span class="brand__name">"ref" <i>"42"</i></span>
                        <span class="brand__tag">"my daily notes"</span>
                    </span>
                </a>

                <nav class="site-nav" id="site-nav" aria-label="Primary navigation">
                    <a
                        class="nav-link"
                        href="/"
                        aria-current=(if active == "home" { "page" } else { "" })
                    >"Home"</a>
                    for section in SECTIONS {
                        <a
                            class="nav-link"
                            href=(format!("/{}/", section.key))
                            aria-current=(if active == section.key { "page" } else { "" })
                        >(section.short)</a>
                    }

                    <div class="nav-tools">
                        <a
                            class="icon-link"
                            href=(GITHUB)
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label="GitHub"
                            title="GitHub"
                        >
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                aria-hidden="true"
                                stroke-width="1.7"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                paths(packed: icons::GITHUB)
                            </svg>
                        </a>
                        <a
                            class="icon-link"
                            href=(site::BILIBILI)
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label="Bilibili"
                            title="Bilibili"
                        >
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                aria-hidden="true"
                                stroke-width="1.7"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                paths(packed: icons::BILIBILI)
                            </svg>
                        </a>

                        <form class="site-search" role="search" autocomplete="off">
                            <label class="visually-hidden" for="site-search-input">"Search notes"</label>
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                aria-hidden="true"
                                stroke-width="1.7"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                paths(packed: icons::SEARCH)
                            </svg>
                            <input
                                id="site-search-input"
                                type="search"
                                placeholder="Search notes..."
                                aria-expanded="false"
                            >
                            <kbd>"/"</kbd>
                            <div id="site-search-results" class="search-results" role="listbox" hidden=""></div>
                        </form>
                    </div>
                </nav>

                <button
                    class="nav-burger"
                    type="button"
                    aria-label="Open menu"
                    aria-expanded="false"
                    aria-controls="site-nav"
                >
                    <svg
                        viewBox="0 0 24 24"
                        fill="none"
                        aria-hidden="true"
                        stroke-width="1.7"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        paths(packed: icons::MENU)
                    </svg>
                </button>
            </div>
        </header>
    })
}

#[component]
async fn footer() -> Result<impl View> {
    Ok(view! {
        <footer class="site-footer">
            <div class="shell">
                <div class="site-footer__base">
                    <span>"Copyright " (site::SITE_TITLE) " · my daily notes"</span>
                </div>
            </div>
        </footer>
    })
}
