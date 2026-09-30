//! The note (article) page: breadcrumbs, prose, contents rail, discussion,
//! and previous/next navigation.

use topcoat::{
    Result,
    context::Cx,
    view::{BoxView, Unescaped, View, ViewExt, component, view},
};

use crate::layout;
use crate::pipeline::humanize_age;
use crate::site::{self, Note};

pub async fn render<'a>(cx: &'a Cx, note: &'static Note) -> Result<BoxView<'a>> {
    let (prev, next) = site::siblings(note.url);
    let position = site::note_position(note.url);
    let related = site::related_notes(note);

    let body = view! { cx =>
        <div class="shell">
            <div class="doc">
                <article class="doc-article">
                    <nav class="crumbs" aria-label="Breadcrumb">
                        <a href="/">"ref42"</a>
                        <span class="sep" aria-hidden="true">"/"</span>
                        <a href=(format!("/{}/", note.section))>
                            (site::section_title(note.section))
                        </a>
                        <span class="sep" aria-hidden="true">"/"</span>
                        <span class="here">(note.title)</span>
                    </nav>

                    <header class="doc-head" data-reveal="">
                        <p class="eyebrow">(site::section_short(note.section))</p>
                        <h1>(note.title)</h1>
                        if !note.description.is_empty() {
                            <p class="lead">(note.description)</p>
                        }

                        <div class="doc-head__meta">
                            if let Some((index, total)) = position {
                                <span class="chip">
                                    "/"
                                    (note.section)
                                    "/"
                                    (index)
                                    " of "
                                    (total)
                                </span>
                            }
                            <span class="chip">
                                (note.minutes)
                                " min read"
                            </span>
                            <span class="chip">
                                (note.words)
                                " words"
                            </span>
                            if !note.updated.is_empty() {
                                <time
                                    class="chip"
                                    datetime=(note.updated)
                                    title=(format!("Last verified {}", note.updated))
                                >
                                    // These notes are toolchain guides, so how
                                    // current they are is the first thing a
                                    // reader needs. The exact date is in the
                                    // rail; the chip answers the question the
                                    // reader actually has, which is "how long
                                    // ago".
                                    "Verified "
                                    (humanize_age(note.age_days))
                                </time>
                            }
                        </div>

                        // The one place the accent means something rather than
                        // decorating: a toolchain guide that has not been checked
                        // for six months may be actively misleading.
                        if note.stale {
                            <p class="stale-note">
                                <span aria-hidden="true">"⚠ "</span>
                                "Last verified "
                                (humanize_age(note.age_days))
                                " — the toolchain may have moved on since. Check the versions before copying anything."
                            </p>
                        }

                        if !note.tags.is_empty() {
                            <ul class="tag-list" aria-label="Tags">
                                for tag in note.tags {
                                    <li class="tag">(*tag)</li>
                                }
                            </ul>
                        }
                    </header>

                    if !note.toc.is_empty() {
                        <details class="toc-inline" id="toc-inline">
                            <summary>"Contents"</summary>
                            <nav class="toc" aria-label="Contents">
                                for entry in note.toc {
                                    <a
                                        class=(if entry.level >= 3 { "toc-child" } else { "" })
                                        href=(format!("#{}", entry.id))
                                    >(entry.title)</a>
                                }
                            </nav>
                        </details>
                    }

                    <div class="prose">
                        (Unescaped::new_unchecked(note.html))
                    </div>

                    doc_nav(prev: prev, next: next)
                    if !related.is_empty() {
                        related_block(notes: related)
                    }
                    discussion()
                </article>

                <aside class="doc-side" aria-label="Contents">
                    if !note.toc.is_empty() {
                        <h2>"On this page"</h2>
                        <nav class="toc" aria-label="On this page">
                            for entry in note.toc {
                                <a
                                    class=(if entry.level >= 3 { "toc-child" } else { "" })
                                    href=(format!("#{}", entry.id))
                                >(entry.title)</a>
                            }
                        </nav>
                    }
                </aside>

                <aside class="doc-rail" aria-label="Note details">
                    <dl class="doc-side__facts">
                        if let Some((index, total)) = position {
                            <div class="fact">
                                <dt>"Note"</dt>
                                <dd>
                                    (index)
                                    " / "
                                    (total)
                                </dd>
                            </div>
                        }
                        <div class="fact">
                            <dt>"Section"</dt>
                            <dd>
                                <a href=(format!("/{}/", note.section))>
                                    (site::section_title(note.section))
                                </a>
                            </dd>
                        </div>
                        if !note.date.is_empty() {
                            <div class="fact">
                                <dt>"Published"</dt>
                                <dd>(note.date)</dd>
                            </div>
                        }
                        if !note.updated.is_empty() {
                            <div class="fact">
                                <dt>"Last verified"</dt>
                                <dd>(note.updated)</dd>
                            </div>
                        }
                        if !note.tested_with.is_empty() {
                            <div class="fact">
                                <dt>"Verified with"</dt>
                                <dd>
                                    for (index, tool) in note.tested_with.iter().enumerate() {
                                        if index > 0 { <br> }
                                        (tool)
                                    }
                                </dd>
                            </div>
                        }
                        <div class="fact">
                            <dt>"Sections"</dt>
                            <dd>(note.toc.len())</dd>
                        </div>
                    </dl>
                </aside>
            </div>
        </div>
    };

    layout::document(cx, layout::PageMeta::article(note), body.boxed()).await
}

#[component]
async fn doc_nav(prev: Option<&'static Note>, next: Option<&'static Note>) -> Result<impl View> {
    let fallback_shelf = next.or(prev).map(|n| n.section).unwrap_or("toolchain");

    Ok(view! {
                <nav class="doc-nav" aria-label="Note navigation">
            match prev {
                Some(prev) => {
                    <a class="prev" href=(prev.url)>
                        <span class="dir">"← Previous note"</span>
                        <span class="name">(prev.title)</span>
                    </a>
                },
                None => {
                    <a class="prev" href=(format!("/{}/", fallback_shelf))>
                        <span class="dir">"← Back to section"</span>
                        <span class="name">"Contents"</span>
                    </a>
                },
            }
            match next {
                Some(next) => {
                    <a class="next" href=(next.url)>
                        <span class="dir">"Next note "</span>
                        <span class="name">(next.title)</span>
                    </a>
                },
                None => {
                    <a class="next" href="/">
                        <span class="dir">"Back to "</span>
                        <span class="name">"All notes"</span>
                    </a>
                },
            }
        </nav>
    })
}

#[component]
async fn related_block(notes: Vec<&'static Note>) -> Result<impl View> {
    Ok(view! {
        <section class="related" aria-labelledby="related-title">
            <h2 id="related-title">"Related notes"</h2>
            <ul class="related-list">
                for item in notes.iter().take(3) {
                    <li>
                        <a href=(item.url)>
                            <span class="related-list__section">
                                (site::section_short(item.section))
                            </span>
                            <strong>(item.title)</strong>
                        </a>
                    </li>
                }
            </ul>
        </section>
    })
}

#[component]
async fn discussion() -> Result<impl View> {
    Ok(view! {
        <section class="discussion" aria-labelledby="discussion-title">
            <div class="discussion__head">
                <p class="eyebrow">"Discussion"</p>
                <h2 id="discussion-title">"Comments and discussion"</h2>
                <p>"Sign in with GitHub to leave a comment, ask a question, or add context."</p>
                <div class="discussion__actions">
                    <a href="https://github.com/ref42/ref42/issues/new" target="_blank" rel="noopener noreferrer">
                        "Open an issue"
                    </a>
                    <a href="https://github.com/ref42/ref42/discussions" target="_blank" rel="noopener noreferrer">
                        "View discussions"
                    </a>
                </div>
            </div>

            <div class="discussion__frame">
                <script
                    src="https://giscus.app/client.js"
                    data-repo="ref42/ref42"
                    data-repo-id="R_kgDOS0bBzQ"
                    data-category="General"
                    data-category-id="DIC_kwDOS0bBzc4C-zX_"
                    data-mapping="pathname"
                    data-strict="0"
                    data-reactions-enabled="1"
                    data-emit-metadata="0"
                    data-input-position="top"
                    data-theme="dark_dimmed"
                    data-lang="en"
                    crossorigin="anonymous"
                    async=""
                ></script>
                <noscript>
                    <p>"Enable JavaScript to load the GitHub Discussions comments."</p>
                </noscript>
            </div>
        </section>
    })
}
