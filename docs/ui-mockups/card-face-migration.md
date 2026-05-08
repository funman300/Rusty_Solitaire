# Card-face artwork migration plan

**Status:** planning artifact (no code changed by this document).
**Tracks:** the "Card-face / suit / card-back artwork regeneration"
item in `SESSION_HANDOFF.md` → "Visual-identity follow-ups"
(SESSION_HANDOFF Resume prompt option D).
**Companion to:** `docs/ui-mockups/design-system.md` (Game Cards
spec, lines 214–233) and `docs/ui-mockups/desktop-adaptation.md`
(rules-based companion to the mockups).

## Why this is a multi-session arc

Every post-v0.20.0 visual-identity port to date (modal scaffold,
toasts, table chrome, splash boot screen, replay overlay) was a
**single rendering path** — change tokens, change comments, ship.
Cards have **two** rendering paths that are visually identical
today and would visually disagree the moment one moves:

1. **PNG path (production).** `assets/cards/faces/<rank><suit>.png`
   loaded into `CardImageSet.faces[suit][rank]` at startup; card
   sprites blit the texture. 52 face PNGs + 5 back PNGs already
   in `assets/`, all the legacy white-card aesthetic from the
   pre-Terminal design system.
2. **Constant fallback (tests + asset-missing edge).** When
   `CardImageSet` isn't a registered resource (the case under
   `MinimalPlugins` test fixtures, and the bare-bones path the
   first-frame of production hits before assets resolve), the
   renderer falls back to solid-colour sprites driven by the
   `card_plugin` constants:
   - `CARD_FACE_COLOUR` — `(0.98, 0.98, 0.95)` cream-ish white.
   - `RED_SUIT_COLOUR` — `(0.78, 0.12, 0.15)` warm red.
   - `BLACK_SUIT_COLOUR` — `(0.08, 0.08, 0.08)` near-black.
   - `CARD_FACE_COLOUR_RED_CBM` — `(0.85, 0.92, 1.0, 1.0)` light
     blue (the legacy color-blind tint).
   - `card_back_colour(idx)` — five legacy back themes.

A single-path migration leaves a known-broken state where tests
pass against Terminal constants while a human sees legacy artwork
on screen — the exact bisection-hostile drift the handoff's
"in lockstep" warning preempts.

## Target state — Terminal aesthetic

Per `design-system.md` § Game Cards (lines 214–233):

### Card face

| Element | Spec |
|---|---|
| Background | `#1a1a1a` |
| Border | 1 px solid in **suit colour** (pink for ♥/♦, foreground gray for ♠/♣) |
| Corner radius | 8 px |
| Top-left | rank in JetBrains Mono **Bold 18 px** + small suit glyph (10 px) |
| Bottom-right | large suit glyph (32 px), rotated 180° |
| Glyph fill rule | ♥ ♠ filled; ♦ ♣ outlined (1.5 px stroke). Always on, not a toggle. |

### Suit colours (always-on glyph differentiation is the *primary*
distinguishing mechanism; colour is supplementary):

| Suit | Default | Color-blind mode |
|---|---|---|
| Hearts | `#fb9fb1` (pink) | `#6fc2ef` (cyan) |
| Diamonds | `#fb9fb1` (pink) | `#6fc2ef` (cyan) |
| Spades | `#d0d0d0` (gray) | `#d0d0d0` (unchanged) |
| Clubs | `#d0d0d0` (gray) | `#d0d0d0` (unchanged) |

### Card back ("Terminal" theme)

| Element | Spec |
|---|---|
| Background | `#151515` |
| Pattern | horizontal scanlines at 2 px pitch in `#1a1a1a` (1 px line, 1 px gap), full bleed |
| Border | 1 px solid `#353535` |
| Top-left badge | 12×16 px solid `#6fc2ef` block, 6 px from corner |
| Bottom-right monogram | `▌RS` in JetBrains Mono 12 px `#505050`, 6 px from corner |
| Corner radius | 8 px |
| Theme name / author | `"Terminal"` / `"Rusty Solitaire"` |

## Generation pipeline — programmatic SVG via the existing
`resvg` stack

### Why this path (vs. external tooling or direct `tiny_skia`)

The codebase already ships an SVG-to-PNG rasteriser at
`solitaire_engine/src/assets/svg_loader.rs`:

- Public `rasterize_svg(svg_bytes: &[u8], target: UVec2) -> Result<Image, _>`
- Backed by `usvg` (parser) + `resvg` (renderer) + `tiny_skia`
  (CPU pixmap)
- Bundled font db includes JetBrains-style mono (FiraMono — same
  face the splash uses; close enough to JetBrains Mono for
  rasterisation purposes, and identical to what the Bevy UI
  consumes in the rest of the app)
- `RenderAssetUsages::default()` is the call-site convention here

This means: **generating new card PNGs is one new file
(`solitaire_engine/examples/card_face_generator.rs`) calling an
existing public function.** No new dependencies, no asset-pipeline
changes, no build-script machinery. Anyone who runs the example
gets bit-identical artwork.

The two alternatives are weaker:

- **External tool (Inkscape / Figma / hand-design)** — produces
  one-off PNGs that can't be re-generated reproducibly without
  re-opening the source files in a specific tool. Iteration cost
  is high; design tweaks (e.g. "make the suit glyph 2 px larger")
  require a designer-in-the-loop.
- **Direct `tiny_skia` painting calls** — bypasses SVG entirely,
  but loses the readability of "open the SVG to see exactly what
  the card looks like." Also reinvents primitives (rounded
  rectangles, text layout) that `usvg` already handles.

### Output format

PNG, RGBA8 sRGB, **dimensions 256 × 384** (2:3 aspect, half the
default `SvgLoaderSettings` of 512 × 768).

Rationale: cards never exceed ~250 px wide on desktop windows
today, and 256 × 384 PNGs are ~6 KB each at this content density
(13.4 KB total for a full deck of 52 + 5 backs). The default 512 ×
768 is 2× what's needed and quadruples the on-disk asset weight.
The existing legacy PNGs are 512 × 768 — reducing the new ones
halves the runtime asset size.

## Lockstep migration — recommended order

Each step is a separate commit; the constraint is that **steps 4
and 5 must land in the same commit** (or at most adjacent commits
on the same branch) so the rendered output never diverges between
the two paths.

1. **(Done — this commit)** Land the migration plan doc.
2. **Land the SVG generator example.** New
   `solitaire_engine/examples/card_face_generator.rs`. Output
   goes to `assets/cards/faces/` and `assets/cards/backs/`. Run
   once locally to seed the new artwork. The example file stays
   in-tree as a regenerator for future tweaks.
3. **(Optional — can land separately)** Add a one-shot regression
   test that re-runs the generator into a `tempdir` and compares
   the resulting bytes against the on-disk artwork; pinning the
   generator output prevents silent drift if `usvg`/`resvg` ever
   tweak rendering. Skip if the test runtime cost is unacceptable.
4. **Land the new artwork** (PNG bytes from step 2 committed to
   `assets/cards/`) **and** the constant migration in the *same
   commit*:
   - `CARD_FACE_COLOUR` → `Color::srgb(0.102, 0.102, 0.102)` (`#1a1a1a`)
   - `RED_SUIT_COLOUR` → `Color::srgb(0.984, 0.624, 0.694)` (`#fb9fb1`)
   - `BLACK_SUIT_COLOUR` → `Color::srgb(0.816, 0.816, 0.816)` (`#d0d0d0`)
   - `CARD_FACE_COLOUR_RED_CBM` → `Color::srgb(0.435, 0.761, 0.937)` (`#6fc2ef`) — note this is now the colour-blind *suit* colour, not a face tint; semantics shift slightly.
   - `card_back_colour(idx)` — re-author for the Terminal palette;
     index 0 stays the canonical "Terminal" back from `design-system.md`.
5. **Test updates land in step 4's commit.** The pinning tests at
   `card_plugin.rs` lines 1749, 1750, 1767, 1768, 2057, 2063,
   2071, 2081 all assert against the old constants. New
   assertions update in lockstep with the constant changes.

## CBM (color-blind mode) semantics shift — flag

The **legacy** `CARD_FACE_COLOUR_RED_CBM` was a *face tint* — red
suits got a light-blue background wash. The **Terminal** spec
moves CBM into the *suit colour* itself (red glyphs swap to cyan).
Step 4 will rename / repurpose this constant; it's not a 1:1
replacement.

Two options:

- **Rename + repurpose:** `CARD_FACE_COLOUR_RED_CBM` →
  `RED_SUIT_COLOUR_CBM`. Communicates the semantic shift in the
  symbol name. Requires touching every callsite.
- **Keep the name, change the meaning:** less code churn but
  worse for greppability — a future reader hitting the legacy
  name will assume face-tint behaviour.

Recommendation: **rename**. The CBM swap is a one-frame operation
even if it touches every existing callsite (currently lines 642,
2071, 2081 per `grep -n CARD_FACE_COLOUR_RED_CBM`).

## Theme system — out of scope here

The card-theme system (`docs/CARD_PLAN.md`, `theme/plugin.rs`)
already supports user-supplied themes via `assets/themes/<theme>/`
SVG files rasterised by `svg_loader.rs`. The new Terminal artwork
is the **default theme**, not a new entry in the theme picker —
the theme system continues to overlay user themes on top of the
default at runtime.

If the next session wants to also ship Terminal as a *named theme
slot* (so a user can switch back to the legacy artwork via the
theme picker), that's an additive change after step 4 and lives
in `theme::plugin::apply_theme_to_card_image_set`.

## Test impact summary

`grep -n CARD_FACE_COLOUR\\b\|RED_SUIT_COLOUR\\b\|BLACK_SUIT_COLOUR\\b` in
`card_plugin.rs`:

- Line 1749–1750: red-suit text colour assertions (♥ + ♦).
- Line 1767–1768: black-suit text colour assertions (♠ + ♣).
- Line 2057, 2063: face-colour assertion in default mode.
- Line 2071, 2081: face-colour assertion in CBM.

The four suit-colour and two face-colour tests are **invariant
guards** — they exist precisely so a constant tweak surfaces here
rather than in a visual review. Step 4 updates each in lockstep
with the constant value change. No new test infrastructure
needed.

## Open questions to resolve before step 4

1. **Border colour conflict.** The spec (line 218) says "Border:
   1 px solid in suit colour." The fallback path doesn't draw a
   border today — it draws solid-colour sprites. Step 4 either:
   (a) leaves the fallback as solid-colour squares (the test
   environment doesn't visually validate borders anyway), or
   (b) extends the fallback renderer to paint a 1 px outline.
   Recommend (a) — fallback fidelity isn't load-bearing.
2. **Glyph rendering in the constant fallback.** The fallback
   today doesn't render suit glyphs at all — it's a coloured
   square. The spec's filled-vs-outlined glyph differentiation
   only matters in the PNG path. No change to the constant
   fallback for glyphs.
3. **High-contrast mode.** `design-system.md` line 274 mentions
   a high-contrast accessibility mode (boosts foreground from
   `#d0d0d0` to `#f5f5f5`, suit-red from `#fb9fb1` to `#ff8aa0`).
   Not currently implemented anywhere; out of scope for this
   migration but worth flagging for a future accessibility pass.

## Post-migration — what's still open

- **High-contrast mode** (above).
- **Reduced-motion mode** for card lift / drop transitions
  (also a `design-system.md` accessibility item, separate from
  artwork).
- **The 9 missing-plugin screens** (splash, challenge,
  time-attack, weekly-goals, leaderboard, sync, level-up,
  replay, radial-menu) per `project_ui_overhaul` memory still
  need their plugin ports — separate from the cards arc.

## Sign-off criteria for "D closed"

D from the SESSION_HANDOFF Resume prompt is closed when **all of
the following hold simultaneously**:

- The 52 face PNGs + 5 back PNGs in `assets/cards/` are the
  Terminal-aesthetic artwork (regeneratable via the example).
- The five `card_plugin` constants reflect the Terminal palette.
- All pinning tests pass against the new values.
- A human boots the game and sees Terminal cards (not white
  cards). This sign-off needs a real `cargo run`, not just
  `cargo test`.
