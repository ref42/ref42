/* ==========================================================================
   ref42 · static export

   Renders the running site to plain files, so it can be published on static
   hosting instead of needing a process that stays up.

   Every page on this site is a pure function of the content that was compiled
   into the binary plus the request path, so "prerendering" is really just
   fetching each URL once and writing the bytes out. Crawling the generated
   sitemap means the export and the site can never disagree about which URLs
   exist.

   Usage:
     node scripts/export.mjs [--out dist] [--base http://127.0.0.1:3000]

   The server is expected to already be running (the deploy workflow starts it).
   ========================================================================== */

import { mkdir, cp, writeFile, rm, stat } from "node:fs/promises";
import { dirname, join } from "node:path";

const args = process.argv.slice(2);

function arg(name, fallback) {
  const index = args.indexOf(`--${name}`);
  return index === -1 ? fallback : args[index + 1];
}

const outDir = arg("out", "dist");
/* A trailing slash here would produce `//main.css` in every asset URL. */
const base = arg("base", "http://127.0.0.1:3000").replace(/\/+$/, "");

/** Every URL the site publishes, taken from its own sitemap. */
async function sitemapPaths() {
  const response = await fetch(`${base}/sitemap.xml`);
  if (!response.ok) {
    throw new Error(`sitemap.xml returned ${response.status}; is the server running?`);
  }

  const xml = await response.text();
  const paths = [...xml.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => {
    /* The sitemap holds absolute URLs; the export needs just the path. */
    const url = new URL(match[1]);
    return url.pathname;
  });

  if (paths.length === 0) {
    throw new Error("sitemap.xml listed no URLs");
  }
  return paths;
}

async function fetchPage(path) {
  const response = await fetch(`${base}${path}`);
  if (!response.ok) {
    throw new Error(`${path} returned ${response.status}`);
  }
  return response.text();
}

/** `/toolchain/cortex-m/` becomes `<out>/toolchain/cortex-m/index.html`. */
function pageFile(path) {
  return join(outDir, path, "index.html");
}

async function main() {
  await rm(outDir, { recursive: true, force: true });
  await mkdir(outDir, { recursive: true });

  const paths = await sitemapPaths();
  let pages = 0;
  let canonicalHost = "";

  for (const path of paths) {
    const html = await fetchPage(path);

    /* The build's own canonical link is the authority on where the site thinks
       it is published, so the CNAME comes from there rather than from a second
       environment variable that could disagree with it. */
    if (path === "/") {
      const match = html.match(/<link rel="canonical" href="([^"]+)"/);
      if (match) canonicalHost = new URL(match[1]).host;
    }

    const file = pageFile(path);
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, html);
    pages += 1;
  }

  /* The server's own 404 has the status and the body, so it is fetched rather
     than reconstructed. GitHub Pages serves `404.html` for unknown paths. */
  const notFound = await fetch(`${base}/404.html`);
  await writeFile(join(outDir, "404.html"), await notFound.text());

  /* Assets are copied from disk rather than crawled: the served tree is exactly
     `assets/` mapped onto the site root, and copying cannot miss a file that no
     page happens to reference (the webfont is referenced from CSS, the WebP
     sources from `<picture>`). */
  const assets = join(process.cwd(), "assets");
  await stat(assets);
  await cp(assets, outDir, { recursive: true });

  /* A custom domain on GitHub Pages is configured by a CNAME file in the
     published tree. A github.io address needs none, and neither does a
     localhost build. */
  if (canonicalHost && !canonicalHost.endsWith(".github.io") && !canonicalHost.startsWith("127.0.0.1")) {
    await writeFile(join(outDir, "CNAME"), `${canonicalHost}\n`);
  }

  console.log(
    `exported ${pages} pages and the asset tree to ${outDir}/ ` +
      `(from ${base}${canonicalHost ? `, canonical host ${canonicalHost}` : ""})`
  );
}

main().catch((error) => {
  console.error(`export failed: ${error.message}`);
  process.exit(1);
});
