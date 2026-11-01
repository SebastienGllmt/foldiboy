# Foldiboy tracking site: design system

Read this before changing anything under `web/`.
It says what the site looks like, why, which code owns each part, and how to add things without breaking the whole.
The `#/guide` route draws every token and component with the same code the pages use; keep it open while you work.
It is deliberately not linked from the site: it is a tool for whoever changes the design, not part of reading the tracking data, so open it by URL (`http://127.0.0.1:4173/#/guide`).

## The idea

The browser window is a foldable handheld game console in the spirit of the late 1980s, opened flat on a desk (hence FOLDIBOY).
The left half carries the hardware and a small second screen; a hinge joins it to the right half, which is the main screen.
The main screen shows the page; the second screen shows the menu and an info panel that reads out whatever the cursor is on.
Both screens are an LCD in one of two schemes (paper by default, or the green colour LCD; SELECT switches), with in-game windows, a pixel font and pixel sprites.
The desk shows around the device, and a Nightstream cartridge (named for the proof system) sits in the top of the control half.

The design is inspired by that era of hardware but is not branded.
No console maker's names, logos, wordmarks, seals or artwork appear anywhere; the product name is FOLDIBOY.

Three qualities decide every choice, in this order:

1. **Usable**: dense tracking data must be quick to read and navigate; decoration never costs content space.
2. **Intuitive**: every element behaves the way its appearance suggests (a window holds content, a button does something, a status shows its tone as a shape).
3. **Thematic**: the handheld is evoked by the shell and the screen's visual language, not by props scattered through the pages.

## File map

| Path | Owns | Changed when |
| --- | --- | --- |
| `web/index.html` | The shell markup: control half (second screen, controls), hinge, main screen `#screen` | Almost never; see "The shell" |
| `web/styles/tokens.css` | Every colour, size, space and font | A new token is needed |
| `web/styles/shell.css` | The two folds, the halves, hinge, hardware parts, bezels, title bar, menu, info panel | The shell changes |
| `web/styles/components.css` | Every block that can appear on the screen | A component is added or changed |
| `web/ui/dom.ts` | `h()`, `s()`, `join()`, `countBy()` | Rarely |
| `web/ui/sprites.ts` | 9×9 pixel bitmaps: tones, shapes, marks | A tone, shape or mark is added |
| `web/ui/status.ts` | Status → tone → sprite + label | A status source is added |
| `web/ui/components.ts` | Builders for pages, windows, tables, fields, numbers, prose | A component is added or changed |
| `web/ui/matrix.ts` | The slot matrix (grids, legend, cursor; the readout goes to the info screen) | Grids change |
| `web/ui/info.ts` | The info screen: page description, row readout, `showInfo()` for builders | The readout changes |
| `web/ui/charts.ts` | The chart kinds | A chart kind is added |
| `web/shell.ts` | Buttons, keys, row cursor, the deck's page list, LIVE light | The control contract changes |
| `web/app.ts` | Fetching the model, routing, rendering the page, title bar, menu, deck line and page description | Routing changes |
| `web/views/routes.ts` | The route table: menu, D-pad order, router, each page's `about` sentence | A page is added |
| `web/views/*.ts` | One module per page, composed from `ui/` builders | Content changes |
| `web/views/guide.ts` | The living catalogue at `#/guide` | Any token, sprite or component changes |
| `web/fonts/` | DotGothic16 (OFL) and its licence | A font is replaced |

Dependencies point one way: `views/` use `ui/`; `ui/` never imports from `views/`; nothing in `ui/` knows about the tracking model except `status.ts`, which reads vocabulary terms.

## Rules

These hold everywhere; a change that breaks one is wrong even if it looks fine.

1. **Views compose builders.** A view never writes component class names, inline styles or SVG by hand; it calls `page`, `win`, `table`, `status` and the rest. If no builder fits, add or extend one (see "New requirements").
2. **Tokens only.** No raw colour, font or size outside `tokens.css`. The only inline styles are data-driven widths in `gauge` and `split`, and the token swatches in the guide.
3. **Status is never colour alone.** Every status is a tone sprite (each tone has its own silhouette) plus a text label, through `status()` or `statusOf()`.
4. **Tones mean status.** The five `--tone-*` colours are only used for statuses. Categories that are not statuses (chart series) use `--series-*` and a marker shape.
5. **Everything sits in a window.** Content blocks are `win()`; messages about the data are `dialogue()`. Nothing floats loose on the screen except the page title.
6. **The shell is fixed.** Pages render inside `#screen` only. The control half, hinge, bezels, title bar, menu and info screen are not page real estate; builders reach the info screen only through `ui/info.ts`.
7. **Real or flagged.** Every number is computed from the tracking model. Anything invented (samples, mock series) is in a window flagged `SAMPLE DATA` or `MOCK DATA`, and never ships on a page that is not the guide unless Sebastien asks for it.
8. **Missing is visible.** Missing fields render as "not specified" (`fields()` does it), empty lists as an `empty()` sentence, unknown statuses as neutral. Never hide a gap.
9. **No new fonts, no branding.** One screen face (DotGothic16), the system face for case print and prose, the system monospace for code.
10. **The guide stays complete.** A new or changed token, sprite, component or chart kind updates `views/guide.ts` in the same change.

## The shell

The device is a foldable opened flat, and it opens one of two ways depending on the window's width (the breakpoint, 720px, is in `shell.css` and `shell.ts`):

- **Book fold** (720px wide and up): control half | hinge | main screen.
  The control half is `clamp(--ctl-min, 21vw, --ctl-max)` wide (232 to 340px); it is a size container, and its D-pad, A/B and wordmark scale with it (`cqw`), so the fold stays visible at every desktop size.
  Top to bottom: wordmark and LIVE light, the second screen (menu, then info), D-pad and A/B, SELECT/START and speaker.
  In a window shorter than 560px (a phone held sideways) the half drops the wordmark and SELECT/START and keeps the screen and the D-pad and A/B.
- **Flip fold** (narrower, phones): main screen on top, the hinge across, and the control half as a short deck below: D-pad | second screen | A/B, then LIVE light, wordmark, SELECT/START.
  The second screen's top line is the deck line, ◀ page ▶; the page name (or START) swaps the info for the page list.

Both folds use the same markup; `shell.css` rearranges it (grid areas and `display: contents`), so no view or script depends on which fold is showing, except the two buttons whose meaning changes (see "Controls").

The parts:

- **Desk margin**: `--margin-desk` of desk around the device (more on top, for the cartridge), so the device reads as an object in the window rather than its border. Short windows and the flip fold keep only a thin margin.
- **Cartridge**: a fixed "Nightstream" cartridge pushed into the top edge of the control half (book fold only). It names the proof system the site tracks; it never changes with the page, which the title bar and the menu already name.
- **Hinge**: a barrel with two knuckles of case plastic, vertical in the book fold and horizontal in the flip fold. It is what makes the device a foldable; never remove it.
- **Bezels**: each screen sits in a bezel with a striped caption strip and a rounded lower-right corner, and a faint pixel-grid overlay on the glass.
- **Title bar**: the first line of the main screen, sticky. The router moves the page's title row (`page()`'s header) into it and adds the load time.
- **Menu**: the menu routes with counts, on the second screen. The current page has the blinking ▶ cursor.
- **Info screen**: under the menu. It reads out what the cursor is on: the page's `about` sentence and a key hint when nothing is selected, the whole selected table row (every column, including those a narrow main screen hides), or the slot under the matrix cursor. See `ui/info.ts`.
- **LIVE light**: on while the server's event stream is connected, a brief flash when a tracking file changes and the screen re-renders, dark when disconnected.

Do not add parts to the case, change the cartridge label, or add text to the bezel captions.
If a new global affordance is needed (a search, a filter), it goes on the second screen or on a page, never on the case; talk to Sebastien first.

## Tokens

All in `web/styles/tokens.css`, in groups:

| Group | Tokens | Use |
| --- | --- | --- |
| Case | `--case*`, `--bezel*`, `--stripe-*`, `--button-*`, `--led-*`, `--hinge-*`, `--desk*`, `--cart*` | The shell only |
| Screen | `--lcd`, `--win`, `--ink`, `--ink-2`, `--ink-3`, `--rule`, `--link`, `--sel*` | Everything drawn on the screen |
| Tones | `--tone-good`, `--tone-warning`, `--tone-serious`, `--tone-critical`, `--tone-neutral`, `--on-tone` | Statuses only |
| Series | `--series-1` … `--series-4` | Chart series and other non-status categories |
| Type | `--font-screen`, `--font-case`, `--font-code`, `--text-title` 32, `--text-heading` 20, `--text-body` 16, `--text-small` 14 | |
| Space | `--s1` 4 … `--s6` 32 | Gaps and padding |
| Shape | `--frame` 2px, `--radius-screen`, `--radius-bezel`, `--radius-half`, `--ctl-min`, `--ctl-max`, `--hinge`, `--margin-desk`, `--cell` | |

Both screens share one scheme, and there are two: **paper** (the default: `#f3efe4` screen, near-white windows, ink cursor) and **green** (the colour LCD: green screen, cursor in the dark shade of the classic four-shade green LCD). SELECT switches between them and the choice is kept in the browser's local storage.
A scheme sets exactly the screen tokens (`--lcd` … `--lcd-dots`) and `--tone-neutral`; paper is `:root`, green is `:root[data-screen="green"]` in `tokens.css`. Anything drawn on a screen must read in both, so use tokens and check both schemes. There is no dark mode.
Both were chosen over grey-on-grey for text contrast. The cursor's fill (`--sel`, the selected row and the current page in the menu) takes its colour from the scheme's own world, with light text (`--sel-ink`): the ink on paper, `#306230` (a shade the original green LCD could show) on green. Never a hue from outside the scheme, such as the link blue; a hovered row only tints unselected rows.
Anything in a cell that brings its own fill (`code`, quiet tags) must follow the row onto `--sel`, and sprites in a selected row sit on a small `--sel-ink` tile so every tone and shape keeps its own colours on the cursor's fill.
Add a token only when a value is needed in two places or carries a meaning; give it a name that says what it is for, not what it looks like.

## Type

DotGothic16 is drawn on a 16px grid, so 16px (body) and 32px (titles, counters) are the crisp sizes; 20px and 14px are allowed for headings and secondary lines.
Rendered Markdown (`prose()`) uses the system face at 15px, because long paragraphs read better there.
Code, file names, revisions and evidence levels use `<code>` (system monospace).
Window titles are short noun phrases and render in capitals; page titles are sentence case.

## Status

Every status in the model has a tone, and every tone has a sprite with its own silhouette:

| Tone | Sprite | Meaning |
| --- | --- | --- |
| good | disc with a tick | done, passing, accepted |
| warning | triangle with "!" | partial, in progress, open |
| serious | diamond with "!" | present but not what is wanted |
| critical | disc with a cross | failing, missing, unclassified |
| neutral | hollow ring with "?" | not run, not decided, unknown |

Where tones come from:

- Vocabulary statuses (`capability_status`, `ticket_status`, `coverage_status`, `decision`) carry `tone` in `vocabulary.toml`; draw them with `statusOf(m.vocabulary.<list>, id)`. Adding a term to the TOML is enough; the site picks up its tone and meaning.
- Statuses that live in code (problem severity, gate verdict) are mapped once in `ui/status.ts` (`SEVERITY_TONE`, `GATE_TONE`).
- An id with no term renders as neutral, so it is visible and obviously unclassified.

Adding a sixth tone is a design change: it needs a token, a sprite with a new silhouette, a slot style, a row in this table and a guide entry. Prefer mapping a new status onto the existing five.

## Components

Each builder is in `web/ui/`; each has a section in `#/guide`.

| Builder | Use for | Not for |
| --- | --- | --- |
| `page({ title, sub, crumb }, …blocks)` | The whole route; detail routes pass `crumb` back to their list | |
| `win(title, body, { aside, flag })` | Every block of content. `aside`: a count or unit. `flag`: synthetic content | Messages to the reader |
| `cols(…blocks)` | Two or three related windows side by side | Long tables, the frontier chart |
| `dialogue(body, tone?)` | A message about the data: not adopted, nothing found | Data |
| `status(tone, label)`, `statusOf(terms, id)` | Any status | Plain labels |
| `tag(text, quiet?)` | A label without a status: a maturity, "baseline" | Statuses |
| `chips(items)`, `statusCounts(…)` | Several statuses or counts by status on one line | |
| `table(columns, rows)` | Lists of records; rows are selectable, read out on the info screen, and can link with `href`. Give rows an `id` (the readout's title) and put the id and status in the first two columns (all a narrow screen shows) | Two-column label/value data |
| `fields(pairs)` | Label / value pairs of one record | Lists of records |
| `list(items, ordered?)` | Short lists inside a field or cell | |
| `counters(items)` | Headline numbers on an overview, each linking to its page; one block per page | Anything else |
| `gauge(label, n, total, tone)` | One real fraction. `tone` is the tone of what is counted, not a verdict on the ratio | Estimates, percentages without a count |
| `split(parts, caption?)` | One total broken down by status | |
| `slotMatrix`, `miniGrid`, `slotLegend` | A universe of 16 × 16 slots | Anything not laid out as a 16 × 16 grid |
| `recordChart`, `frontierChart` | The two chart kinds | Ad-hoc SVG |
| `empty(text)` | An empty block: say what is missing in a sentence | |
| `prose(html)` | Rendered Markdown from a trusted local file (the README) | Anything else; it is the only `innerHTML` |
| `route(…)`, `link(href, text)` | Links between pages (`#/loops/<id>`) | |

Domain helpers built on these live in `views/common.ts` (record links, vocabulary statuses, the file-status notice) and `views/evaluation.ts` (an evaluation set).

## Layout

A page is: title row, then windows top to bottom in order of importance.
Put what the reader came for first: the overview leads with counters and progress, a list page with its records, a detail page with the record's definition.
Use `cols()` for two or three windows that belong together and are each readable at half width (about 380px minimum); everything else is full width.
Long tables are always full width.
A `dialogue()` about the data (such as "not adopted yet") comes right under the title.

The main screen is a size container named `screen`, and components adapt to its width, not the window's (`components.css`, "narrow main screen").
Under 640px, tables keep only their first two columns, `cols()` stack, counters go two across and page padding tightens; under 470px and 400px the matrix cells shrink so a 16 × 16 grid fits a phone.
Design a page so it still reads at 640px with only the first two columns of each table; the info screen supplies the rest of a row.

Detail pages: `#/<route>/<id>` renders the route's `detail` view if it has one, with a crumb back to the list.
Routes without a detail view treat the id as a row to select on the list page (`#/tickets/T-SITE` selects and scrolls to that ticket), so every record has a linkable address.

## Controls

The shell maps buttons and keys; views only mark what can be selected (table rows do this automatically).

| Button | Key | Does |
| --- | --- | --- |
| D-pad ← → | ← → | Previous / next page in menu order; on a slot matrix, moves the slot cursor |
| D-pad ↑ ↓ | ↑ ↓ | Previous / next selectable row, read out on the info screen; on a slot matrix, moves the slot cursor; with no rows, scrolls; with the deck's page list open, moves through it |
| A | X | Opens the selected row's `href`; in the page list, goes to the page |
| B | Z | Back. Flip fold: first closes the page list, then clears the selected row, then goes back |
| START | Enter | Overview. Flip fold: opens or closes the page list |
| SELECT | Shift, pressed and released alone | Switches the screen scheme (paper / green), remembered in this browser |

Mouse and normal keyboard use keep working: links are links, a click selects a row, Enter on a focused link follows it.
A new interaction pattern (for example a second kind of cursor) goes in `shell.ts`, uses the existing buttons, and is added to this table, to the button tooltips in `index.html` and to the comment at the top of `shell.ts`.

## Matrix

`slotMatrix(grids, legend, start)` draws grids of 16 × 16 slots side by side with a legend above; the slot under the cursor is read out on the info screen (its key and status in the title, then the slot's `detail`).
A slot has a tone (fill colour), an optional glyph (critical slots carry ✕ so they read without colour; neutral slots are hatched), and `marked` for structural slots such as the CB prefix, drawn as an outlined box.
The cursor starts on `start` (the opcode page starts on the first failing slot), follows hover and focus, and runs from one grid into the next at the right edge.
Only one interactive matrix should be on a page.

## Charts

Two kinds exist: `recordChart` (one value per run over time, best so far as a step line, new records filled) and `frontierChart` (up to four series where lower-left is better, frontier points filled, dominated points hollow).
The tracking files hold no time series yet, so neither appears on a page; both are in the guide with sample data, ready for the loops that will produce them.
Rules: crisp pixel style (step lines, square-pixel markers, no smoothing or gradients); each series has a colour and a marker shape; every chart has a one-sentence `summary` for screen readers; the frontier chart needs a full-width window.
A new kind of chart is a new builder in `ui/charts.ts` plus a guide section, never SVG in a view.

## New requirements

When the tracking data grows, decide in this order:

1. **Can existing builders show it?** A new record type is usually a `table` on a list page and `fields` on a detail page. A new status list is a vocabulary list drawn with `statusOf`.
2. **Is it a new page?** Write `views/<name>.ts` returning one `page()`, and add one entry to `ROUTES` in `views/routes.ts` (label, `menu`, `count`, `about`, `list`, optional `detail` and `detailAbout`). The menu, deck line, D-pad cycle, router and info screen follow.
3. **Is it a new kind of block?** Add a builder to `ui/components.ts` (or a new `ui/` module for something large), its CSS block to `components.css` using tokens, a guide section, and a row in the Components table above. Make it generic over its data; keep tracking-model knowledge in `views/`.
4. **Is it a new status?** Give it a tone in `vocabulary.toml` if it is vocabulary; otherwise add one mapping in `ui/status.ts`.
5. **Is it a new loop shape?** Add its bitmap to `SHAPE` in `ui/sprites.ts`; until then it shows the neutral ring.
6. **Does it need the shell?** It almost never does. Talk to Sebastien first.

Copy: plain sentences; say what a number counts ("498/512 pass"), not what it implies; name files and ids exactly as in the tracking files.

## Checking a change

1. `bun run typecheck`.
2. `bun run dev` and open the routes you touched, plus `#/guide`.
3. Look at 1440×900, 960×1080 (half a screen), 760×900, 1280×520 (short), and a phone (390×844, the flip fold): nothing clipped, no text overlapping, the hinge visible, tables scroll inside their window rather than the screen.
4. Use the controls on the pages you touched: ↑ ↓ through rows (the info screen reads each out), X on a row with a link, Z back, ← → between pages, Shift to switch to the other screen scheme (check both); on a phone, Enter for the page list.
5. Every status you added shows a sprite and a label.

## History

The design was chosen over eight rounds of proposals with Sebastien; the proposals, his verdicts and the static mocks are in `draft-personal/autoresearch/design-handoff.md` and `draft-personal/autoresearch/design/`.
The first site used round 7's "left rail" layout; round 9 replaced it with round 5's fold-out (the foldable with a second screen), on the same components, with the book/flip folds, the paper and green screen schemes, a desk margin and a fixed Nightstream cartridge.
