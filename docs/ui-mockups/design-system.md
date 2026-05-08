---
name: Terminal
colors:
  surface: '#151515'
  surface-dim: '#0d0d0d'
  surface-bright: '#2a2a2a'
  surface-container-lowest: '#0a0a0a'
  surface-container-low: '#1a1a1a'
  surface-container: '#202020'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353535'
  on-surface: '#d0d0d0'
  on-surface-variant: '#a0a0a0'
  inverse-surface: '#d0d0d0'
  inverse-on-surface: '#151515'
  outline: '#505050'
  outline-variant: '#353535'
  surface-tint: '#a54242'
  primary: '#a54242'
  on-primary: '#151515'
  primary-container: '#3a1f1f'
  on-primary-container: '#d5a8a8'
  inverse-primary: '#993e3e'
  secondary: '#acc267'
  on-secondary: '#151515'
  secondary-container: '#2a3320'
  on-secondary-container: '#c5d585'
  tertiary: '#e1a3ee'
  on-tertiary: '#151515'
  tertiary-container: '#3a2a40'
  on-tertiary-container: '#eec3f5'
  error: '#fb9fb1'
  on-error: '#151515'
  error-container: '#4a2530'
  on-error-container: '#fdc3ce'
  background: '#151515'
  on-background: '#d0d0d0'
  surface-variant: '#353535'
  suit-red: '#fb9fb1'
  suit-black: '#d0d0d0'
  suit-red-cb: '#acc267'
  highlight-valid: '#acc267'
  highlight-celebration: '#e1a3ee'
  highlight-warning: '#ddb26f'
  highlight-info: '#12cfc0'
typography:
  hud-score:
    fontFamily: JetBrains Mono
    fontSize: 24px
    fontWeight: '700'
    lineHeight: 32px
    letterSpacing: '-0.02em'
  hud-timer:
    fontFamily: JetBrains Mono
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  card-rank:
    fontFamily: JetBrains Mono
    fontSize: 18px
    fontWeight: '700'
    lineHeight: 18px
  body-md:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  label-caps:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: '0.08em'
  headline:
    fontFamily: JetBrains Mono
    fontSize: 28px
    fontWeight: '700'
    lineHeight: 32px
    letterSpacing: '-0.01em'
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.5rem
  lg: 0.75rem
  xl: 1rem
  full: 9999px
spacing:
  margin-edge: 1rem
  gutter-card: 0.375rem
  stack-overlap: 2rem
  touch-target-min: 48dp
---

## Brand & Style

The "Terminal" design system replaces the previous "Premium Solitaire" calm-indie aesthetic with a **retro-terminal / synthwave** identity. The intent is the visual confidence of a well-tuned terminal emulator (think Berkeley Mono dotfiles, base16-eighties, CRT phosphor): monospaced, dense, legible, snappy. It is *not* casino-glitz, *not* skeuomorphic felt, and *not* whimsical.

The personality is **technical, deliberate, slightly playful**. Cards are flat with thin colored strokes; the HUD reads like a status bar; modals look like terminal panes. Motion is short and snap-easing — no bouncy springs. Long-session calm is preserved by keeping the chroma low and reserving saturated accents for *meaning* (CTAs, feedback, celebrations) rather than decoration.

Influences: base16-eighties (Chris Kempson), Berkeley Mono, Vim/Neovim status lines, the iA Writer aesthetic, classic CRT phosphor with no chromatic aberration.

## Palette

The palette is base16-eighties — a 16-slot terminal palette where indices 00–07 form a monochrome ramp and 08–0F provide saturated accents. We map base16 slots to Material Design 3 token roles below.

### Source palette (base16-eighties)

| Slot | Hex | Role |
|---|---|---|
| base00 | `#151515` | background |
| base01 | `#202020` | surface-container |
| base02 | `#303030` | line-highlight (subtle) |
| base03 | `#505050` | outline / muted text |
| base04 | `#b0b0b0` | secondary text |
| base05 | `#d0d0d0` | foreground / on-surface |
| base06 | `#e0e0e0` | bright text |
| base07 | `#f5f5f5` | brightest highlight |
| base08 | `#fb9fb1` | red — used for `error`, `suit-red` |
| base09 | `#ddb26f` | orange — used for warning chips |
| base0A | `#acc267` | yellow/lime — used for `highlight-valid` (drag targets, valid moves) |
| base0B | `#12cfc0` | green/teal — used for `highlight-info` (toasts, neutral status) |
| base0C | `#6fc2ef` | cyan/sky — historically the primary CTA; now reserved for ad-hoc accents only |
| base0D | `#6fc2ef` | (alias) |
| base08 (project) | `#a54242` | brick red — primary CTA, focus ring, `selection` (project-specific extension; the base16-eighties `base08` slot is `#fb9fb1` pink which we keep as `error`/`suit-red`) |
| `suit-red-cb` slot | `#acc267` | lime — color-blind-mode swap for red suits (was `#6fc2ef` cyan before the 2026-05-08 primary-accent swap; lime is the next-best non-red base16-eighties accent) |
| base0E | `#e1a3ee` | violet — used for celebration (level-up, achievement unlock) |
| base0F | `#fb9fb1` | (alias) |

### Semantic assignments

- **CTA / Primary action**: brick red `#a54242`. Reserved for "Play," "New Game," "Save," "Resume," and the focus ring on selected cards. Never used decoratively. (Was cyan `#6fc2ef` before the 2026-05-08 swap.)
- **Valid-move / drag-target highlight**: lime `#acc267`. Reserved for in-game feedback only. Never appears in chrome.
- **Celebration**: lavender `#e1a3ee`. Used for level-up flashes, achievement unlock cards, and the daily-streak chip when the streak is active. Quiet otherwise.
- **Warning / soft alert**: gold `#ddb26f`. Used for "challenge expires in N minutes" chips, sync-pending status, and the daily-seed countdown.
- **Info**: teal `#12cfc0`. Used for neutral system toasts and the sync-connected indicator.
- **Error**: pink `#fb9fb1`. Used for sync conflict, server unreachable, invalid move shake.

## Suit Colors

**Two-color traditional pairing**, with mandatory color-blind
support. Saturated red for hearts + diamonds, near-white for clubs
+ spades — the "Microsoft Solitaire on dark mode" feel of a real
playing-card deck. (A brief 4-color-deck experiment shipped between
v0.21.0 and the next post-cut commit; reverted to traditional
2-color at the player's request.)

| Suit | Default | Color-blind mode | Glyph differentiation |
|---|---|---|---|
| Hearts | `#e35353` (saturated red) | `#acc267` (lime) | Solid filled glyph |
| Diamonds | `#e35353` (saturated red) | `#acc267` (lime) | **Outlined glyph (1.5px stroke)** |
| Spades | `#e8e8e8` (near-white) | `#e8e8e8` (unchanged) | Solid filled glyph |
| Clubs | `#e8e8e8` (near-white) | `#e8e8e8` (unchanged) | **Outlined glyph (1.5px stroke)** |

The outlined-glyph treatment is the **primary** differentiation mechanism. Color is supplementary. This means a player viewing the game on a monochrome display, or with severe red-green deficiency, can still distinguish all four suits without context. This is a hard requirement, not an optional setting.

The "color-blind mode" toggle in Settings swaps both red suits (hearts + diamonds) from `#e35353` to `#acc267` (lime); clubs + spades stay at the near-white. The toggle does not turn the outlined glyphs on or off, because outlined glyphs are always on. (Was red→cyan before the 2026-05-08 primary-accent swap; CBM moved to lime to stay hue-distinct from the new red-family primary.)

## Typography

**Monospace-forward, dual-font system.**

- **JetBrains Mono** is used for: HUD (score, timer, moves), card rank/value text, all labels, all headlines, all numerals anywhere in the app, and any chip-style component. This is the dominant face.
- **Inter** is used only for: long-form body copy (Help screen, Settings descriptions, achievement tooltips, onboarding copy). It is the *exception*, not the default.

Weights: 400 regular, 500 medium for labels, 700 bold for HUD numbers and headlines. No 600 / no italics anywhere — the terminal aesthetic doesn't have them.

Letter spacing: tight (`-0.02em`) on HUD score for visual mass; wide (`+0.08em`) on uppercase labels for readability at 12px. Body uses default (0).

HUD numbers must use **tabular figures** (`font-feature-settings: 'tnum'`) so the timer and score don't reflow as digits change.

## Layout & Spacing

Optimized for **Android portrait, 390×844 (Pixel 6 baseline), API 34**.

- **Margins**: 16px (1rem) edge safety margin. *Tighter than the previous system's 24px.* Eighties palettes are dense by nature; over-padding fights the aesthetic.
- **Tableau**: 7-column layout, 32px (2rem) vertical card overlap. Tighter than before to fit a longer cascade on phone screens.
- **HUD position**: top of screen, in the system safe area. Bottom 64px holds the action bar (Undo / Hint / New Game / Auto-complete). Action bar is **always visible** in-game — no hover-fade — because there is no hover on touch.
- **Touch target minimum**: 48dp on all interactive elements. Cards in the tableau may be smaller visually but use a 48dp invisible hit area centered on the visible glyph.

## Elevation & Depth

Depth is created through **tonal layering and 1px outlines**, not blur shadows. (Synthwave-flat, not Material-soft.)

- **Level 0 (Background)**: the `#151515` base canvas.
- **Level 1 (Tableau slots, empty piles)**: 1px dashed outline in `#353535`. Empty foundations show a faint suit glyph at 12% opacity inside the outline.
- **Level 2 (Cards at rest)**: solid `#1a1a1a` fill, 1px solid border in the suit color (so the suit is detectable at a glance even if the card is partially obscured).
- **Level 3 (Active / dragged card)**: same border, but glow effect: 0 0 12px of `#a54242` at 40% opacity. **No scale transform** — flatness preserved. Z-index lifts above siblings.
- **Modals**: full-screen with backdrop `#151515` at 95% opacity (just enough to dim the table without blurring it). Modal panel is `#202020` with a 1px `#505050` border — like a terminal pane.
- **Toasts**: bottom of screen, `#202020` fill, 1px border in the toast's accent color (info=teal, warning=gold, error=pink, celebration=lavender). 16px monospaced caption.

No `box-shadow` is used anywhere. **All depth is achieved with borders and tonal value.** This is a hard constraint.

## Shapes

The shape language is **soft-rounded but tight**:

- **Cards**: `rounded-md` (8px) — slightly less rounded than the previous system's 16px to read more "technical."
- **Buttons / chips / inputs**: `rounded` (4px) default, `rounded-sm` (2px) for the smallest chips.
- **Modals / sheets**: `rounded-lg` (12px).
- **Avatars / circular indicators**: `rounded-full`.
- **Card-back pattern corners**: matches the card's `rounded-md`.

Selection highlights use a **2px inset stroke** in `#a54242` following the host shape's corner radius. Never an outer stroke — the outer stroke is reserved for the suit-color hairline.

## Motion

**Snappy, no spring.** All transitions use `ease-out` with a 120ms duration unless specified.

- Card lift (start drag): 80ms.
- Card place (drop): 120ms with a 16ms holdframe (no bounce).
- Modal enter: 200ms ease-out, fade + 8px translate-up.
- Modal exit: 120ms ease-in, fade only.
- Selection ring appear: 80ms.
- Win-summary stat reveal: 60ms each, staggered 40ms.
- HUD number tick: instant (no transition) — terminal counters don't ease.

**Optional CRT effect**: a 1-frame scanline sweep across the screen on game-state transitions (start, win, restart). User-toggleable in Settings. Off by default.

## Components

### Game Cards

Flat face design.
- Background: `#1a1a1a`
- Border: 1px solid in suit color (pink for hearts/diamonds, foreground gray for spades/clubs)
- Top-left: rank in JetBrains Mono Bold 18px + small suit glyph (10px)
- Bottom-right: large suit glyph (32px), upright (same orientation as the top-left small glyph — single-orientation digital play does not benefit from the traditional 180° inverted-corner indicator)
- Corner radius: 8px
- Suit differentiation: hearts and spades have **filled** glyphs; diamonds and clubs have **outlined** glyphs (1.5px stroke)

### Card Back ("Terminal" theme)

- Theme name: `"Terminal"`
- Author: `"Rusty Solitaire"`
- Background: `#151515`
- Pattern: horizontal scanlines at 2px pitch in `#1a1a1a` (1px line, 1px gap), full bleed
- Border: 1px solid `#353535`
- Top-left badge: a 12×16px solid `#a54242` block (the "terminal cursor"), 6px from the corner
- Bottom-right monogram: the characters `▌RS` in JetBrains Mono 12px, color `#505050`, 6px from the corner
- Corner radius: 8px (matches face)

### Primary Buttons

Solid `#a54242` fill, `#151515` text, JetBrains Mono Medium 14px uppercase with `+0.08em` tracking. 4px corner radius. Pressed state: darken to `#7a3030`. Disabled: `#353535` fill, `#505050` text.

### Secondary Buttons

Transparent fill, 1px `#505050` border, `#d0d0d0` text. Hover/press: border becomes `#a54242`, text becomes `#a54242`.

### HUD Chips

`#202020` fill, no border, 4px radius. Monospaced 16px text. Score chip pulses to `#acc267` for 200ms when score increases.

### Drag Targets

When a card is being dragged over a valid pile, the pile's empty-slot dashed outline becomes:
- Solid 1px in `#acc267`
- Plus a 0 0 8px outer glow in `#acc267` at 30% opacity

This is the *only* place glow effects appear in the system.

### Modals

Full-screen backdrop at 95% opacity. Centered panel: `#202020` fill, 1px `#505050` border, 12px corner radius. Title bar shows the screen name in monospaced 14px, color `#a0a0a0`, with a single `▌` cursor character prefix to reinforce the terminal pane motif.

### Navigation Bar

Fixed at the bottom of in-game screens. Height: 64px. `#202020` fill, 1px top border in `#353535`. Four icon buttons: Undo / Hint / New / Auto-complete. Icons: 24px, 1.5px stroke weight, color `#d0d0d0`. Active/pressed: icon color `#a54242`.

### Status / Sync Indicator

Top-right corner of the HUD: a 6px circular dot.
- Connected & synced: `#12cfc0`
- Pending: `#ddb26f` (pulsing 1.5s)
- Error: `#fb9fb1` (steady)
- Offline: `#505050`

## Accessibility

1. **Color-blind mode** (Settings → Gameplay): swaps the red suits' default `#e35353` for `#acc267` (lime). Outlined-glyph differentiation remains active in *all* modes.
2. **High-contrast mode** (Settings → Gameplay): boosts on-surface from `#d0d0d0` to `#f5f5f5`, outline from `#505050` to `#a0a0a0`, suit-red from `#fb9fb1` to `#ff8aa0`.
3. **Reduce-motion mode** (Settings → Gameplay): disables card-lift transition (instant z-lift), disables CRT scanline effect, disables the warning-chip pulse animation.
4. **Tabular figures** are mandatory for any number that updates live (timer, score, moves) so they don't reflow.
5. **Touch targets** are 48dp minimum even when the visual element is smaller.
6. **Text contrast**: all body text on background passes WCAG AA at minimum (`#d0d0d0` on `#151515` = 9.5:1; `#a0a0a0` on `#151515` = 5.7:1).
