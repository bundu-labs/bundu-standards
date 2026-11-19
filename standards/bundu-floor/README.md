# The Bundu floor

**Version 1.0.0, adopted.** The owner set these values on 2026-11-19. Licence: CC BY 4.0 for the
text, CC0 for [`floor.json`](floor.json), which holds every value below as data.

The Bundu floor is the set of minimums no screen in the ecosystem can go under: how large a
control is to hit, how large text is, how much contrast it has, and how much space sits around
each layer of a screen. It is the seventh of the ecosystem's engineering constraints
(`bundu-floor`).

[Mzizi](https://mzizi.dev) implements it: its token layer applies these values by default, and its
tests, component contracts and `mzizi check` fail a build that goes under one. Mzizi is a Bundu
Foundation project, built by Nyuchi; Nyuchi runs its cyber section. This page is where the floor's
text is published.

## Who the floor serves

- **A market trader in Mbare at midday** reads prices on a cracked phone in full sun. Body text at
  7:1 contrast and 17px is what makes that possible. That is the outdoor-readability constraint.
- **A family sharing one phone**, from a grandmother with less-steady hands to a child, can hit
  every button. The 45px hit area and the 9px gap between controls are why.
- **A developer in Bulawayo or Nairobi** does not need to know the WCAG numbers to ship to them,
  because the design system will not let the build go under the floor.

> Every screen built on the floor is readable in the sun and usable with one thumb, because the
> system will not let it be anything less.

## The number rule

Every length in the floor is a multiple of 3, a multiple of 7, or a prime.

1. Start from the recommended value: WCAG 2.2, Apple's Human Interface Guidelines or Material 3.
2. Take the nearest allowed value at or above it, never below it. So WCAG's 44 becomes 45
   (3 × 15), and Material's 8 becomes 9 (3 × 3).
3. Cards, containers and page layout step on the seven: 7, 14, 21, 28, 49. The one exception is
   the page edge on phones, which takes the general rule: 17, the next allowed value at or above
   16.

## Hit area

| Where                                                    | Floor   | Built from | Recommended | Source                                                    |
| -------------------------------------------------------- | ------- | ---------- | ----------- | --------------------------------------------------------- |
| Touch: a coarse pointer, or any window under 64rem       | 45 × 45 | 3 × 15     | 44          | WCAG 2.2 SC 2.5.5 Target Size (Enhanced), AAA; Apple 44pt |
| Fine pointer: a fine pointer with hover, 64rem and wider | 24 × 24 | 3 × 8      | 24          | WCAG 2.2 SC 2.5.8 Target Size (Minimum), AA               |

The fine-pointer tier is the media query
`(pointer: fine) and (hover: hover) and (min-width: 64rem) and (not (any-pointer: coarse))`. Every
other screen is touch. Dense controls keep the full hit area in both tiers.

## Control heights

| Control height | Floor | Built from |
| -------------- | ----- | ---------- |
| Small          | 33    | 3 × 11     |
| Large          | 41    | prime      |

Both were off the number rule before 2026-11-19 (32 and 40) and take the nearest
allowed value at or above. A small touch target that was 40 is 41 as well.

## Text, contrast and leading

| Requirement                                             | Floor                                 | Standard                                   |
| ------------------------------------------------------- | ------------------------------------- | ------------------------------------------ |
| Body text                                               | 17px on touch, 15px on a fine pointer | Apple's default body size is 17pt          |
| Captions and secondary labels                           | 14px                                  |                                            |
| Body text contrast                                      | 7:1                                   | WCAG 2.2 SC 1.4.6 Contrast (Enhanced), AAA |
| Text that is not body text                              | 4.5:1                                 | WCAG 2.2 SC 1.4.3 Contrast (Minimum), AA   |
| Interface parts (borders, icons, focus, control states) | 3:1                                   | WCAG 2.2 SC 1.4.11 Non-text Contrast, AA   |
| Body leading                                            | 1.5, as the default                   | WCAG 2.2 SC 1.4.12 Text Spacing, AA        |

## Spacing, layer by layer

Each layer of a screen has its own spacing, and each has two values. The **default** value is the
floor at default density. The **dense** value is the absolute floor: nothing goes under it. A
component may go larger than either, never smaller, and no brand pack or density setting can lower
one.

Use the default for reading, forms, onboarding, payments, and anything someone does once and must
get right. Use dense for tables, data views, toolbars, filter rows, status pills, and lists people
scan.

| Layer      | Value                             | Default | Dense | Recommended (default / dense) | Source of the recommendation                      |
| ---------- | --------------------------------- | ------- | ----- | ----------------------------- | ------------------------------------------------- |
| Pills      | Side padding                      | 9       | 5     | 8 / 4                         | A compact status pill; Material 3 large badge 4dp |
| Pills      | Dot or icon to label              | 5       | 3     | 4 / 2                         |                                                   |
| Chips      | Side padding                      | 12      | 9     | 12 / 8                        | Material 3 chip (8dp compact)                     |
| Controls   | Button side padding               | 17      | 12    | 16 / 12                       | Material 3 button (12dp dense)                    |
| Controls   | Field side padding                | 17      | 12    | 16 / 12                       | Material 3 text field                             |
| Controls   | Icon to label                     | 9       | 7     | 8 / 6                         | Material 3 icon to label                          |
| Controls   | Between controls                  | 9       | 9     | 8 / 8                         | Material 3 spacing between targets                |
| Cards      | Padding                           | 21      | 14    | 16 / 12                       | Material 3 card (12dp list item)                  |
| Cards      | Title to body                     | 7       | 7     | 4 / 4                         | The Material 3 4dp step                           |
| Cards      | Between cards                     | 14      | 7     | 8 / 4                         | Material 3 space between cards                    |
| Containers | Padding (dialogs, sheets, panels) | 28      | 21    | 24 / 16                       | Material 3 dialog (16dp sheet)                    |
| Containers | Gap between their parts           | 21      | 14    | 16 / 12                       | Material 3 dialog content spacing                 |
| Page       | Edge on phones                    | 17      | 17    | 16                            | Apple layout margin 16pt; Material 3 compact 16dp |
| Page       | Edge on wide screens              | 28      | 21    | 24 / 16 to 24                 | Material 3 medium and expanded windows, 24dp      |
| Page       | Between sections                  | 49      | 28    | 48 / 24                       | The common 48px section rhythm                    |

The phone page edge has no dense step. Controls, cards, containers and pages are nested: a container
holds cards, and a card holds controls, so each layer steps up from the one inside it.

## Floor against floor

How the Bundu floor compares with WCAG 2.2, Apple's Human Interface Guidelines, Material 3 and
Anthropic's brand guidelines, as read on 2026-11-19. The Bundu floor enforces each value; the
others mostly recommend.

| Requirement         | Bundu floor                                  | WCAG 2.2                                 | Apple HIG                               | Material 3                        | Anthropic brand                             |
| ------------------- | -------------------------------------------- | ---------------------------------------- | --------------------------------------- | --------------------------------- | ------------------------------------------- |
| Touch target        | 45 × 45px, enforced                          | 44 × 44px (2.5.5, AAA)                   | 44 × 44pt, recommended                  | 48 × 48dp, recommended            | Not set: a visual identity, not a UI system |
| Fine-pointer target | 24 × 24px, enforced                          | 24 × 24px, or enough spacing (2.5.8, AA) | 28pt macOS control height, a convention | Not separate: 48dp everywhere     | Not set                                     |
| Body text           | 17px touch, 15px fine pointer; captions 14px | No size; text resizes to 200% (1.4.4)    | 17pt default body; captions 12 and 11pt | Body Large 16sp, Body Medium 14sp | Lora body, Poppins headings                 |
| Body contrast       | 7:1, enforced                                | 4.5:1 at AA (1.4.3), 7:1 at AAA (1.4.6)  | 4.5:1 for custom colours                | 4.5:1; 3:1 for large text         | Not a rule                                  |
| Interface parts     | 3:1, enforced                                | 3:1 (1.4.11, AA)                         | Covered by system materials             | 3:1                               | Not set                                     |
| Line height         | 1.5 body, the default                        | Content survives 1.5 (1.4.12, AA)        | About 1.29 (17pt on 22pt)               | 1.5 (Body Large, 16 on 24sp)      | Not set                                     |
| Spacing             | A floor per layer, default and dense         | Only through 2.5.8 spacing and 1.4.12    | Layout margins and safe areas           | 8dp grid, 4dp for small parts     | Not set                                     |
| Who can lower it    | No one                                       | A page meets the level or it does not    | Each app (guidelines, App Review)       | Each app (guidelines)             | Not set                                     |

### Anthropic's palette against the floor

Anthropic's guidelines name colours and typefaces but set no accessibility rules. Each colour,
measured with the WCAG 2 contrast formula against the brand's own Light (`#faf9f5`) and Dark
(`#141413`) grounds:

| Colour                  | On Light | On Dark |
| ----------------------- | -------- | ------- |
| Orange `#d97757`        | 2.96:1   | 5.90:1  |
| Blue `#6a9bcc`          | 2.78:1   | 6.30:1  |
| Green `#788c5d`         | 3.49:1   | 5.01:1  |
| Mid Gray `#b0aea5`      | 2.11:1   | 8.29:1  |
| Dark `#141413` on Light | 17.50:1  | n/a     |

On Light, no accent is legible as text; on Dark, the accents reach 4.5:1 but not the 7:1 body
floor. That fits how Anthropic uses the palette: accents for shapes, running text in Dark on Light.
The Bundu floor makes the same split explicit and testable.

### Where the floor stands

- **Ahead.** 7:1 body contrast is WCAG AAA, where Apple and Material ask for 4.5:1. The floor is
  enforced, not advised. Each layer has its own spacing minimum, which none of the others sets. And
  1.5 leading is the default, where WCAG only requires that content survive it.
- **In line.** 45px on touch and 24px on a fine pointer meet WCAG 2.5.5 and 2.5.8 and Apple's 44pt.
  Material's 48dp stays 3px larger, and an Android-only product may still choose 48.
- **Worth watching.** Apple's Dynamic Type and Android's font scaling grow text with the person's
  setting, so the px floors must respect the system text scale. And Material and most tools count
  in 4 and 8, so designers coming from those kits will see 9 where they expect 8, 17 for 16, 21 for
  16 on cards and 28 for 24.

## What changed on 2026-11-19

- Hit area: 49 to **45** on touch, and 28 to **24** on a fine pointer.
- Control heights: 32 to **33** and 40 to **41**, onto the number rule: the nearest allowed value at or above each.
- The gap between controls: 7 to **9**, the next allowed value above Material's 8.
- The single 14px inset floor became a **floor per layer, each with a dense step**.
- Unchanged: body text 17px and 15px, captions 14px, contrast 7:1, 4.5:1 and 3:1, and body
  leading 1.5.

## Sources

- WCAG 2.2: <https://www.w3.org/TR/WCAG22/>, success criteria 1.4.3, 1.4.4, 1.4.6, 1.4.11, 1.4.12,
  2.5.5 and 2.5.8 (register `wcag`).
- Apple: [Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines).
- Google: [Material 3](https://m3.material.io/), touch targets, type scale and component specs.
- Anthropic: brand guidelines, colours and typefaces. Contrast ratios computed with the WCAG 2
  relative-luminance formula.
- The owner's canonical values, 2026-11-19.
