# CPA Editor (Rust) – design rules

Style rules for every screen of the egui application. The values live in `cpa-editor/src/theme.rs`; screens use the
helpers there instead of their own colours and sizes.

## Language
- The whole interface is English, including empty states and status texts.

## Theme
- Follows the light/dark setting of the system; both themes are tuned in `theme::apply`.
- Text: body 14 px, small 12 px, heading 18 px, monospace 13 px.
- Spacing: 8 px between items, button padding 10×5, interactive height 26 px.
- Fields and drop-down lists always have a visible 1 px border; disabled text keeps at least 3:1 contrast.

## Buttons
- At most one **primary** button in view (`theme::primary_button`, or `theme::action_button` with `primary`):
  accent fill, white strong text (the default fonts have no bold face), minimum 96×28. The primary button is the next step: Save CPA… only with unsaved
  changes, Save changes only when the form differs from the CPA, Validate CPA before the first run. Otherwise these
  are standard buttons. A disabled button is never primary.
- **Danger** buttons (`theme::danger_button`) for delete actions: red text, disabled without a selection, separated
  from the add buttons and followed by a confirmation dialog.
- Other buttons are standard egui buttons.

## Status colours
- OK / valid: green (`theme::ok_color`), problem / error: red (`theme::error_color`), warning: amber.
- Status is shown as a badge (`theme::badge`): coloured background, short text, e.g. "Valid" or "3 errors".

## Layout
- Tabs at the top as a tab bar: the active tab has an accent underline and strong text; no focus ring.
- Master-detail screens: list or tree left, `SidePanel::right` (default 420 px, resizable) for details.
- A detail panel without a selection shows a centred empty state (weak text explaining what to select), never a
  disabled form.
- Forms use one `Grid` with two columns: labels left, fields stretch to the same right edge.
- The log is a collapsible, resizable bottom panel ("Log (n)"), with the file actions above it; an empty log is
  closed.
- One verb per action: "Open CPA…" everywhere (menu, bottom bar, empty states).

## Trees and tables
- Text that does not fit ends with "…"; the full text is in the tooltip.
- Group rows (partners) are header rows: bold, slightly larger, a 3 px coloured bar on the left; no full light fill
  in dark mode. Light fills are reserved for the selection. Partner colours are blue and violet, never a status
  colour (red, amber, green).
- The last column takes the remaining width; put the longest column last.
- Send and Receive are shown with a drawn arrow (shaft and head), not a filled triangle, which means open/close.
- The default egui fonts have only basic characters: no ▶ ⏷ ● ✓ ⚠ → ←. Draw shapes with `theme::paint_arrow` /
  `theme::paint_direction` instead.
- Secondary information in a row is weak text after the primary text.

## Empty states
- Centred: a heading saying what is missing and one weak sentence saying what to do, with the action button below it.
