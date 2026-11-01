# Foldiboy tracking site: design system

Read this before changing anything under `web/`.
It says what the site looks like, why, which code owns each part, and how to add things without breaking the whole.
The `#/guide` route draws every token and component with the same code the pages use; keep it open while you work.

## The idea

The browser window is a handheld game console of the late 1980s, reduced to its essentials.
The case shrinks to a narrow rail on the left that carries every hardware part; the rest of the window is the screen, and the pages are what the screen shows.
The screen is a pocket grey LCD drawn in full colour, with in-game windows, a pixel font and pixel sprites.

The design is inspired by that era of hardware but is not branded.
No console maker's names, logos, wordmarks, seals or artwork appear anywhere; the product name is FOLDIBOY.

Three qualities decide every choice, in this order:

1. **Usable**: dense tracking data must be quick to read and navigate; decoration never costs content space.
2. **Intuitive**: every element behaves the way its appearance suggests (a window holds content, a button does something, a status shows its tone as a shape).
3. **Thematic**: the handheld is evoked by the shell and the screen's visual language, not by props scattered through the pages.

## File map

| Path | Owns | Changed when |
| --- | --- | --- |
| `web/index.html` | The shell markup: rail, bezel, `#screen` | Almost never; see "The shell" |
| `web/styles/tokens.css` | Every colour, size, space and font | A new token is needed |
| `web/styles/shell.css` | Rail, hardware parts, bezel, screen glass, menu bar | The shell changes |
| `web/styles/components.css` | Every block that can appear on the screen | A component is added or changed |
| `web/ui/dom.ts` | `h()`, `s()`, `join()`, `countBy()` | Rarely |
| `web/ui/sprites.ts` | 9×9 pixel bitmaps: tones, shapes, marks | A tone, shape or mark is added |
| `web/ui/status.ts` | Status → tone → sprite + label | A status source is added |
| `web/ui/components.ts` | Builders for pages, windows, tables, fields, numbers, prose | A component is added or changed |
| `web/ui/matrix.ts` | The slot matrix (grids, legend, readout, cursor) | Grids change |
| `web/ui/charts.ts` | The chart kinds | A chart kind is added |
| `web/shell.ts` | Buttons, keys, row cursor, LIVE light | The control contract changes |
| `web/app.ts` | Fetching the model, routing, rendering into `#screen` | Routing changes |
| `web/views/routes.ts` | The route table: menu, D-pad order, router | A page is added |
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
6. **The shell is fixed.** Pages render inside `#screen` only. The rail, bezel and menu bar are not page real estate.
7. **Real or flagged.** Every number is computed from the tracking model. Anything invented (samples, mock series) is in a window flagged `SAMPLE DATA` or `MOCK DATA`, and never ships on a page that is not the guide unless Sebastien asks for it.
8. **Missing is visible.** Missing fields render as "not specified" (`fields()` does it), empty lists as an `empty()` sentence, unknown statuses as neutral. Never hide a gap.
9. **No new fonts, no branding.** One screen face (DotGothic16), the system face for case print and prose, the system monospace for code.
10. **The guide stays complete.** A new or changed token, sprite, component or chart kind updates `views/guide.ts` in the same change.

## The shell

The rail, top to bottom: LIVE light, the FOLDIBOY wordmark printed up the rail, a spacer, D-pad, A/B, SELECT/START, speaker holes.
The bezel has a striped caption strip on top and the rounded lower-right corner that the case has; inside it, the screen glass with a faint pixel-grid overlay.
Below 700px wide the rail becomes a strip along the bottom, like the lower half of the device.

The menu bar is the first line of the screen and belongs to the shell: it lists the menu routes with counts, a one-line key hint, the style guide link and the load time.

The LIVE light reports the server connection: on while the event stream is connected, a brief flash when a tracking file changes and the screen re-renders, dark when disconnected.

Do not add parts to the rail or text to the bezel caption.
If a new global affordance is needed (a search, a filter), it goes in the menu bar's end slot or on a page, never on the case.

## Tokens

All in `web/styles/tokens.css`, in groups:

| Group | Tokens | Use |
| --- | --- | --- |
| Case | `--case*`, `--bezel*`, `--stripe-*`, `--button-*`, `--led-*` | The shell only |
| Screen | `--lcd`, `--win`, `--ink`, `--ink-2`, `--ink-3`, `--rule`, `--link`, `--sel*` | Everything drawn on the screen |
| Tones | `--tone-good`, `--tone-warning`, `--tone-serious`, `--tone-critical`, `--tone-neutral`, `--on-tone` | Statuses only |
| Series | `--series-1` … `--series-4` | Chart series and other non-status categories |
| Type | `--font-screen`, `--font-case`, `--font-code`, `--text-title` 32, `--text-heading` 20, `--text-body` 16, `--text-small` 14 | |
| Space | `--s1` 4 … `--s6` 32 | Gaps and padding |
| Shape | `--frame` 2px, `--radius-screen`, `--radius-bezel`, `--rail`, `--cell` | |

The screen palette is the single "pocket grey" palette; there is no dark mode and no palette switch.
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
| `table(columns, rows)` | Lists of records; rows are selectable and can link with `href` | Two-column label/value data |
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

Detail pages: `#/<route>/<id>` renders the route's `detail` view if it has one, with a crumb back to the list.
Routes without a detail view treat the id as a row to select on the list page (`#/tickets/T-SITE` selects and scrolls to that ticket), so every record has a linkable address.

## Controls

The shell maps buttons and keys; views only mark what can be selected (table rows do this automatically).

| Button | Key | Does |
| --- | --- | --- |
| D-pad ← → | ← → | Previous / next page in menu order; on a slot matrix, moves the slot cursor |
| D-pad ↑ ↓ | ↑ ↓ | Previous / next selectable row; on a slot matrix, moves the slot cursor; with no rows, scrolls |
| A | X | Opens the selected row's `href` |
| B | Z | Back |
| START | Enter | Overview |
| SELECT | Shift, pressed and released alone | Scrolls to the next window on the page |

Mouse and normal keyboard use keep working: links are links, a click selects a row, Enter on a focused link follows it.
A new interaction pattern (for example a second kind of cursor) goes in `shell.ts`, uses the existing buttons, and is added to this table, to the key hint in `app.ts` and to the button tooltips in `index.html`.

## Matrix

`slotMatrix(grids, legend, start)` draws grids of 16 × 16 slots side by side, a legend above, and a readout window below that shows the slot under the cursor.
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
2. **Is it a new page?** Write `views/<name>.ts` returning one `page()`, and add one entry to `ROUTES` in `views/routes.ts` (label, `menu`, `count`, `list`, optional `detail`). The menu, D-pad cycle and router follow.
3. **Is it a new kind of block?** Add a builder to `ui/components.ts` (or a new `ui/` module for something large), its CSS block to `components.css` using tokens, a guide section, and a row in the Components table above. Make it generic over its data; keep tracking-model knowledge in `views/`.
4. **Is it a new status?** Give it a tone in `vocabulary.toml` if it is vocabulary; otherwise add one mapping in `ui/status.ts`.
5. **Is it a new loop shape?** Add its bitmap to `SHAPE` in `ui/sprites.ts`; until then it shows the neutral ring.
6. **Does it need the shell?** It almost never does. Talk to Sebastien first.

Copy: plain sentences; say what a number counts ("498/512 pass"), not what it implies; name files and ids exactly as in the tracking files.

## Checking a change

1. `bun run typecheck`.
2. `bun run dev` and open the routes you touched, plus `#/guide`.
3. Look at 1440×900, 1024×768 and a narrow window (640 wide): nothing clipped, no text overlapping, tables scroll inside their window rather than the screen.
4. Use the controls on the pages you touched: ↑ ↓ through rows, X on a row with a link, Z back, ← → between pages, Shift to jump windows.
5. Every status you added shows a sprite and a label.

## History

The design was chosen over eight rounds of proposals with Sebastien; the proposals, his verdicts and the static mocks are in `draft-personal/autoresearch/design-handoff.md` and `draft-personal/autoresearch/design/`.
The chosen direction is round 7's "left rail" layout built on round 6's trimmed handheld.
