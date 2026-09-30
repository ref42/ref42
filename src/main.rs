//! ref42 — my daily notes, rebuilt on [Topcoat](https://github.com/tokio-rs/topcoat).
//!
//! Every note is rendered from the markdown in `content/` at build time
//! (`build.rs`), so the running server has no markdown or filesystem
//! dependency for content, and the URLs are unchanged from the previous site.

mod icons;
mod layout;
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

use topcoat::router::{Router, RouterBuilderDiscoverExt};

#[tokio::main]
async fn main() {
    // `discover()` picks up the annotated `#[page]` in `pages`; the static
    // file layer is registered by hand.
    let router = Router::builder()
        .layer(static_files::serve_static)
        .discover()
        .build();

    topcoat::start(router).await.unwrap();
}
