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
  resolvable references, header cells with `scope`, no zoom lock, no inline style, handler or
  script, because the CSP allows none).
- `css_audit::audit(css)`: the stylesheet's own tokens, in both themes: APCA Lc 75 for text
  (60 for control edges and the focus ring) and WCAG 7:1 / 3:1, the Bundu floor (45px / 24px
  hit regions, 17px / 15px body, 14px captions, 17px inset, 9px gaps, 1.5 leading), a visible
  3px `:focus-visible`, forced colours, reduced motion, wrapping, `.sr-only`, no text below the
  caption floor. `apca` implements APCA-W3 0.1.9; `mzizi-tokens` has no function for it.

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

The status badge is `mzizi-ui`'s `Badge`; no other primitive is used.
