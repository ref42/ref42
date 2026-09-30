//! Shared inline SVG icons, so the site needs no icon font or network request.

use topcoat::{
    Result,
    view::{View, component, view},
};

/// The `<path>` children of an icon, for use inside an `<svg>` element the
/// caller writes itself. This keeps icon markup inline instead of nesting a
/// second `<svg>`.
#[component]
pub async fn paths(packed: &str) -> Result<impl View> {
    Ok(view! {
        for d in packed.split('|') {
            <path d=(d) />
        }
    })
}

/// A 24×24 stroked icon. `packed` holds one or more path `d` values separated
/// by `|`, so every icon is a single `&'static str`.
///
/// Pages normally inline `<svg>` themselves and call `paths` for the children;
/// this is the standalone form, kept for anywhere a complete element is
/// handier than open-coded markup.
#[component]
#[allow(dead_code)]
pub async fn icon(packed: &str) -> Result<impl View> {
    Ok(view! {
        <svg
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            for d in packed.split('|') {
                <path d=(d) />
            }
        </svg>
    })
}

pub const ARROW_RIGHT: &str = "M5 12h14|m13 6 6 6-6 6";
pub const ARROW_UP: &str = "M12 19V5|m6 11 6-6 6 6";
pub const ARROW_LEFT: &str = "M19 12H5|m11 18-6-6 6-6";
pub const SEARCH: &str = "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z|m21 21-4.3-4.3";
pub const MENU: &str = "M3.5 7h17|M3.5 12h17|M3.5 17h17";
pub const GITHUB: &str = "M9 19.2c-4 1.2-4-2-5.6-2.5m11.2 5.1v-3.2a2.8 2.8 0 0 0-.8-2.2c2.6-.3 5.4-1.3 5.4-5.8a4.6 4.6 0 0 0-1.3-3.2 4.3 4.3 0 0 0-.1-3.2s-1.4-.4-4.5 1.7a11.2 11.2 0 0 0-5.8 0C4.4 2.5 3 2.9 3 2.9a4.3 4.3 0 0 0-.1 3.2A4.6 4.6 0 0 0 1.6 9.4c0 4.5 2.8 5.5 5.4 5.8a2.8 2.8 0 0 0-.8 2.2v3.2";
pub const BILIBILI: &str =
    "M6.5 3.5 9 6.5|M17.5 3.5 15 6.5|M4.5 6.5h15v14h-15z|M9.5 12.5v2.5|M14.5 12.5v2.5";

/// The shelf glyph for a section key.
#[allow(dead_code)]
pub fn shelf(key: &str) -> &'static str {
    match key {
        "toolchain" => {
            "M14.7 6.3a4.5 4.5 0 0 0 5.9 5.9l-8.4 8.4a2.1 2.1 0 0 1-3-3l8.4-8.4Z|M14.7 6.3 17.9 3.1"
        }
        "esp32" => {
            "M7 7h10v10H7z|M9.5 2.5v4|M14.5 2.5v4|M9.5 17.5v4|M14.5 17.5v4|M2.5 9.5h4|M2.5 14.5h4|M17.5 9.5h4|M17.5 14.5h4"
        }
        "stm32" => "m12 2.8 8.5 4.7v9.4L12 21.2 3.5 16.9V7.5Z|M3.5 7.5 12 12l8.5-4.5|M12 12v9.2",
        _ => "M3 8.5h18v11H3z|M8.5 8.5V6a2 2 0 0 1 2-2h3a2 2 0 0 1 2 2v2.5|M3 13h18",
    }
}
