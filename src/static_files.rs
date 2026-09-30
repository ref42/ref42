//! Serves the images, fonts, stylesheet and client script from `static/`,
//! mirroring it into `assets/` at build time.
//!
//! Every page on the site sits behind a single catch-all, so the layer has to
//! recognise content URLs itself and step aside for them. It is also the one
//! place that sees every response, which makes it the right place for the
//! security headers and the cache policy.

use std::path::{Component, Path as FsPath, PathBuf};

use topcoat::{
    Result,
    context::Cx,
    router::{
        Body, HeaderMap, HeaderName, HeaderValue, Next, StatusCode, header, layer,
        request::{headers, uri},
        response::Response,
    },
};

use crate::pipeline::short_hash;
use crate::site::{ASSET_VERSION, FONT_VERSION};

/// The content security policy every response carries.
///
/// There is no inline script left anywhere on the site — the search index moved
/// into a fetched `.json` file — so `script-src` needs no hash, no nonce and no
/// `'unsafe-inline'`. That is the whole reason this can be strict.
///
/// `style-src` allows giscus, and that is not cosmetic. giscus injects a
/// stylesheet *into this page* (`https://giscus.app/default.css`) whose rules set
/// `width: 100%` on its iframe; without it the frame keeps the browser's 300px
/// default and sits marooned inside a 798px container. It was blocked here, and
/// the only visible symptom was a comment box that looked too narrow — no error
/// on the page, just one line in the console.
///
/// `'unsafe-inline'` covers the shelf "next section" link's inline style. There
/// is no inline script, so this does not weaken `script-src`.
/// `upgrade-insecure-requests` is deliberately absent: it would rewrite relative
/// asset URLs to `https:` on a local HTTP dev server and break it.
const CSP: &str = "default-src 'self'; \
     script-src 'self' https://giscus.app; \
     frame-src https://giscus.app; \
     frame-ancestors 'none'; \
     img-src 'self' data:; \
     style-src 'self' 'unsafe-inline' https://giscus.app; \
     font-src 'self'; \
     base-uri 'self'; \
     form-action 'self'; \
     object-src 'none'";

/// Streams a static file when the request names one; otherwise hands the
/// request to the page dispatcher.
#[layer("/")]
pub async fn serve_static(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let path = uri(cx).path().to_string();
    let trimmed = path.trim_matches('/');

    // The site root and every content URL belong to the page dispatcher.
    if trimmed.is_empty() || crate::site::is_content_path(trimmed) {
        return dispatch_to_page(cx, body, next).await;
    }

    let segments: Vec<&str> = trimmed.split('/').collect();
    let Some(full) = resolve(&segments) else {
        return dispatch_to_page(cx, body, next).await;
    };

    let Ok(bytes) = tokio::fs::read(&full).await else {
        return dispatch_to_page(cx, body, next).await;
    };

    let cache = cache_control(cx, &full);
    // A strong validator computed from the bytes that were just read: the hash
    // is nearly free, and it is what makes `no-cache` cheap instead of a full
    // re-download of the stylesheet on every single navigation.
    let etag = format!("\"{}\"", short_hash(&bytes));

    if is_fresh(cx, &etag) {
        let mut response = Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header("etag", etag)
            .header("cache-control", cache)
            .body(Body::from(()))?;
        harden(response.headers_mut());
        return Ok(response);
    }

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type(&full))
        .header("cache-control", cache)
        .header("etag", etag)
        .body(Body::from(bytes))?;
    harden(response.headers_mut());

    Ok(response)
}

/// Renders the page dispatcher's response with the page cache policy applied.
async fn dispatch_to_page(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let mut response = next.run(cx, body).await?;
    // Pages are revalidated rather than heuristically cached. Without an
    // explicit policy the browser is free to invent a freshness lifetime for a
    // 200 response, which is how a redeploy stays invisible for hours.
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    harden(response.headers_mut());
    Ok(response)
}

/// True when the client's `If-None-Match` already holds this exact tag.
fn is_fresh(cx: &Cx, etag: &str) -> bool {
    let Some(value) = headers(cx).get(header::IF_NONE_MATCH) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };

    value.split(',').any(|candidate| {
        let candidate = candidate.trim();
        // `W/` marks a weak validator; the comparison is the same either way
        // for the GET-only, byte-identical bodies served here.
        candidate == "*"
            || candidate == etag
            || candidate
                .strip_prefix("W/")
                .is_some_and(|weak| weak == etag)
    })
}

/// Headers that describe, rather than depend on, the response body.
fn harden(headers: &mut HeaderMap) {
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(CSP),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );
}

/// Cache policy for one asset.
///
/// Stylesheets, scripts and fonts are requested with the build's content hash
/// in `?v=`. A URL carrying the current version can never go stale, so it may
/// be cached forever; anything requested without it — a stale URL from a cached
/// page, or somebody typing the path by hand — is revalidated instead.
fn cache_control(cx: &Cx, path: &FsPath) -> &'static str {
    let version = uri(cx).query().and_then(version_param);
    if matches!(version, Some(version) if version == ASSET_VERSION || version == FONT_VERSION) {
        return "public, max-age=31536000, immutable";
    }

    match extension(path).as_str() {
        // Text that describes the site rather than being a picture of it: the
        // feed, the sitemap and `robots.txt` belong here too, because a reader
        // that gets yesterday's feed is worse than one that waits 100 ms.
        "css" | "js" | "mjs" | "html" | "json" | "xml" | "txt" => "no-cache",
        _ => "public, max-age=86400",
    }
}

/// The `v` parameter of a query string, if it has one.
fn version_param(query: &str) -> Option<&str> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == "v").then_some(value)
    })
}

fn extension(path: &FsPath) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Resolves URL segments under `assets/`, refusing anything that could climb
/// out of it.
fn resolve(segments: &[&str]) -> Option<PathBuf> {
    if segments.is_empty() {
        return None;
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut out = root.clone();

    for segment in segments {
        for part in FsPath::new(segment).components() {
            match part {
                Component::Normal(name) => out.push(name),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
            }
        }
    }

    (out != root && out.starts_with(&root)).then_some(out)
}

/// Small, allocation-light MIME table for the file types this site ships.
fn content_type(path: &FsPath) -> &'static str {
    match extension(path).as_str() {
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        _ => "application/octet-stream",
    }
}
