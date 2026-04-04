# ObjectiveAI Web (`objectiveai-web`)

Next.js / React App Router. Maya's primary workspace.

---

## Visual Direction

**Copper Spectrum. Material Honesty. Every Element Load-Bearing.**

Dark, infrastructure-first. "Agentic collective judgment harness" energy. All colors derive from copper (~hue 20°). Backgrounds are copper cooled to charcoal. Text is copper lightened to cream. Borders are copper at low alpha. Nothing arbitrary.

No pills. No decorative borders. No shadows at rest on cards. Whitespace is the separator, not lines. Typography-driven hierarchy: JetBrains Mono for system/data/nav, Space Grotesk for headlines and body.

---

## Design System

### Copper Spectrum (unified palette — no split between landing and site)

| Variable | Value | Role |
|----------|-------|------|
| `--page-bg` | `#0b0908` | Copper cooled to near-black |
| `--card-bg` | `#131010` | Copper ash — cards, surfaces |
| `--bg-code` | `#0f0c0b` | Deeper than page — terminals, code |
| `--border` | `rgba(180,140,120, 0.07)` | Copper at 7% — ghost border |
| `--border-bright` | `rgba(180,140,120, 0.12)` | Copper at 12% |
| `--border-brightest` | `rgba(180,140,120, 0.22)` | Copper at 22% — focus/active |
| `--text` | `#ddd4c8` | Light copper — body |
| `--text-dim` | `#9a8d80` | Mid copper — secondary |
| `--text-muted` | `#5c5248` | Dark copper — labels |
| `--text-heading` | `#f0e8de` | Near-white copper — headings |
| `--accent` | `#c0a090` | Full copper — brand moments only |

### Key Principles

- **Cards are borderless.** Background shift (`--card-bg` on `--page-bg`) IS the container. No border at rest. Hover brightens to `#171312`.
- **Buttons use `box-shadow: inset` not `border`.** Filled and ghost buttons have identical dimensions. No optical size mismatch.
- **Radius: 2-4px.** `--radius-sm: 2px` for tags/chips, `--radius-md: 3px` for buttons/inputs, `--radius-lg: 4px` for cards, `--radius-xl: 6px` for terminals.
- **No pills anywhere.** No `border-radius: 50px`, no `border-radius: 50%` on icon buttons. Square icon buttons with 4px radius.
- **No divider lines.** Sections separated by whitespace (72-100px). No `border-top`, no `<hr>`.
- **Copper reserved for brand moments.** CTAs, terminal prompt, status badge, active states. Not on every interactive element.

### Score Colors (functional, unchanged)

| Variable | Value | Usage |
|----------|-------|-------|
| `--color-success` | `rgb(34, 197, 94)` | Scores ≥ 66% |
| `--color-warning` | `rgb(234, 179, 8)` | Scores ≥ 33% |
| `--color-danger` | `rgb(249, 115, 22)` | Scores ≥ 15% |
| `--color-error` | `rgb(239, 68, 68)` | Scores < 15% |

### Retired — Do Not Use

- Purple (`#6B5CFF`) — v1 accent. Dead.
- Green (`#3fb950`) — v2 accent. Dead.
- Flat hex borders (`#221f1a`, `#2e2a25`) — replaced by copper-at-alpha.
- `border-radius: 50px` / `50%` — no pills.
- `design-tokens.css` light theme vars — overridden, ignore.

### Typography

Two voices, one system:

| Voice | Font | Usage |
|-------|------|-------|
| System | JetBrains Mono | Nav links, card titles, tags, labels, metadata, buttons, terminal, section labels |
| Human | Space Grotesk | Headlines, body paragraphs, descriptions |

Both loaded via `next/font/google` in `layout.tsx`.

### Spacing & Shape

- Radius scale: `2px` (tags) → `3px` (buttons, inputs) → `4px` (cards, icon buttons) → `6px` (terminals)
- Section spacing: 80-100px vertical padding. Whitespace IS the separator.
- Button sizing: `box-shadow: inset 0 0 0 1px` for all borders — prevents optical size mismatch between filled and ghost variants

---

## CSS Rules

**ALL styles go in `globals.css`.** No separate CSS files per component.

**Class naming:** `.pillBtn` (primary button), `.pillBtnGhost` (ghost button), `.card`, `.tag`, `.filterChip`, `.iconBtn`, `.site-nav`, `.landing-*`, `.promptBlock*`

### Hard Rules
- **NO Tailwind.** The project does not use Tailwind.
- **NO shadcn/ui** or any component library.
- **NO inline styles** except for truly dynamic values (computed widths, positions).
- **NO separate CSS files** — everything in `globals.css`.
- **NO new npm packages** unless absolutely necessary.

### Note: `design-tokens.css`

`lib/design-tokens.css` is a legacy file for the function tree's responsive canvas system. It contains light theme vars and purple accent that are overridden. Do not use its color/theme values for page-level work.

---

## Container Layout

Use `.container` (1100px max) or `.containerWide` (1400px max) for page content wrappers. Never inline `maxWidth` or padding overrides.

The landing page is an exception — it uses its own `max-width: 780px` on `.landing-hero` and `.landing-section` directly, not the `.container` class. This is intentional (narrower, more focused reading width).

`.containerWide` is only for browse pages (functions, profiles, ensembles, ensemble-llms).

---

## Landing Page

### Structure
```
Nav
→ Hero: badge ("API + CLI live") + descriptor ("Your agent's advisory board.") + headline ("Your agent doesn't have to decide alone.") + subtitle + CTA buttons + proof point + terminal + email form
→ The Problem: logprobs insight block with comparison + score bars
→ How It Works: three numbered steps
→ Use Cases: 2x2 grid
→ Browse: link to /functions
→ Bottom CTA: compact PromptBlock
```

### Key Components
- **PromptBlock** (`components/PromptBlock.tsx`) — Terminal-style CLI install with copy button. Default and compact variants.
- **Terminal block** — Custom `.landing-terminal` with dot bar. Badge says "API + CLI live."
- **Email form** — Buttondown integration for CLI launch notifications.

### What It Does NOT Have
- Sign-up / login buttons (it does have "Sign up free" CTA linking to NextAuth)
- Feature bullets or "why choose us"
- Testimonials, social proof, pricing tiers
- Decorative gradients, glows, color splashes

### Animation
- Scroll-driven fade-ins via IntersectionObserver (no libraries)
- Copy button: `scale(1.05)` + copper border flash, CSS transition only
- Status badge: subtle copper pulse animation
- No bouncing, parallax, or particle effects

### Mobile
- 768px breakpoint for layout stacking
- Test at 375px — nothing should overflow

---

## Client-Side SDK Pattern

No server-side API routes except NextAuth. All data fetching uses the JS SDK:

**Public (no auth):**
```tsx
import { createPublicClient } from "@/lib/client";
const client = createPublicClient();
const functions = await Functions.list(client);
```

**Auth-required:**
```tsx
const { getClient } = useObjectiveAI();
const client = await getClient();
```

**Stripe** is the only exception — uses `fetch` directly.

Key files: `lib/client.ts`, `hooks/useObjectiveAI.ts`, `lib/provider.ts`

---

## Authentication

OAuth (Google, GitHub, X, Reddit) via NextAuth.

- Anonymous users get ~5¢ free credit
- CORS: `Access-Control-Allow-Origin: *`
- No user tiers
- Key files: `app/api/auth/[...nextauth]/route.ts`, `lib/provider.ts`, `contexts/AuthContext.tsx`

---

## Browse Pages

All browse pages follow the same pattern:
- Filter toggle left of search bar
- Collapsible sidebar (desktop) / bottom sheet (mobile)
- Load more pagination, responsive grid
- SSR/ISR with `revalidate = 120` and `unstable_cache`
- Reference: `app/functions/page.tsx`

---

## Planning Assets

Check `objectiveai-web/planning/` before design decisions — moodboard, color system, wireframes, logo assets, design guidelines, and CTA strategy docs. `planning/design-system.md` is the canonical design system reference.

---

## Navigation

```
Functions → Browse, Profiles
Ensembles → Browse, LLMs
Information → Team, Docs, Legal
```

Note: Docs sidebar still shows old terminology (maps to current API endpoints). Will update when Ronald renames endpoints.
