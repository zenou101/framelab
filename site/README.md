# FrameLab website

The official landing page for FrameLab, published with GitHub Pages at
<https://zenou101.github.io/framelab/>.

The site is deliberately separate from the desktop application. It is plain HTML, CSS and JavaScript
built with Vite; there is no React runtime, router or UI framework, and no dependency on the Tauri
frontend.

```
site/
  index.html              landing page
  404.html                custom not-found page
  src/styles.css          design tokens, layout and responsive rules
  src/main.js             navigation, gallery, release lookup, reveal transitions
  public/                 favicon, mark, Open Graph card, screenshots, robots, sitemap
  scripts/build-assets.mjs  one-off image pipeline (not part of the build)
  vite.config.mjs         base path and multi-page build
```

## Develop

```bash
cd site
npm ci
npm run dev
```

Vite serves the site at <http://127.0.0.1:4310/framelab/> because the build is configured for the
repository's project-pages subpath.

## Build

```bash
cd site
npm run build     # writes site/dist
npm run preview   # serves the production build at http://127.0.0.1:4311/framelab/
```

The build emits `index.html` and `404.html`. All asset references are rebased for `/framelab/`, so
the output can be served from the project-pages URL as-is.

## Base path

`vite.config.mjs` sets `base: "/framelab/"`. If the repository is renamed or the site moves to a
custom domain, build with `SITE_BASE=/ npm run build` (or the appropriate path) and update the
absolute URLs that cannot be rebased:

- `index.html` — canonical, Open Graph, Twitter and JSON-LD URLs
- `public/sitemap.xml`, `public/robots.txt`
- `404.html` — uses the Vite base automatically; no absolute URLs

## Screenshots and Open Graph image

The four screenshots in `public/screenshots/` are real captures of FrameLab 0.1.0 on Windows 10
22H2. Do not substitute mockups or composited data.

To regenerate the WebP versions, thumbnails, favicons and `og.png` from the original PNG captures,
run this from the repository root (it reuses the desktop app's `sharp` dependency):

```bash
node site/scripts/build-assets.mjs
```

The script reads `.tmp/window2.png`, `.tmp/settings-test.png`, `.tmp/light.png` and
`.tmp/legal-about.png`. `.tmp/` is scratch space and is not committed, so keep new captures in the
same place or adjust the paths at the top of the script. The generated files are committed, so the
site build does not need `sharp`.

## Download data

The page does not hardcode a version. On load it requests the latest published release from the
GitHub REST API and fills in the release tag, installer/portable file names, sizes and direct asset
links. The request is cached in `sessionStorage` for ten minutes. If the API is unavailable, the
buttons fall back to `https://github.com/zenou101/framelab/releases/latest`, so downloads still work.

This is the only network request the website makes (plus the README/legal links when clicked).

## Deploy

`.github/workflows/pages.yml` builds the site and deploys it on every push to `main` that changes
`site/**` or the workflow itself. It can also be run from the Actions tab with **Deploy website →
Run workflow**.

One-time repository setting: **Settings → Pages → Build and deployment → Source: GitHub Actions**.
No branch, `gh-pages` folder or custom domain is required. The workflow uses the standard Pages
actions and the `github-pages` environment; it does not touch the application CI workflow
(`.github/workflows/ci.yml`) or release assets.
