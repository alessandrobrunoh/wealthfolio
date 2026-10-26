# UI Restyle — Light-First Minimal Redesign (OpenAI/Cursor direction)

> Status: Proposed design. Direction chosen by the requester: **B —
> light-first**, with the dark variant rebuilt on the same visual logic. Style
> references: Linear, Cursor, OpenAI (provided in the requesting thread). Repo:
> Wealthfolio app UI (`apps/frontend` + `packages/ui`).

## The Problem

Wealthfolio's current look (the "Classic" theme) reads as a traditional
desktop-finance app rather than a modern tool. The specific symptoms:

- **Chrome outweighs content.** Pill-shaped buttons and badges (9999px radii),
  10px card radii, drop shadows on cards, switches, popups and dialogs. The
  container draws more attention than the balances, holdings and charts inside
  it.
- **Airy density on dense data.** Large card titles (1.25rem), generous paddings
  and 44px control heights suit forms, not tables of holdings. The app's core
  screens are data-dense; the current spacing wastes vertical space.
- **Warm-paper personality.** The light theme is Flexoki-derived (warm cream
  paper, warm grays) and the dark variant inherits hues from it. Neither hits
  the crisp, near-neutral baseline users now expect from tools like Linear,
  OpenAI or Cursor.
- **Mixed typographic voices.** Inter for most UI, but Merriweather serif is
  hardcoded as a display voice in several features (goal cards, retirement
  planner headlines, spending insights, the sidebar wordmark) and is also a
  user-selectable font preference. The serif adds a traditional accent that
  works against a single minimal voice.

The user asked for "minimal, modern, compact" in the Linear/OpenAI/Cursor
family. The gap is real: those systems use hairline borders, small radii,
400–500 weights, near-monochrome surfaces, and let content be the only visual
texture.

## Overview

Restyle the entire app UI around a **light-first, near-monochrome, hairline
system**, with OpenAI as the primary reference (structure from whitespace and
hairline borders, one filled dark action per view, no decoration) and Cursor as
the secondary reference (weight-400 headings, sharp 4px geometry). Linear
contributes the compactness discipline, not its dark canvas or accent color.

One visual language across both theme variants: same geometry, same density,
same components. Light is the default; dark is a true inverted variant built on
the same rules — not a second personality.

All changes flow through the existing theme-token system
(`apps/frontend/src/globals.css`,
`apps/frontend/src/themes/definitions/classic.css`) and the shared components in
`packages/ui`. No per-page repaints, no logic changes.

Direction decisions already taken:

1. **Light-first.** White/paper canvas, near-black ink text, neutral grays.
   Chosen over dark-first because a finance management app is read like a
   document: statements, balances, tables.
2. **Small radii.** 4–8px on buttons, inputs, cards; pills (9999px) and 10–16px
   card radii are retired.
3. **Hairlines over shadows.** Borders at low-opacity black (OpenAI pattern,
   `rgba(0,0,0,0.12)`-style) replace most drop shadows; elevation comes from
   surface steps and borders.
4. **Inter as the default UI voice.** Hardcoded serif styling leaves the default
   UI; the persisted sans/serif/mono preference stays available. JetBrains Mono
   stays for code/identifiers; financial figures use tabular numerals where that
   improves scanning.
5. **Finance semantics untouched.** Gain/loss colors and chart series colors
   survive the monochrome reduction; the primary action is a single dark filled
   button (OpenAI's "one period at the end of the sentence"), not a chromatic
   accent that would collide with green/red semantics.

## Goals

1. One coherent minimal visual language across all screens and both theme
   variants (light default, dark variant).
2. Content over chrome: numbers, tables and charts are the visual texture;
   containers recede.
3. Compact, scannable density — denser tables, smaller titles, tighter paddings
   — without breaking readability, touch targets, or the existing 4.5:1 contrast
   guarantees (`e2e/design-system/colors.spec.ts`).
4. Token-driven restyle: values change in the theme system and shared
   components; call sites change only where layout composition itself must
   compress.
5. Layout and page composition tighten where the current structure is airy (page
   headers, card stacks, table rows), preserving RTL correctness.

## Non-Goals

- No new features, pages, routes, or navigation restructuring.
- No backend, adapter, storage, or business-logic changes.
- No new theming infrastructure — keep the existing token contract
  (`themes/definitions/classic.css` roles); restyle values, not mechanisms.
- Not removing the dark variant; it gets the same system inverted.
- Not removing the font preference setting (sans/serif/mono); only the hardcoded
  serif usages in feature components go away.
- No marketing-site work (none exists in this repo).
- No rebranding: logo, app name, and icon set stay as they are.

## Technical Plan

### Strategy: retune the existing token system, in place

The theming machinery is already right for this job: one definition file owns
every color and geometry value, a generator compiles it, and shared components
read tokens instead of hardcoding styles. So the restyle is a **value swap
inside the existing system**, not a new system:

```
themes/definitions/classic.css        source of truth (:root light + .dark)
        |   pnpm themes:generate  (apps/frontend/scripts/theme-generate.mjs)
        v
themes/generated/themes.css           committed output; themes:check guards staleness
        |   @theme inline (globals.css) -> Tailwind utilities
        |                              (bg-background, text-muted-foreground, ...)
        v
packages/ui components                read geometry/surface tokens
        |                              (--button-radius, --card-shadow, --popup-radius, ...)
        v
feature pages                         compose shared components; only the
                                       density/layout pass touches these
```

No new theme is added and none of the machinery changes: the generator, the
theme selector, the token contract (`requiredTokens`), and the per-role comments
stay. Classic keeps its name and remains the only theme; its values change.

### Palette — light (default)

Neutral white canvas, near-black ink, hairline structure (OpenAI baseline).
Semantic hues (success/destructive/warning/info) keep their current families so
contrast guarantees carry over; they are re-verified on the new surfaces.

| Role                       | Today (Flexoki) | New                               |
| -------------------------- | --------------- | --------------------------------- |
| background                 | cream `#fffcf0` | `#ffffff`                         |
| foreground                 | warm near-black | `#0d0d0d`                         |
| card / popover             | cream `#f2f0e5` | `#ffffff` + hairline border       |
| primary (filled action)    | warm near-black | `#0d0d0d`, white text             |
| secondary / muted / accent | warm grays      | neutral grays `#f4f4f4`–`#f5f5f5` |
| muted-foreground           | warm gray       | `#666666` (5.7:1 on white)        |
| border / input             | warm gray       | `rgba(0,0,0,0.10)` hairline       |
| ring (focus)               | warm gray       | `#0d0d0d`                         |

The current primary is already a dark filled button — that survives; what
changes is everything around it (warm cream surfaces, pills, shadows).

### Palette — dark (inverted variant, same geometry)

Neutral dark, not Flexoki's warm dark:

| Role             | New                                           |
| ---------------- | --------------------------------------------- |
| background       | `#0a0a0a`                                     |
| foreground       | `#ededed`                                     |
| card / popover   | `#141414` + `rgba(255,255,255,0.10)` hairline |
| primary          | `#f0f0f0` with dark text                      |
| muted-foreground | `#a3a3a3`                                     |
| border / input   | `rgba(255,255,255,0.10)`                      |

### Geometry and density tokens

The `--radius` base derives sm/md/lg/xl, so one value retunes the ladder.

| Token                | Today      | New                   | Effect                                                                     |
| -------------------- | ---------- | --------------------- | -------------------------------------------------------------------------- |
| `--radius`           | 0.625rem   | 0.5rem                | ladder becomes 4/6/8/12px                                                  |
| `--button-radius`    | 9999px     | 6px                   | pill buttons retired                                                       |
| `--control-radius`   | calc       | calc (→ 6px)          | inputs/selects follow                                                      |
| `--control-height`   | 2.75rem    | 2.25rem               | 36px desktop; 44px touch floor kept by the existing `pointer: coarse` rule |
| `--card-radius`      | 10px       | 8px                   | calmer cards                                                               |
| `--card-shadow`      | soft drop  | none                  | hairline border carries the edge                                           |
| `--popup-radius`     | 1rem       | 0.625rem              | menus/palette tighten                                                      |
| `--dialog-radius`    | 10px       | 0.75rem               | dialogs stay slightly rounder                                              |
| `--badge-radius`     | 9999px     | 0.25rem               | pills → small rounded rects                                                |
| `--segmented-radius` | 9999px     | 6px                   | toggle groups follow                                                       |
| `--popup-shadow`     | heavy 50px | one soft `0 4px 16px` | popovers still read as floating                                            |
| `--card-title-size`  | 1.25rem    | 0.9375rem             | card titles stop shouting                                                  |

Button/field/switch/thumb shadows (`--button-*-shadow`, `--field-shadow`,
`--switch-thumb-shadow`, `--segmented-thumb-shadow`) go to none or a single
hairline inset; dialogs keep a slightly larger soft shadow because they float
over content.

### Typography

- Inter stays; weights stay in the 400–500 band; `--button-weight` 500 stays.
- The `@theme` type scale is untouched (11px floor, rem-based). Density comes
  from geometry tokens and layout, not from shrinking the scale.
- **Hardcoded `font-serif` is removed from feature components** (goal cards,
  retirement-planner headlines, spending insights, sidebar wordmark): display
  numbers move to Inter with `tabular-nums`.
- The user-facing font preference (sans/serif/mono, persisted) **stays** —
  removing a persisted setting is a product decision outside this restyle.
  Merriweather remains loaded for users who pick it.

### Color ramp and variable cleanup

- The neutral `base-*` ramp is replaced with a gray scale, and
  `--color-paper`/`--color-black` move to neutral values. Colored ramps
  (`red-*`, `orange-*`, `yellow-*`, `green-*`, `cyan-*`, `blue-*`, `purple-*`,
  `magenta-*`) retain their hues and step indices because they carry status,
  toast, and other functional meanings. Contrast is re-verified per spec.
- `--color-paper`/`--color-black` move to neutral `#ffffff`/`#0a0a0a`.
- The `--flexoki-*` prefix is renamed to role-based names in the definitions;
  the two direct consumers (theme-selector mini previews, the lever-slider thumb
  in `globals.css`) follow. Keeping a "flexoki" name over a neutral palette
  would mislead every future reader.
- Chart palettes (`--chart-1..9`, treemap heatmaps, fi-stream, event colors):
  **hue families unchanged**; they are already muted and multi-series
  distinction is a functional requirement, not decoration. Optional lightness
  retune on the new surfaces happens only if visual QA asks for it.

### Component work (packages/ui)

Almost all of this lands as token values; component code changes only where
radius/shadow is hardcoded instead of tokenized: buttons, card, badge,
popover/menu/command, dialog/sheet, switch, segmented toggle group, calendar,
tabs. Table and data-grid rows get denser padding and `tabular-nums` on numeric
columns.

### Layout composition pass

After the token swap, one density sweep over the app shell and main screens:
page headers (smaller titles, tighter margins), card-stack gaps (e.g.
`space-y-6` → `space-y-4`), card padding (`p-6` → `p-4` where tables live),
table row heights. The sidebar and the mobile liquid-glass nav retune through
their existing tokens. Guided by the net-worth e2e layout suite so the principal
dashboard stays verified.

### Validation

The detailed validation sequence is in **Phase 5 — Guardrail updates and full
validation** below. The main guarantees are contrast in both modes, layout and
sticky-header regressions, RTL behavior, and visual review of both themes.

### What does not change

Theme infrastructure (generator, selector, token contract), the font preference
setting, semantic and chart token roles, RTL/i18n rules, component APIs, all
logic and data.

## Alternatives Considered

### A. Dark-first direction (Linear as primary reference)

Near-black canvas, hairline borders, one electric accent.

**Rejected.** The strongest counter to light-first is real — many finance tools
are dark (trading terminals) and dark hides glare during long sessions. But
Wealthfolio is read like a document (statements, balances, tables), the user
prefers light, and a light default with a true dark variant covers both.
Linear's acid-lime accent was also rejected on its own: it collides with
gain/loss green-red semantics in a finance app. Linear contributes compactness
discipline only.

### B. Fuse all three references

Take Linear's accent, Cursor's parchment, OpenAI's whitespace in one system.

**Rejected.** The three systems disagree on canvas (dark vs cream vs white),
accent (lime vs ember vs none) and geometry (pills vs 4px vs mixed). A fusion
keeps every conflict and resolves none. The plan picks one primary (OpenAI) with
Cursor as secondary detail reference.

### C. Ship a new optional theme ("Minimal") alongside Classic

The generator already supports extra themes (`themeOrder` lists six IDs with no
definition files). Add `minimal.css`, keep Classic as default.

**Rejected.** The request is to restyle the app, not to add a preference. An
opt-in theme doubles the surface every future component change must be checked
against, and leaves the dated default in place for everyone who never opens
settings. Classic is retuned in place.

### D. Per-component repaint instead of token swap

Edit button/card/badge styles directly in `packages/ui` and feature pages.

**Rejected.** The theme system exists precisely so surfaces are values, not
code. A repaint scatters the same changes across dozens of files, invites drift
between light and dark, and cannot be checked by `themes:check` or re-tuned
later. Component code changes only where it bypasses tokens today (hardcoded
radius/shadow, hardcoded `font-serif`).

### E. Drop the font preference setting with the serif voice

Remove sans/serif/mono entirely, Merriweather with it.

**Rejected.** It is a persisted user setting; deleting it changes product
behavior beyond a restyle and silently overrides user choice. Only the hardcoded
serif usages in feature components are removed; the setting stays.

### F. Monochrome chart palette to match the chrome

Take the allocation palette to a gray ladder for full visual unity.

**Rejected.** Chart series must stay distinguishable at small sizes and for
color-vision-deficient users; hue variety there is function, not decoration. The
muted forest/sage/sand palette already sits calmly on neutral surfaces.

### G. Keep the `--flexoki-*` variable names

Rename is churn; names are invisible to users.

**Rejected.** A neutral palette behind Flexoki-named variables misleads every
future reader and invites re-warming by accident. The rename is mechanical and
touches three files (definitions, theme-selector previews, lever-slider thumb).

## Detailed Implementation

Five phases, each independently verifiable. Phase 1–2 are deterministic file
lists; Phases 3–4 are defined sweeps with a method and acceptance criteria, plus
the files experience says they will touch. No files are deleted; the only new
artifact is regenerated output. No schema, config, or doc changes (no doc
references these tokens — verified).

### Phase 1 — Theme token core

| #   | File                                                                                        | Change                                                                                                                                                                                                                                                                                                                                                | Rationale                                                                                            |
| --- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| 1   | `apps/frontend/src/themes/definitions/classic.css`                                          | Replace Flexoki light/dark ramps with the neutral palettes from the Technical Plan; retune geometry tokens (`--radius` ladder, pill retirements, shadow removals, `--control-height`, `--card-title-size`); rename `--flexoki-*` variables to role names (`--scale-bg`, `--scale-surface`, `--scale-ink`, `--scale-red`, … — final names chosen here) | Single source of truth; every consumer follows one edit                                              |
| 2   | `apps/frontend/src/themes/generated/themes.css`                                             | Regenerated via `pnpm themes:generate`, committed                                                                                                                                                                                                                                                                                                     | Committed build output; `themes:check` fails when stale                                              |
| 3   | `apps/frontend/src/globals.css`                                                             | Neutralize `base-*`; set `--color-paper`→`#ffffff`, `--color-black`→`#0a0a0a`; preserve colored semantic ramps and their step indices; follow the variable rename in the lever-slider thumb; retune `.card-mobile` radii to the new ladder                                                                                                            | Neutral surface ramps change without removing semantic colors; rename keeps the slider/thumb working |
| 4   | `apps/frontend/src/components/theme-selector.tsx`                                           | Follow the variable rename in the mini theme previews                                                                                                                                                                                                                                                                                                 | Only component reading `--flexoki-*` directly                                                        |
| 5   | `apps/frontend/scripts/theme-generate.mjs`, `apps/frontend/scripts/theme-generate.test.mjs` | Comments only ("Flexoki" mentions); no logic change expected — test assertions use synthetic fixtures, not real theme values                                                                                                                                                                                                                          | Keep docs honest; generator is name-agnostic (verified against `requiredTokens`)                     |

**Acceptance:** app boots in both modes; `themes:check` clean; colors.spec.ts
passes (fix any role that drops below 4.5:1 by darkening its foreground step,
not by re-warming the palette).

### Phase 2 — Single typographic voice

Remove hardcoded `font-serif`; display figures move to Inter + `tabular-nums`:

| #   | File                                                                                     | Occurrences |
| --- | ---------------------------------------------------------------------------------------- | ----------- |
| 6   | `apps/frontend/src/pages/layouts/navigation/app-sidebar.tsx`                             | wordmark    |
| 7   | `apps/frontend/src/features/goals/components/goal-card.tsx`                              | 5           |
| 8   | `apps/frontend/src/features/goals/pages/goals-dashboard-page.tsx`                        | 1           |
| 9   | `apps/frontend/src/features/goals/save-up/components/save-up-detail.tsx`                 | 1           |
| 10  | `apps/frontend/src/features/goals/retirement-planner/pages/risk-lab-page.tsx`            | 4           |
| 11  | `apps/frontend/src/features/goals/retirement-planner/pages/dashboard-page.tsx`           | 1           |
| 12  | `apps/frontend/src/features/spending/components/reports/insights/what-changed-stage.tsx` | 2           |
| 13  | `apps/frontend/src/features/spending/components/reports/insights/where-i-am-stage.tsx`   | 1           |

Explicitly untouched (the persisted font preference):
`apps/frontend/src/components/font-selector.tsx`,
`apps/frontend/src/pages/settings/appearance/appearance-form.tsx`,
`apps/frontend/src/pages/onboarding/onboarding-appearance.tsx`,
`apps/frontend/src/lib/settings-provider.tsx`,
`apps/frontend/src/addons/iframe/addon-sandbox-theme.ts` (+ its test), and the
Merriweather `@fontsource` imports in `globals.css`.

**Acceptance:** `grep -rn "font-serif" apps/frontend/src --include="*.tsx"`
returns only the preference-setting files listed above.

### Phase 3 — Component stragglers (`packages/ui`)

Method: sweep `packages/ui/src/components/{ui,common,data-grid,financial}` for
`rounded-full` on controls, `shadow-*` classes, hardcoded control heights
(`h-11`, `h-12`), and oversized paddings on chrome elements; replace with the
existing tokens. Expected hot files (from the component inventory):

| #   | Files                                                                                                                                                                                  |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 14  | `button.tsx`, `button-variants.ts`, `badge.tsx`, `switch.tsx`, `animated-toggle-group.tsx`, `toggle.tsx`, `toggle-group.tsx`                                                           |
| 15  | `input.tsx`, `textarea.tsx`, `select.tsx`, `combobox.tsx`, `checkbox.tsx`, `radio-group.tsx`, `calendar.tsx`, `calendar-rac.tsx`, `date-picker-input.tsx`, `date-range-picker.tsx`     |
| 16  | `card.tsx`, `table.tsx`, `dialog.tsx`, `alert-dialog.tsx`, `sheet.tsx`, `popover.tsx`, `dropdown-menu.tsx`, `command.tsx`, `tooltip.tsx`, `hover-card.tsx`, `tabs.tsx`, `progress.tsx` |
| 17  | `chart.tsx` — neutral gridline/cursor defaults if hardcoded; `data-grid` and `financial` dirs — row density and `tabular-nums` on numeric columns                                      |

Most of these already read tokens, so the expected diff is small; a file with no
hardcoded value simply doesn't change.

**Acceptance:** no `rounded-full` or `shadow-*` on buttons, fields, badges,
segmented controls, or menus in the package (full-round stays where it is
semantic: avatars, dots, loading spinners);
`pnpm --filter frontend exec vitest run` on affected suites.

### Phase 4 — Density and layout composition

| #   | Files                                                                                                                                                                                                                           | Change                                                                                                                                             | Rationale                                                              |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| 18  | `packages/ui/src/components/ui/page.tsx`, `shell.tsx`                                                                                                                                                                           | tighten error-page/page-shell spacing                                                                                                              | shared shells follow the density                                       |
| 19  | `apps/frontend/src/pages/layouts/app-layout.tsx`, `mobile-navigation-container.tsx`, `mobile-swipeable-layout.tsx`                                                                                                              | outer gaps and content padding tighten                                                                                                             | the app frame sets the rhythm                                          |
| 20  | `apps/frontend/src/pages/layouts/navigation/app-sidebar.tsx`, `floating-navigation-bar.tsx`, `mobile-navbar.tsx`, `apps/frontend/src/components/liquid-glass.tsx`                                                               | neutral glass tokens (set in Phase 1 definitions) land here; nav row heights tighten                                                               | floating nav and sidebar keep mechanism, change values                 |
| 21  | Page sweep: `apps/frontend/src/pages/{net-worth,dashboard,holdings,activity,performance,allocation-targets,income,insights,health,account,asset,settings,ai-assistant}` and feature pages under `src/features/{goals,spending}` | page headers shrink (title + actions row), card-stack gaps `space-y-6`→`space-y-4`, table-card padding `p-6`→`p-4`, sticky-header offsets re-check | the visible "compact" outcome; one consistent rule, not per-page taste |

Order matters: net-worth first (it owns the layout e2e suite), then the rest.

**Acceptance:** `pnpm test:e2e:net-worth` green; sticky headers
(`30-activity-sticky-header.spec.ts`, `35-holdings-sticky-header.spec.ts`,
`37-performance-header-layout.spec.ts`) still pass; `?dir=rtl` spot check on
net-worth, holdings, settings.

### Phase 5 — Guardrail updates and full validation

| #   | Files                                                                                                                              | Change                                                             | Rationale                                                |
| --- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------- |
| 22  | `e2e/design-system/colors.spec.ts`                                                                                                 | re-run on the new palette; update any hardcoded expected pairs     | the 4.5:1 contract is the restyle's safety net           |
| 23  | `e2e/design-system/visual.spec.ts`, `gallery.ts`, `overlays.spec.ts`, `call-site-overrides.spec.ts`, `rtl.spec.ts`, `a11y.spec.ts` | regenerate visual baselines; re-verify overlay layering, RTL, a11y | restyle changes pixels; these specs own those guarantees |

Full validation sequence (from repo root):

1. `pnpm themes:generate` && `pnpm --filter frontend themes:check`
2. `pnpm check` && `pnpm type-check`
3. `pnpm --filter frontend exec vitest run` (unit + component suites)
4. `pnpm build` && `pnpm build:tauri` (ramp + token changes hit both bundles)
5. Design-system e2e suite (`e2e/design-system/`)
6. `pnpm test:e2e:net-worth`
7. Manual visual QA, both modes, plus `?dir=rtl`

### Risks carried into implementation

- **Contrast edges:** muted-foreground `#666666` on gray-tinted surfaces (not
  white) can dip; colors.spec.ts is the arbiter, with the fix at the token step,
  not in component code.
- **Sticky headers and scroll offsets** interlock with tighter headers; the
  three sticky-header specs are the regression net.
- **Visual baselines** churn once, wholesale — expected, not a failure.
- **Addon sandbox** inherits theme values; `addon-sandbox` e2e must stay green
  since addons re-render the host palette.
