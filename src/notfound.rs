//! 404 — kept in the spirit of the original panic-styled page.

use std::fmt::Write as _;

use topcoat::{
    Result,
    context::Cx,
    router::request::uri,
    view::{Unescaped, View, view},
};

use crate::icons;
use crate::icons::paths;
use crate::pipeline::xml_escape;
use crate::site::{self, Note, SECTIONS};

/// The panic transcript for the path that was actually requested.
///
/// The transcript is hand-written HTML and is rendered unescaped, so the request
/// path enters through exactly one door and is escaped there: it is text from
/// the URL bar, and treating it as markup would be an injection.
///
/// The static version of this said `open /missing` whatever you had typed, which
/// made the one page whose whole job is to notice a mistake the one page that
/// ignored it.
fn transcript(path: &str, suggestion: Option<&Note>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<span class=\"dim\">$</span> RUST_BACKTRACE=1 ref42 open {}",
        xml_escape(path)
    );
    out.push_str("<span class=\"err\">thread 'main' panicked</span> at 'route not found',\n");
    out.push_str("src/router.rs:404:9\n\n");
    out.push_str("stack backtrace:\n");
    out.push_str("   0: ref42::router::resolve\n");
    out.push_str("   1: ref42::notes::open\n");
    out.push_str("   2: core::ops::function::FnOnce::call_once\n\n");

    match suggestion {
        Some(note) => {
            let _ = write!(
                out,
                "<span class=\"cmt\">help:</span> did you mean <span class=\"ok\">`{}`</span>?",
                xml_escape(note.url)
            );
        }
        None => out.push_str(
            "<span class=\"cmt\">help:</span> run <span class=\"ok\">`cargo run -- /`</span> to return home",
        ),
    }

    out
}

/// A 404 body. Callers wrap it in the document shell.
pub async fn page(cx: &Cx) -> Result<impl View> {
    let path = uri(cx).path().to_string();
    let suggestion = site::closest_note(path.trim_matches('/'));
    let transcript = transcript(&path, suggestion);

    Ok(view! { cx =>
        <div class="shell">
            <section class="panic" aria-labelledby="panic-title">
                <div class="panic__copy">
                    <p class="eyebrow">"404 / panic"</p>
                    <h1 id="panic-title">
                        "thread 'main' panicked"
                        <span class="caret" aria-hidden="true"></span>
                    </h1>
                    <p class="lead">
                        "The route you asked for was not found in this build."
                        " This note does not exist. The link may be wrong, or the note has not been published yet."
                    </p>

                    // Offering the note that was probably meant, with its real
                    // title, is the difference between a themed 404 and one that
                    // is actually useful. It is a plain link rather than the
                    // primary button: titles are long, and a button that wraps to
                    // three lines reads worse than a sentence.
                    if let Some(note) = suggestion {
                        <p class="panic__meant">
                            "Did you mean "
                            <a href=(note.url)>(note.title)</a>
                            "?"
                        </p>
                    }

                    <div class="panic__actions">
                        <a class="btn btn--primary" href="/">
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                aria-hidden="true"
                                stroke-width="1.7"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                paths(packed: icons::ARROW_LEFT)
                            </svg>
                            "Back home"
                        </a>
                        <a class="btn" href="/toolchain/">
                            "Open toolchain section"
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
                        </a>
                    </div>

                    <div class="panic__suggest">
                        for section in SECTIONS {
                            <a class="chip" href=(format!("/{}/", section.key))>
                                "/"
                                (section.key)
                            </a>
                        }
                    </div>
                </div>

                <pre class="panic__window" aria-label="Rust panic backtrace"><code>(Unescaped::new_unchecked(transcript))</code></pre>
            </section>
        </div>
    })
}
