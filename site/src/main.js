import "./styles.css";

const ready = (fn) => {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", fn, { once: true });
  } else {
    fn();
  }
};

ready(() => {
  setupHeader();
  setupMobileNav();
  setupReveal();
  setupGallery();
  loadLatestRelease();
});

function setupHeader() {
  const header = document.querySelector("[data-header]");
  if (!header) return;

  const sync = () => header.classList.toggle("is-scrolled", window.scrollY > 6);
  sync();
  window.addEventListener("scroll", sync, { passive: true });
}

function setupMobileNav() {
  const toggle = document.querySelector("[data-nav-toggle]");
  const panel = document.querySelector("[data-mobile-nav]");
  if (!toggle || !panel) return;

  const label = toggle.querySelector(".sr-only");
  const links = panel.querySelectorAll("a");

  const setOpen = (open) => {
    toggle.setAttribute("aria-expanded", String(open));
    if (label) label.textContent = open ? "Close menu" : "Open menu";
    panel.hidden = !open;
    document.body.classList.toggle("nav-open", open);
  };

  toggle.addEventListener("click", () => {
    setOpen(toggle.getAttribute("aria-expanded") !== "true");
  });

  links.forEach((link) => link.addEventListener("click", () => setOpen(false)));

  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (toggle.getAttribute("aria-expanded") !== "true") return;
    setOpen(false);
    toggle.focus();
  });

  window.addEventListener("resize", () => {
    if (window.innerWidth > 1020) setOpen(false);
  });
}

function setupReveal() {
  const elements = Array.from(document.querySelectorAll("[data-reveal]"));
  if (!elements.length) return;

  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduce || !("IntersectionObserver" in window)) {
    elements.forEach((element) => element.classList.add("is-in"));
    return;
  }

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-in");
        observer.unobserve(entry.target);
      });
    },
    { rootMargin: "0px 0px -6% 0px", threshold: 0.05 },
  );

  elements.forEach((element) => observer.observe(element));
}

function setupGallery() {
  const gallery = document.querySelector("[data-gallery]");
  if (!gallery) return;

  const image = gallery.querySelector("[data-gallery-image]");
  const title = gallery.querySelector("[data-gallery-title]");
  const meta = gallery.querySelector("[data-gallery-meta]");
  const open = gallery.querySelector("[data-gallery-link]");
  const thumbs = Array.from(gallery.querySelectorAll("[data-gallery-thumb]"));
  if (!image || !thumbs.length) return;

  const select = (thumb) => {
    const source = thumb.dataset.src;
    if (!source) return;

    thumbs.forEach((item) => {
      const active = item === thumb;
      item.classList.toggle("is-active", active);
      if (active) item.setAttribute("aria-current", "true");
      else item.removeAttribute("aria-current");
    });

    if (title && thumb.dataset.title) title.textContent = thumb.dataset.title;
    if (meta && thumb.dataset.meta) meta.textContent = thumb.dataset.meta;
    if (open) open.href = source;
    if (thumb.dataset.alt) image.alt = thumb.dataset.alt;

    if (image.getAttribute("src") === source) return;

    image.classList.add("is-fading");
    image.src = source;

    const settle = () => image.classList.remove("is-fading");
    if (image.complete) requestAnimationFrame(settle);
    else {
      image.addEventListener("load", settle, { once: true });
      image.addEventListener("error", settle, { once: true });
      window.setTimeout(settle, 1200);
    }
  };

  thumbs.forEach((thumb) => {
    thumb.addEventListener("click", (event) => {
      event.preventDefault();
      void select(thumb);
    });
  });

  const preloadRest = () => {
    thumbs
      .filter((thumb) => !thumb.classList.contains("is-active"))
      .forEach((thumb) => {
        if (!thumb.dataset.src) return;
        const preload = new Image();
        preload.src = thumb.dataset.src;
      });
  };

  if ("requestIdleCallback" in window) window.requestIdleCallback(preloadRest, { timeout: 2500 });
  else window.setTimeout(preloadRest, 1500);
}

async function loadLatestRelease() {
  const repo = "zenou101/framelab";
  const endpoint = `https://api.github.com/repos/${repo}/releases/latest`;
  const cacheKey = "framelab-latest-release";
  const maxAge = 10 * 60 * 1000;

  let payload = null;
  try {
    const cached = sessionStorage.getItem(cacheKey);
    if (cached) {
      const parsed = JSON.parse(cached);
      if (parsed && Date.now() - parsed.at < maxAge) payload = parsed.release;
    }
  } catch {
    /* ignore malformed cache */
  }

  if (!payload) {
    try {
      const response = await fetch(endpoint, {
        headers: { Accept: "application/vnd.github+json" },
      });
      if (!response.ok) return;
      payload = await response.json();
      try {
        sessionStorage.setItem(cacheKey, JSON.stringify({ at: Date.now(), release: payload }));
      } catch {
        /* storage full or blocked */
      }
    } catch {
      return;
    }
  }

  if (!payload || !payload.tag_name) return;

  document.querySelectorAll("[data-release-tag]").forEach((element) => {
    element.textContent = payload.tag_name;
    element.hidden = false;
  });

  const assets = Array.isArray(payload.assets) ? payload.assets : [];
  const findAsset = (pattern) =>
    assets.find((asset) => asset && asset.state === "uploaded" && pattern.test(asset.name));

  const installer = findAsset(/setup\.exe$/i);
  const portable = findAsset(/portable\.zip$/i);

  applyAsset(installer, {
    link: "[data-installer-link]",
    name: "[data-installer-name]",
    size: "[data-installer-size]",
    fallbackName: "Windows x64 · .exe",
  });

  applyAsset(portable, {
    link: "[data-portable-link]",
    name: "[data-portable-name]",
    size: "[data-portable-size]",
    fallbackName: "Windows x64 · .zip",
  });
}

function applyAsset(asset, selectors) {
  if (!asset) return;

  const link = document.querySelector(selectors.link);
  const name = document.querySelector(selectors.name);
  const size = document.querySelector(selectors.size);

  if (link && asset.browser_download_url) {
    link.href = asset.browser_download_url;
    link.setAttribute("download", "");
  }
  if (name) name.textContent = asset.name || selectors.fallbackName;
  if (size && Number.isFinite(asset.size)) size.textContent = formatBytes(asset.size);
}

function formatBytes(bytes) {
  const mb = bytes / (1024 * 1024);
  if (mb >= 0.1) return `${mb.toFixed(1)} MB`;
  return `${Math.max(1, Math.round(bytes / 1024))} KB`;
}
