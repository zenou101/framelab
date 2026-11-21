// One-off asset pipeline for the website.
//
// Run from the repository root (it reuses the desktop app's `sharp` dev dependency):
//   node site/scripts/build-assets.mjs
//
// Source captures are development screenshots of FrameLab 0.1.0. Nothing here
// fabricates UI: the four images below are the only product screenshots used by
// the site, and they are copied as captured apart from lossy WebP conversion and
// thumbnail resizing.

import { createRequire } from "node:module";
import { mkdir, copyFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const require = createRequire(pathToFileURL(resolve(process.cwd(), "package.json")));
const sharp = require("sharp");

const root = process.cwd();
const out = resolve(root, "site/public");
const shots = resolve(out, "screenshots");

const captures = [
  { src: ".tmp/window2.png", name: "overview-dark", alt: "Overview, dark theme" },
  { src: ".tmp/settings-test.png", name: "overview-light", alt: "Overview, light theme" },
  { src: ".tmp/light.png", name: "games-light", alt: "Games library, light theme" },
  { src: ".tmp/legal-about.png", name: "legal-light", alt: "Legal and About, light theme" },
];

await mkdir(shots, { recursive: true });

for (const capture of captures) {
  const input = resolve(root, capture.src);
  await sharp(input)
    .webp({ quality: 90, effort: 6 })
    .toFile(resolve(shots, `${capture.name}.webp`));
  await sharp(input)
    .resize({ width: 440, withoutEnlargement: true })
    .webp({ quality: 78, effort: 6 })
    .toFile(resolve(shots, `${capture.name}-thumb.webp`));
  const meta = await sharp(input).metadata();
  console.log(`${capture.name}: ${meta.width}x${meta.height}`);
}

// Mark and favicons ---------------------------------------------------------
await copyFile(resolve(root, "assets/framelab-mark.svg"), resolve(out, "mark.svg"));
await sharp(resolve(root, "assets/framelab-mark.svg"))
  .resize(32, 32)
  .png()
  .toFile(resolve(out, "favicon-32.png"));
await sharp(resolve(root, "assets/framelab-mark.svg"))
  .resize(180, 180)
  .png()
  .toFile(resolve(out, "apple-touch-icon.png"));

// Open Graph card (1200x630) ------------------------------------------------
const W = 1200;
const H = 630;
const shotW = 512;
const shotH = Math.round((shotW * 868) / 1334);

const background = `
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="#08090B"/>
  <g stroke="#FFFFFF" stroke-opacity="0.045">
    ${Array.from({ length: 14 }, (_, i) => `<line x1="0" y1="${i * 48}" x2="${W}" y2="${i * 48}"/>`).join("")}
    ${Array.from({ length: 26 }, (_, i) => `<line x1="${i * 48}" y1="0" x2="${i * 48}" y2="${H}"/>`).join("")}
  </g>
  <text x="64" y="112" fill="#7D848F" font-family="Consolas, monospace" font-size="13" letter-spacing="4">WINDOWS PERFORMANCE MONITOR</text>
  <text x="62" y="208" fill="#F3F4F6" font-family="Segoe UI, Arial, sans-serif" font-size="58" font-weight="600" letter-spacing="-2">See the frames</text>
  <text x="62" y="270" fill="#F3F4F6" font-family="Segoe UI, Arial, sans-serif" font-size="58" font-weight="600" letter-spacing="-2">your FPS counter</text>
  <text x="62" y="332" fill="#6B9DFF" font-family="Segoe UI, Arial, sans-serif" font-size="58" font-weight="600" letter-spacing="-2">hides.</text>
  <text x="64" y="404" fill="#A2A8B1" font-family="Segoe UI, Arial, sans-serif" font-size="19">Frame timings, 1% lows and system load —</text>
  <text x="64" y="432" fill="#A2A8B1" font-family="Segoe UI, Arial, sans-serif" font-size="19">recorded locally on Windows.</text>
  <text x="64" y="566" fill="#7D848F" font-family="Consolas, monospace" font-size="13" letter-spacing="1">LOCAL-FIRST · PRESENTMON/ETW · MIT LICENCE</text>
  <rect x="${W - shotW - 72}" y="${Math.round(H / 2 - shotH / 2 - 14)}" width="${shotW + 12}" height="${shotH + 12}" fill="#0C0E11" stroke="#262A31"/>
</svg>`;

const frame = await sharp(Buffer.from(background)).png().toBuffer();
const shot = await sharp(resolve(shots, "overview-dark.webp"))
  .resize(shotW, shotH, { fit: "cover" })
  .png()
  .toBuffer();
const mark = await sharp(resolve(root, "assets/framelab-mark.svg"))
  .resize(44, 44)
  .png()
  .toBuffer();

await sharp(frame)
  .composite([
    { input: mark, left: 64, top: 56 },
    {
      input: Buffer.from(
        `<svg xmlns="http://www.w3.org/2000/svg" width="180" height="44"><text x="56" y="33" fill="#F3F4F6" font-family="Segoe UI, Arial, sans-serif" font-size="27" font-weight="600" letter-spacing="-0.8">FrameLab</text></svg>`,
      ),
      left: 64,
      top: 50,
    },
    {
      input: shot,
      left: W - shotW - 66,
      top: Math.round(H / 2 - shotH / 2 - 8),
    },
  ])
  .png({ compressionLevel: 9 })
  .toFile(resolve(out, "og.png"));

console.log("wrote screenshots, icons and og.png");
