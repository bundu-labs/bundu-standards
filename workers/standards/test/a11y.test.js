// Accessibility guards for standards.bundu.org. They read the layout, the pages and the
// stylesheet, so they run before the build, and they fail when a keyboard or screen-reader
// aid goes missing or a colour pair drops under the Bundu floor (7:1 for text).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";

const read = (p) => readFileSync(new URL(p, import.meta.url), "utf8");
const base = read("../src/layouts/Base.astro");
const css = read("../src/styles/site.css");

test("every page starts with a skip link to the main region", () => {
  const skip = base.indexOf(
    '<a class="skip" href="#main">Skip to main content</a>',
  );
  assert.ok(skip > -1, "skip link missing");
  assert.ok(
    skip < base.indexOf("<header"),
    "the skip link must be the first thing in the body",
  );
  assert.match(base, /<main id="main" tabindex="-1">/);
  assert.match(
    css,
    /\.skip:focus\s*\{[^}]*top:/,
    "the skip link must become visible on focus",
  );
});

test("the navigation is labelled and marks the current page", () => {
  assert.match(base, /<nav aria-label="Main">/);
  assert.match(base, /aria-current=\{current\("\/register\/"\)\}/);
  assert.match(base, /<html lang="en">/);
});

test("short link texts carry a full accessible name", () => {
  assert.match(base, />JSON<span class="sr-only">[^<]+<\/span><\/a>/);
  assert.match(base, />Source<span class="sr-only">[^<]+<\/span><\/a>/);
  assert.match(css, /\.sr-only\s*\{[^}]*clip/);
});

test("decorative brand images are hidden from screen readers", () => {
  for (const m of base.matchAll(/<img [^>]*>/g))
    assert.match(m[0], /alt=""/, m[0]);
});

test("status is text, never colour alone", () => {
  for (const page of [
    "../src/pages/register/index.astro",
    "../src/pages/register/[id].astro",
  ]) {
    assert.match(
      read(page),
      /<span class="sr-only">Status: <\/span>\{s\.status\}/,
      page,
    );
  }
});

test("the register has jump links to each group, and each group is a labelled region", () => {
  const reg = read("../src/pages/register/index.astro");
  assert.match(reg, /<nav aria-labelledby="on-this-page">/);
  assert.match(reg, /<section aria-labelledby=\{d\.key\}>/);
});

test("there is an accessibility statement with keyboard help and a way to report a barrier", () => {
  const url = new URL("../src/pages/accessibility.astro", import.meta.url);
  assert.ok(existsSync(url));
  const page = readFileSync(url, "utf8");
  for (const s of [
    "<kbd>Tab</kbd>",
    "<kbd>Shift</kbd>",
    "<kbd>Enter</kbd>",
    "Report a barrier",
    "<caption>",
  ]) {
    assert.ok(page.includes(s), s);
  }
  assert.match(base, /href="\/accessibility\/"/);
});

test("focus is always visible, and long text wraps instead of scrolling sideways", () => {
  assert.match(css, /:focus-visible\s*\{\s*outline:\s*3px solid/);
  assert.match(css, /overflow-wrap:\s*anywhere/);
});

// WCAG relative luminance and contrast.
const lum = (hex) => {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const l = c.map((v) =>
    v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4,
  );
  return 0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2];
};
const contrast = (a, b) => {
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
};
const tokens = (block) =>
  Object.fromEntries(
    [...block.matchAll(/--([a-z-]+):\s*(#[0-9a-fA-F]{6})/g)].map((m) => [
      m[1],
      m[2],
    ]),
  );

test("every text colour pair meets the Bundu floor of 7:1 in both themes", () => {
  const light = tokens(
    css.slice(css.indexOf(":root {"), css.indexOf("}", css.indexOf(":root {"))),
  );
  const darkStart = css.indexOf("@media (prefers-color-scheme: dark)");
  const dark = {
    ...light,
    ...tokens(css.slice(darkStart, css.indexOf("}\n}", darkStart))),
  };
  for (const [name, t] of [
    ["light", light],
    ["dark", dark],
  ]) {
    for (const [fg, bg] of [
      ["fg", "bg"],
      ["muted", "bg"],
      ["accent", "bg"],
      ["fg", "chip"],
      ["muted", "chip"],
      ["on-copper-container", "copper-container"],
    ]) {
      const c = contrast(t[fg], t[bg]);
      assert.ok(c >= 7, `${name}: --${fg} on --${bg} is ${c.toFixed(2)}:1`);
    }
  }
});
