import { readFileSync, writeFileSync } from "node:fs";
import sharp from "sharp";

const svg = readFileSync("assets/framelab-mark.svg");
const png = await sharp(svg, { density: 384 }).resize(1024, 1024).png().toBuffer();
writeFileSync("assets/framelab-icon.png", png);
console.log("wrote assets/framelab-icon.png", png.length, "bytes");
