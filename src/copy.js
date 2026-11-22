// Copy this page as Markdown or JSON. Shared, byte for byte, by standards.bundu.org
// (workers/standards-api/public/copy.js) and vocab.bundu.org (src/copy.js); a test keeps the
// two copies identical.
//
// The control is in the page with `hidden`. Without this script it stays hidden, so a
// reader never meets a button that does nothing. Markdown is built from the page's own
// <main>; JSON is the page's data, from <script id="page-json">.
(() => {
  const root = document.querySelector("[data-copy]");
  const main = document.querySelector("main");
  if (!root || !main) return;
  const select = root.querySelector("select");
  const button = root.querySelector("button");
  const status = root.querySelector("[role=status]");
  if (!select || !button || !status) return;

  const BLOCK = new Set([
    "P",
    "UL",
    "OL",
    "DL",
    "TABLE",
    "PRE",
    "DIV",
    "SECTION",
    "NAV",
    "H1",
    "H2",
    "H3",
    "H4",
    "H5",
    "H6",
  ]);

  const skip = (el) =>
    el.hidden ||
    el.getAttribute("aria-hidden") === "true" ||
    el.hasAttribute("data-md-skip") ||
    el.tagName === "SCRIPT" ||
    el.tagName === "STYLE";

  const tidy = (s) =>
    s
      .replace(/[ \t]+/g, " ")
      .replace(/ *\n */g, "\n")
      .trim();

  // Inline content: text, links, code, emphasis.
  const inline = (node) => {
    let out = "";
    for (const n of node.childNodes) {
      if (n.nodeType === Node.TEXT_NODE) {
        out += n.nodeValue.replace(/\s+/g, " ");
        continue;
      }
      if (n.nodeType !== Node.ELEMENT_NODE || skip(n)) continue;
      if (n.dataset.md !== undefined) {
        out += n.dataset.md;
        continue;
      }
      const tag = n.tagName;
      if (tag === "BR") out += "\n";
      else if (tag === "CODE" || tag === "KBD")
        out += "`" + n.textContent + "`";
      else if (tag === "STRONG" || tag === "B")
        out += "**" + inline(n).trim() + "**";
      else if (tag === "EM" || tag === "I") out += "_" + inline(n).trim() + "_";
      else if (tag === "A") {
        const text = inline(n).trim();
        const href = n.getAttribute("href");
        out +=
          href && !href.startsWith("#")
            ? `[${text}](${new URL(href, location.href).href})`
            : text;
      } else if (BLOCK.has(tag)) out += " " + inline(n) + " ";
      else out += inline(n);
    }
    return out;
  };

  // A list item: its inline parts, with any block parts after an em dash.
  const item = (li) => {
    const parts = [];
    let run = "";
    for (const n of li.childNodes) {
      if (
        n.nodeType === Node.ELEMENT_NODE &&
        BLOCK.has(n.tagName) &&
        !skip(n)
      ) {
        if (tidy(run)) parts.push(tidy(run));
        run = "";
        const t = tidy(inline(n));
        if (t) parts.push(t);
      } else {
        const wrap = document.createElement("span");
        wrap.append(n.cloneNode(true));
        run += inline(wrap);
      }
    }
    if (tidy(run)) parts.push(tidy(run));
    return parts.join(" — ").replace(/\n/g, " ");
  };

  const cell = (c) => tidy(inline(c)).replace(/\|/g, "\\|").replace(/\n/g, " ");

  const table = (t) => {
    const out = [];
    const cap = t.querySelector("caption");
    if (cap && !cap.classList.contains("sr-only"))
      out.push(`**${tidy(inline(cap))}**`, "");
    const rows = [...t.querySelectorAll("tr")].filter((r) => !skip(r));
    const keyed =
      !t.tHead &&
      rows.length &&
      rows.every((r) => r.cells[0] && r.cells[0].tagName === "TH");
    if (keyed) {
      // A two-column table of names and values reads better as a list.
      for (const r of rows)
        out.push(
          `- **${cell(r.cells[0])}:** ${[...r.cells].slice(1).map(cell).join(" ")}`,
        );
      return out.join("\n");
    }
    rows.forEach((r, i) => {
      out.push("| " + [...r.cells].map(cell).join(" | ") + " |");
      if (i === 0) out.push("|" + [...r.cells].map(() => " --- |").join(""));
    });
    return out.join("\n");
  };

  const blocks = (node, out) => {
    for (const n of node.childNodes) {
      if (n.nodeType === Node.TEXT_NODE) {
        const t = tidy(n.nodeValue);
        if (t) out.push(t);
        continue;
      }
      if (n.nodeType !== Node.ELEMENT_NODE || skip(n)) continue;
      const tag = n.tagName;
      if (/^H[1-6]$/.test(tag))
        out.push("#".repeat(Number(tag[1])) + " " + tidy(inline(n)));
      else if (tag === "P" || tag === "CAPTION") out.push(tidy(inline(n)));
      else if (tag === "UL" || tag === "OL") {
        const items = [...n.children].filter(
          (li) => li.tagName === "LI" && !skip(li),
        );
        out.push(
          items
            .map((li, i) => (tag === "OL" ? `${i + 1}. ` : "- ") + item(li))
            .join("\n"),
        );
      } else if (tag === "DL") {
        for (const c of n.children) {
          if (skip(c)) continue;
          if (c.tagName === "DT") out.push(`**${tidy(inline(c))}**`);
          else if (c.tagName === "DD") flow(c, out);
        }
      } else if (tag === "TABLE") out.push(table(n));
      else if (tag === "PRE")
        out.push("```\n" + n.textContent.replace(/\n+$/, "") + "\n```");
      else flow(n, out);
    }
  };

  // An element that may hold paragraphs, or only text.
  const flow = (el, out) => {
    if ([...el.children].some((c) => BLOCK.has(c.tagName))) blocks(el, out);
    else {
      const t = tidy(inline(el));
      if (t) out.push(t);
    }
  };

  const markdown = () => {
    const out = [];
    blocks(main, out);
    const canonical = document.querySelector('link[rel="canonical"]');
    out.push(`Source: ${canonical ? canonical.href : location.href}`);
    return out.filter(Boolean).join("\n\n") + "\n";
  };

  const json = () => {
    const el = document.getElementById("page-json");
    return el
      ? JSON.stringify(JSON.parse(el.textContent), null, 2) + "\n"
      : null;
  };

  const write = async (text) => {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Older browsers, or a page not served over HTTPS.
      const area = document.createElement("textarea");
      area.value = text;
      area.setAttribute("readonly", "");
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.append(area);
      area.select();
      let ok = false;
      try {
        ok = document.execCommand("copy");
      } catch {
        ok = false;
      }
      area.remove();
      return ok;
    }
  };

  let timer;
  button.addEventListener("click", async () => {
    // Clear first, so copying twice is announced twice.
    status.textContent = "";
    const format = select.value;
    const label = select.options[select.selectedIndex].text;
    const text = format === "json" ? json() : markdown();
    const ok = text !== null && (await write(text));
    status.textContent = ok
      ? `Copied as ${label}.`
      : `Could not copy. Your browser blocked the clipboard.`;
    clearTimeout(timer);
    timer = setTimeout(() => (status.textContent = ""), 5000);
  });

  // Offer JSON only where the page has data.
  if (!document.getElementById("page-json")) {
    for (const o of [...select.options]) if (o.value === "json") o.remove();
  }
  root.hidden = false;
})();
