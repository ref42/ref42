#!/usr/bin/env python3
"""Generate WebP siblings for the images under `static/`.

The site prefers WebP, and the decision is made at build time from what is on
disk: `build.rs` wraps an `<img>` in a `<picture>` with a WebP `<source>` when a
`.webp` file sits next to the source image *and is smaller than it*. So nothing
needs configuring — but the `.webp` file has to exist, and this is what puts it
there.

Run it after adding screenshots:

    python scripts/optimize-images.py

Requires Pillow (`pip install Pillow`). It is safe to run repeatedly:

  * a `.webp` that is newer than its source is left alone, unless `--force`;
  * a conversion that would come out *larger* is discarded, because offering a
    heavier image defeats the point — one flat KiCad diagram re-encoded at
    quality 82 was 5% bigger than the PNG it came from;
  * originals are never modified or deleted. The PNG stays as the fallback.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from urllib.parse import unquote

try:
    from PIL import Image, UnidentifiedImageError
except ImportError:  # pragma: no cover - the message is the feature
    sys.exit("Pillow is required: pip install Pillow")

# Anything this big is almost certainly an export mistake rather than an
# intention: the site's own logo arrived as a 17067x17067 PNG rendered at 44px,
# which cost 2.4 MB and 291 megapixels of decoding on every page view.
SUSPICIOUS_PIXELS = 4096

RASTER = {".png", ".jpg", ".jpeg"}
ROOT = Path(__file__).resolve().parent.parent / "static"
CONTENT = ROOT.parent / "content"

MARKDOWN_IMAGE = re.compile(r"!\[[^\]]*\]\(([^)\s]+)")
HTML_IMAGE = re.compile(r"""<img[^>]*\bsrc=["']([^"']+)["']""", re.IGNORECASE)

# A `.webp` counts as current when it is not meaningfully older than its source.
# Comparing mtimes exactly does not survive a `git clone`, where every file gets
# much the same timestamp and the order between a PNG and its sibling is
# arbitrary — that alone re-encoded seven files that needed nothing. A source
# genuinely replaced later is minutes or days newer, so a small tolerance
# separates "cloned" from "changed" without a manifest of hashes.
STALE_AFTER_SECONDS = 30


def human(size: int) -> str:
    if size >= 1024 * 1024:
        return f"{size / (1024 * 1024):.1f} MB"
    return f"{size / 1024:.0f} KB"


def convert(source: Path, quality: int, force: bool) -> tuple[str, int, int]:
    """Returns (outcome, before, after). before==after means unchanged."""
    target = source.with_suffix(".webp")
    before = source.stat().st_size

    if target.exists() and not force:
        age = source.stat().st_mtime - target.stat().st_mtime
        if age < STALE_AFTER_SECONDS:
            return "current", before, target.stat().st_size

    try:
        with Image.open(source) as image:
            width, height = image.size
            if max(width, height) > SUSPICIOUS_PIXELS:
                print(
                    f"  ! {source.relative_to(ROOT.parent)} is {width}x{height}; "
                    f"consider re-exporting it smaller"
                )
            # A palette or greyscale PNG is still just image data to Pillow.
            if image.mode in ("P", "LA", "RGBA") or "transparency" in image.info:
                image = image.convert("RGBA")
            elif image.mode != "RGB":
                image = image.convert("RGB")

            image.save(target, "WEBP", quality=quality, method=6)
    except UnidentifiedImageError:
        return "unreadable", before, before
    except Image.DecompressionBombError:
        print(f"  ! {source.name} refused by Pillow as a decompression bomb; skipped")
        return "skipped", before, before

    after = target.stat().st_size
    if after >= before:
        # Keep the PNG; a bigger WebP is worse than no WebP.
        target.unlink()
        return "no-gain", before, before

    return "wrote", before, after


def referenced_images() -> tuple[set[Path], set[str]]:
    """Images the notes point at, and references that point at nothing.

    Only referenced images are worth converting. `build.rs` wraps an image that
    came through note markdown in a `<picture>` when a sibling `.webp` exists,
    but it never touches the favicon, the OG tile or the header logo, which the
    layout links directly — a `.webp` beside those would sit in the repository
    and never be fetched by anything. That is four files, and it is exactly what
    a naive `rglob` over `static/` produces.

    The dangling references are returned because they are a typo the build will
    only report as a warning, and one worth seeing while you are already here.
    """
    found: set[Path] = set()
    missing: set[str] = set()

    for page in sorted(CONTENT.rglob("*.md")):
        text = page.read_text(encoding="utf-8", errors="replace")
        for pattern in (MARKDOWN_IMAGE, HTML_IMAGE):
            for match in pattern.finditer(text):
                src = match.group(1).strip()
                if src.startswith(("http://", "https://", "data:", "//")):
                    continue
                # Written root-relative (`/esp32/c3.png`) from the site root,
                # which is `static/esp32/c3.png` on disk.
                candidate = ROOT / unquote(src).lstrip("/")
                if candidate.suffix.lower() not in RASTER:
                    continue
                if candidate.is_file():
                    found.add(candidate)
                else:
                    missing.add(src)

    return found, missing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--quality", type=int, default=82, help="WebP quality (default 82)")
    parser.add_argument("--force", action="store_true", help="re-encode even up-to-date files")
    parser.add_argument("--dry-run", action="store_true", help="report without writing anything")
    args = parser.parse_args()

    if not ROOT.is_dir():
        sys.exit(f"no {ROOT} directory")

    sources, missing = referenced_images()
    sources = sorted(sources)

    if missing:
        print(f"{len(missing)} reference(s) point at a file that is not there:")
        for src in sorted(missing):
            print(f"  ! {src}")
        print("")

    if not sources:
        print(f"no images under {ROOT} are referenced by content/")
        return 0

    if args.dry_run:
        print(f"would examine {len(sources)} referenced image(s)\n")

    wrote = current = no_gain = 0
    before_total = after_total = 0

    for source in sources:
        relative = source.relative_to(ROOT)
        if args.dry_run:
            print(f"  {relative}")
            continue

        outcome, before, after = convert(source, args.quality, args.force)
        before_total += before
        after_total += after

        if outcome == "wrote":
            wrote += 1
            print(f"  wrote    {relative.with_suffix('.webp')}   {human(before)} -> {human(after)}")
        elif outcome == "no-gain":
            no_gain += 1
            print(f"  kept png {relative}   WebP came out larger, so it was discarded")
        elif outcome == "current":
            current += 1
        elif outcome in ("unreadable", "skipped"):
            print(f"  skipped  {relative}   ({outcome})")

    if args.dry_run:
        return 0

    print(f"\n{wrote} written, {current} already current, {no_gain} without a gain")
    # `before` is always the source file and `after` is what a browser would
    # actually fetch, so this is the site's image weight, not just this run's.
    print(f"static/ rasters: {human(before_total)} as source, {human(after_total)} as served")
    if wrote:
        print("commit the .webp files; the build offers them automatically")
    return 0


if __name__ == "__main__":
    sys.exit(main())
