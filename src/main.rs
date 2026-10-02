//! ref42 — my daily notes, rebuilt on [Topcoat](https://github.com/tokio-rs/topcoat).
//!
//! Every note is rendered from the markdown in `content/` at build time
//! (`build.rs`), so the running server has no markdown or filesystem
//! dependency for content, and the URLs are unchanged from the previous site.
//!
//! The binary has two jobs. With no arguments it serves the site, which is all
//! the deploy ever asks of it. `ref42 new` writes a note into `content/`, and
//! lives here rather than in a script so the rules it writes are the rules
//! `build.rs` reads — same module, so they cannot drift.

mod icons;
mod layout;
mod new_note;
mod note;
mod notfound;
mod pages;
mod site;
mod static_files;

// The content transforms live in `src/pipeline.rs` because a build script is
// not a test target: `cargo test` never runs `#[cfg(test)]` modules inside
// `build.rs`. Declaring the same file here is what makes those transforms
// testable, and `build.rs` includes it by path so the two targets cannot drift
// apart. It is compiled unconditionally because the static file layer reuses
// the content hash for its ETags.
mod pipeline;

use std::process::ExitCode;

use topcoat::router::{Router, RouterBuilderDiscoverExt};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        // `generate` and `gen` because that is the verb people reach for; `new`
        // is what the usage text advertises.
        Some("new" | "generate" | "gen") => new_note::run(&args[1..]),
        Some("help" | "--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        None => serve(),
        Some(other) => {
            eprintln!("ref42: unknown command `{other}`\n");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

const USAGE: &str = "\
ref42 — the notes site

    ref42                 serve the site (HOST and PORT, default 127.0.0.1:3000)
    ref42 new <title>     start a note, see `ref42 new --help`
    ref42 help            this

Previewing is `cargo run`; the deploy runs the binary with no arguments.";

#[tokio::main]
async fn serve() -> ExitCode {
    // `discover()` picks up the annotated `#[page]` in `pages`; the static
    // file layer is registered by hand.
    let router = Router::builder()
        .layer(static_files::serve_static)
        .discover()
        .build();

    match topcoat::start(router).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ref42: {err}");
            ExitCode::FAILURE
        }
    }
}
