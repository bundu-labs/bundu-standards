# bundu-site

The page shell, stylesheet and accessibility audit that standards.bundu.org and
vocab.bundu.org share, so the two sites look and behave the same. No Tailwind, no
JavaScript runtime: Dioxus components rendered on the server (Mzizi Roots), one
`styles/site.css` served by both at `/styles/site.css`.

- `Document` / `render` / `render_html`: head, skip link, header, `main` with the copy
  control, footer.
- `audit::audit(html)`: the rules both sites keep on every rendered page (`lang`, skip link
  first, one `main`, one banner, one footer, labelled navigation, one `h1` and no skipped
  heading level, names for links, buttons and fields, `alt` on images, unique ids and
  resolvable references, header cells with `scope`, keyboard help in the footer of every page
  (Tab, Shift+Tab, Enter, a link to the accessibility statement, no `accesskey`), no zoom lock, no inline style, handler or
  script, because the CSP allows none).
- `css_audit` (feature `audit`, for tests): parses the stylesheet with `lightningcss`. It lints
  `styles/page.css` as written (a literal colour, length, time or bare number fails; page CSS
  declares no token), and holds the bundle's resolved tokens, in both themes and both pointer
  tiers, to the Bundu floor: APCA Lc 75 for text (30 for a control's edge and the focus ring) and
  WCAG 7:1 / 3:1, the hit region (45px touch from `mzizi-tokens`, 24px fine pointer), 9px between
  controls, 17px page edge, 17/15px body, 14px captions, 1.5 leading, a visible `:focus-visible`
  ring, forced colours, reduced motion, wrapping, `.sr-only`.

## Styles: tokens only

Every value in `styles/page.css` is `var(--token)`. The token layer is `tokens/tokens.css`, the
Mzizi tokens (`styles/pattern/tokens.css` of mzizi-dev/mzizi-registry), vendored by
`tools/vendor-tokens.sh` at the commit pinned in `tokens/PIN`; CI runs `--check` and fails on
drift, and `--update` re-pins to the head of main. A value Mzizi has no token for is proposed upstream (the nine chrome tokens
were, and are in the pinned file), never typed into page CSS.

`build.rs` bundles page CSS and the token file with Lightning CSS (`lightningcss`
`=1.0.0-alpha.72`, `default-features = false`: the crate, version, features and browser targets of mzizi-registry's
`mzizi-css`: Chrome and Edge 111, Firefox 113, Safari 16.2), drops every custom property no rule
uses, and minifies. Rust only, no JavaScript. The sites serve the result at `/styles/site.css`
with an `ETag` (a hash of the bytes).

Each site runs both audits over every page it renders in `cargo test`.

## To go upstream (mzizi-registry)

Each is a Rust Dioxus component with its contract in the module docs. None is published or
pushed from here.

| Module                       | What it is                                                                 |
| ---------------------------- | -------------------------------------------------------------------------- |
| `components/skip_link.rs`    | `SkipLink`: first focusable element, targets `#main`                       |
| `components/site_header.rs`  | `SiteHeader` and `NavLink`: brand, labelled main nav, `aria-current`       |
| `components/copy_control.rs` | `CopyControl`: "Copy this page as" Markdown or JSON, hidden until script   |
| `lib.rs` `Document`          | the page frame; the registry has `discover-shell` for Astro, no Rust shell |
| `css_audit.rs` `apca`        | APCA-W3 0.1.9, for `mzizi-tokens`                                          |

The status badge is `mzizi-roots`' `Badge`; no other primitive is used.
