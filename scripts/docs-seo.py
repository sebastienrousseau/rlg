#!/usr/bin/env python3
"""Add search-engine metadata to the built API docs site.

rustdoc pages carry no canonical link, and its search box rewrites the
address to `?search=...`, so crawlers find an endless set of duplicate
URLs for every item page. Run after `cargo doc` and `mdbook build`,
before the site is uploaded, this:

- adds `<link rel="canonical">` to every page that is not a redirect
  and has none;
- marks rustdoc's `help.html` and `settings.html` `noindex`;
- writes `sitemap.xml` listing the pages that exist in this build, so
  removed items stop being rediscovered;
- writes `robots.txt`, which keeps crawlers off `?search=` URLs and
  the `src/` source views and points them at the sitemap.

Usage: scripts/docs-seo.py [--check] <site-dir> <base-url>

With --check, it changes nothing and fails unless the site already
carries all of the above.
"""

import re
import sys
from pathlib import Path
from xml.sax.saxutils import escape

NOINDEX_PAGES = {"help.html", "settings.html"}
NOINDEX_TAG = '<meta name="robots" content="noindex">'
REDIRECT = re.compile(r'<meta\s+http-equiv="refresh"', re.I)
CANONICAL = re.compile(r'<link\s+rel="canonical"\s+href="([^"]*)"', re.I)
NOINDEX = re.compile(r'<meta\s+name="robots"\s+content="[^"]*noindex', re.I)
ROBOTS_RULES = ("Disallow: /*?search=", "Disallow: /*&search=", "Disallow: /src/")


def pages(site):
    """Every HTML page in the site, as (path relative to site, text)."""
    for path in sorted(site.rglob("*.html")):
        yield path.relative_to(site).as_posix(), path.read_text(encoding="utf-8")


def wants_canonical(rel, text):
    """Redirect stubs and 404 pages must not declare a canonical."""
    return not (REDIRECT.search(text) or rel.endswith("404.html"))


def in_sitemap(rel, text):
    """Pages a search engine should index."""
    if rel.startswith("src/") or not wants_canonical(rel, text):
        return False
    return not (rel in NOINDEX_PAGES or NOINDEX.search(text))


def add_to_head(text, tag):
    """Insert tag just before </head>."""
    head_end = text.lower().find("</head>")
    if head_end < 0:
        return text
    return text[:head_end] + tag + text[head_end:]


def annotate(rel, text, base):
    """Return the page with its canonical and robots tags added."""
    if wants_canonical(rel, text) and not CANONICAL.search(text):
        text = add_to_head(text, f'<link rel="canonical" href="{base}/{rel}">')
    if rel in NOINDEX_PAGES and not NOINDEX.search(text):
        text = add_to_head(text, NOINDEX_TAG)
    return text


def sitemap(urls):
    """A sitemap.xml document for the given URLs."""
    body = "".join(f"  <url><loc>{escape(url)}</loc></url>\n" for url in urls)
    return (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        f"{body}</urlset>\n"
    )


def robots(base):
    """A robots.txt that blocks search URLs and source views."""
    rules = "\n".join(ROBOTS_RULES)
    return f"User-agent: *\n{rules}\n\nSitemap: {base}/sitemap.xml\n"


def write(site, base):
    """Annotate every page and write sitemap.xml and robots.txt."""
    urls = []
    for rel, text in pages(site):
        new = annotate(rel, text, base)
        if new != text:
            (site / rel).write_text(new, encoding="utf-8")
        if in_sitemap(rel, new):
            urls.append(f"{base}/{rel}")
    (site / "sitemap.xml").write_text(sitemap(urls), encoding="utf-8")
    (site / "robots.txt").write_text(robots(base), encoding="utf-8")
    print(f"docs-seo: {len(urls)} pages in sitemap.xml, robots.txt written")


def page_problems(site, base):
    """Pages missing or misdeclaring their metadata."""
    problems = []
    for rel, text in pages(site):
        found = CANONICAL.findall(text)
        if not wants_canonical(rel, text):
            continue
        if found != [f"{base}/{rel}"]:
            problems.append(f"{rel}: canonical {found or 'missing'}")
        if rel in NOINDEX_PAGES and not NOINDEX.search(text):
            problems.append(f"{rel}: not noindex")
    return problems


def sitemap_problems(site, base):
    """Sitemap entries that are not indexable pages of this build."""
    path = site / "sitemap.xml"
    if not path.is_file():
        return ["sitemap.xml missing"]
    problems = []
    locs = re.findall(r"<loc>([^<]*)</loc>", path.read_text(encoding="utf-8"))
    for url in locs:
        rel = url.removeprefix(f"{base}/")
        page = site / rel
        if rel == url or not page.is_file():
            problems.append(f"sitemap: {url} is not a page of this site")
        elif not in_sitemap(rel, page.read_text(encoding="utf-8")):
            problems.append(f"sitemap: {url} should not be indexed")
    return problems or ([] if locs else ["sitemap.xml lists no pages"])


def robots_problems(site, base):
    """What robots.txt is missing."""
    path = site / "robots.txt"
    text = path.read_text(encoding="utf-8") if path.is_file() else ""
    wanted = [*ROBOTS_RULES, f"Sitemap: {base}/sitemap.xml"]
    return [f"robots.txt: missing {line!r}" for line in wanted if line not in text]


def check(site, base):
    """Fail unless every page, the sitemap and robots.txt are in order."""
    problems = [
        *page_problems(site, base),
        *sitemap_problems(site, base),
        *robots_problems(site, base),
    ]
    for problem in problems:
        print(f"docs-seo: {problem}", file=sys.stderr)
    if not problems:
        print("docs-seo: canonical, sitemap and robots.txt in order")
    return 1 if problems else 0


def main(argv):
    """Parse the command line and run."""
    checking = argv[:1] == ["--check"]
    args = argv[1:] if checking else argv
    if len(args) != 2 or not Path(args[0]).is_dir():
        print(__doc__, file=sys.stderr)
        return 2
    site, base = Path(args[0]), args[1].rstrip("/")
    if checking:
        return check(site, base)
    write(site, base)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
