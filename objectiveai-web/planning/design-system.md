# ObjectiveAI Design System

**Canonical reference for all visual design decisions.**
Established 2026-04-01. Supersedes `design-guidelines.md` and all prior color/type references.

---

## Design Direction

Dark, authoritative, brutalist-influenced. Infrastructure aesthetic, not consumer.
"Supreme Court energy but not scary." Warm carbon canvas with copper accents — like aged hardware, tactile instruments, printed technical manuals.

Grain as a medium, not a filter. Monospace where precision matters, grotesk where readability matters.

---

## Color System

### Warm Carbon Neutrals

Near-blacks and near-whites with a warm undertone. Never pure `#000000` or `#ffffff`.

| Token | Hex | Usage |
|-------|-----|-------|
| `--bg` | `#0d0b09` | Page background |
| `--bg-raised` | `#151310` | Cards, raised surfaces, inputs |
| `--bg-code` | `#110f0c` | Terminal blocks, code backgrounds |
| `--border` | `#221f1a` | Subtle dividers, card borders |
| `--border-bright` | `#2e2a25` | More visible borders, terminal dots |
| `--border-brightest` | `#3a3530` | Hover states on borders |
| `--text` | `#e8e2da` | Primary body text |
| `--text-dim` | `#958c82` | Secondary text, descriptions, prompts |
| `--text-muted` | `#5a534a` | Labels, captions, comments, lowest emphasis |
| `--text-heading` | `#f7f2eb` | Headings, strong emphasis (warm white, not pure) |

### Dusty Copper Accent

Single accent color used sparingly: links, focus states, CTAs, status indicators, section labels.

| Token | Value | Usage |
|-------|-------|-------|
| `--accent` | `#c0a090` | Primary accent — links, badges, active states |
| `--accent-hover` | `#d0b0a0` | Hover state |
| `--accent-subtle` | `rgba(192,160,144,0.10)` | Badge backgrounds, card tags, input focus glow |
| `--accent-muted` | `#786050` | Disabled or very low-emphasis accent |

### Score Colors (Functional — Unchanged)

These are data visualization colors, not brand colors. They remain vivid and saturated to communicate score confidence levels. Never use these for UI chrome.

| Token | Value | Condition |
|-------|-------|-----------|
| `--score-high` | `rgb(34, 197, 94)` | Score >= 66% |
| `--score-mid` | `rgb(234, 179, 8)` | Score >= 33% |
| `--score-low` | `rgb(249, 115, 22)` | Score >= 15% |
| `--score-critical` | `rgb(239, 68, 68)` | Score < 15% |

### Logo Colors

Current logo assets use `#1b1b1b` (dark) and `#eeeeee` (light).
Warm-shifted equivalents for future logo refresh: `#1a1815` and `#eee8e0`.

### Retired Colors — Do Not Use

| Color | Reason |
|-------|--------|
| `#6B5CFF` (purple) | v1 accent, retired |
| `#271884` (deep purple) | v1 brand, retired |
| `#3DF2E1` (cyan) | v1 accent, retired |
| `#3fb950` (green) | v1.5 landing accent, replaced by copper |
| `#000000` (pure black) | Use warm carbon `#0d0b09` instead |
| `#ffffff` (pure white) | Use `#f7f2eb` instead |

---

## Typography

### Font Stack

| Role | Font | Fallback | Weight Range |
|------|------|----------|-------------|
| **Body** | Space Grotesk | system-ui, -apple-system, sans-serif | 400, 500, 600, 700 |
| **Monospace** | JetBrains Mono | Geist Mono, ui-monospace, monospace | 400, 500, 600 |

### Type Scale

| Token | Size | Usage |
|-------|------|-------|
| `--text-xs` | 10px | Swatch labels, fine print |
| `--text-sm` | 12px | Card descriptions, small body, code |
| `--text-base` | 13px | Default body text |
| `--text-md` | 14px | Larger body, step descriptions |
| `--text-lg` | 15px | Hero sub-copy |

### Heading Scale

| Level | Desktop | Mobile | Weight | Font |
|-------|---------|--------|--------|------|
| Hero h1 | `clamp(38px, 5.5vw, 56px)` | auto via clamp | 700 | Space Grotesk |
| h1 | 48px | 32px | 700 | Space Grotesk |
| h2 | 32px | 24px | 700 | Space Grotesk |
| h3 | 20px | 18px | 600 | Space Grotesk |
| Section label | 10-11px | same | 600 | JetBrains Mono, uppercase, tracking 0.08em |

### Monospace Usage

JetBrains Mono is used for:
- Terminal/code blocks
- Section labels (uppercase, tracked)
- Badges and status indicators
- Score values
- Inline `code` references
- The `{ai}` logo mark

Space Grotesk is used for everything else: headings, body copy, buttons, form labels.

---

## Spacing

| Token | Value |
|-------|-------|
| `--space-1` | 4px |
| `--space-2` | 8px |
| `--space-3` | 12px |
| `--space-4` | 16px |
| `--space-6` | 24px |
| `--space-8` | 32px |
| `--space-12` | 48px |

---

## Border Radius

Infrastructure-tight. No pills, no squircles, no organic curves.

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 3-4px | Small elements: tags, inline badges |
| `--radius-md` | 5-6px | Cards, terminals, inputs, buttons |
| `--radius-lg` | 6-8px | Large containers, comparison blocks |

Do **not** use radius > 8px on any element. The old 12-20px pill/card radii are retired.

---

## Component Patterns

### Buttons

```
Primary:    bg: var(--accent), color: var(--bg), radius: 6px, weight: 600
Ghost:      bg: transparent, border: 1px solid var(--accent), color: var(--accent)
Hover:      primary bg shifts to var(--accent-hover)
```

### Cards

```
bg: var(--bg-raised)
border: 1px solid var(--border)
radius: 6px
padding: 16-20px
tag: JetBrains Mono, 9-10px, bg var(--accent-subtle), color var(--accent)
```

### Terminal Blocks

```
Container: bg var(--bg-code), border 1px solid var(--border), radius 6px
Bar:       border-bottom 1px solid var(--border), 3 dots at var(--border-bright)
Body:      JetBrains Mono, 12px, line-height 1.8
Comment:   color var(--text-muted)
Prompt $:  color var(--text-dim)
Command:   color var(--text)
```

### Badges

```
bg: var(--accent-subtle)
color: var(--accent)
font: JetBrains Mono, 10-11px, weight 500
radius: 4px
Pulsing dot: var(--accent), 5-6px, animation 2s ease-in-out infinite
```

### Inputs

```
bg: var(--bg-raised)
border: 1px solid var(--border)
radius: 6px
color: var(--text)
placeholder: inherited (browser default muted)
Focus: border-color var(--accent), box-shadow 0 0 0 2px var(--accent-subtle)
```

### Score Bars

```
Track: var(--border), height 3px, radius 2px
Fill:  score color (functional), proportional width
Label: JetBrains Mono, 10px, var(--text-dim)
Value: JetBrains Mono, 10px, var(--text-muted), right-aligned
```

### Section Labels

```
font: JetBrains Mono
size: 10-11px
weight: 600
transform: uppercase
tracking: 0.08em
color: var(--accent)
```

### Links

```
color: var(--accent)
border-bottom: 1px solid rgba(192,160,144,0.3)
weight: 500
hover: opacity 0.8 or border-color solid
```

---

## Layout

| Token | Value | Usage |
|-------|-------|-------|
| `.container` | max-width: 1100px | Standard content width |
| `.container-wide` | max-width: 1400px | Browse grids, wide layouts |
| `.container-narrow` | max-width: 780px | Landing page, reading width |

---

## Responsive Breakpoints

| Name | Value | Usage |
|------|-------|-------|
| Mobile | 640px | Stack layouts, reduce font sizes |
| Tablet | 768px | Landing page mobile stack |
| Desktop | 1024px | Full desktop layouts |
| Safety | 375px | Prevent overflow on smallest phones |

---

## Animation

| Token | Value | Usage |
|-------|-------|-------|
| `--transition-fast` | 0.15s | Hover color, opacity |
| `--transition-normal` | 0.2s | Border color, box-shadow, focus |
| Badge pulse | 2s ease-in-out infinite | Status dot opacity 1 → 0.3 |

Functional motion only. No decorative animation, no parallax, no scroll-triggered transitions except simple fade-in reveals.

---

## Hard Rules

1. **No Tailwind.** All styles go in `globals.css`.
2. **No shadcn/ui.** No component libraries.
3. **No inline styles** except dynamic values (widths, positions).
4. **No separate CSS files.** Everything in one `globals.css`.
5. **No new npm packages** without clear necessity.
6. **No pure black or pure white.** Use the warm carbon scale.
7. **No radius > 8px.** Infrastructure aesthetic, not consumer.
8. **No purple.** `#6B5CFF` and `#271884` are retired permanently.
9. **Score colors are functional only.** Never use green/yellow/orange/red for UI chrome.
10. **JetBrains Mono for all monospace.** Geist Mono is fallback only.

---

## Migration Notes

- The landing page (`.landing` scope in globals.css) will be the first page migrated to this system.
- Other pages migrate per-touch, not bulk swap.
- Legacy CSS variables in `:root` (`--page-bg`, `--card-bg`, `--accent`, etc.) remain until their pages are migrated.
- `design-tokens.css` is a legacy file for the function tree canvas. Do not use for page-level work.
