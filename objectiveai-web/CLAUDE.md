# ObjectiveAI Web (`objectiveai-web`)

Next.js / React App Router. Maya's primary workspace.

---

## Visual Direction

Dark, authoritative, brutalist-influenced. Infrastructure aesthetic, not consumer. "Supreme Court energy but not scary." Warm carbon + dusty copper palette — serious but not cold. Grain as a medium, not a filter.

**The landing page is the reference implementation.** When building or updating any page, match the landing page's palette, typography feel, and spacing — not the older site-wide CSS variables. Landing page uses agent/swarm terminology throughout.

---

## Design System

### Landing Page Palette (the canonical direction)

The landing page scopes its own vars on `.landing` in `globals.css`. These represent where we're going:

| Variable | Value | Usage |
|----------|-------|-------|
| `--l-bg` | `#0d0b09` | Page background — warm carbon |
| `--l-bg-raised` | `#151310` | Cards, raised surfaces |
| `--l-bg-code` | `#110f0c` | Terminal blocks, code backgrounds |
| `--l-border` | `#221f1a` | Subtle dividers |
| `--l-border-bright` | `#2e2a25` | More visible borders |
| `--l-text` | `#e8e2da` | Primary body text |
| `--l-text-dim` | `#958c82` | Secondary text, descriptions |
| `--l-text-muted` | `#5a534a` | Labels, captions, lowest emphasis |
| `--l-text-heading` | `#f7f2eb` | Headings, emphasis, strong |
| `--l-accent` | `#c0a090` | Accent — dusty copper (status, links, highlights) |

### Site-Wide CSS Variables (legacy, still used by browse/detail pages)

| Variable | Value | Notes |
|----------|-------|-------|
| `--page-bg` | `#1B1B1B` | Lighter than landing — will migrate to `#0a0a0a` |
| `--text` | `#EDEDF2` | |
| `--text-muted` | `#999999` | |
| `--card-bg` | `#252525` | |
| `--accent` | `#EDEDF2` | Monochrome accent |
| `--border` | `rgba(237, 237, 242, 0.1)` | |

As pages get touched, migrate them toward the landing palette. Don't do a bulk swap — update per-page as work happens.

### Score Colors (unchanged, used everywhere)

| Variable | Value | Usage |
|----------|-------|-------|
| `--color-success` | `rgb(34, 197, 94)` | Scores ≥ 66% |
| `--color-warning` | `rgb(234, 179, 8)` | Scores ≥ 33% |
| `--color-danger` | `rgb(249, 115, 22)` | Scores ≥ 15% |
| `--color-error` | `rgb(239, 68, 68)` | Scores < 15% |

### Retired — Do Not Use in New Work

- Purple (`#6B5CFF`) — was v1 accent. Still in `design-tokens.css`. Do not propagate.
- Green (`#3fb950`) — was landing page accent (GitHub green). Replaced by copper `#c0a090`.
- `design-tokens.css` light theme vars — overridden by `globals.css`, ignore them.

### Typography

| Context | Font | CSS Variable |
|---------|------|-------------|
| Body | Space Grotesk | `--font-space-grotesk` |
| Monospace | JetBrains Mono | `--font-jetbrains-mono` |

Both fonts loaded via `next/font/google` in `layout.tsx`. Migrate remaining pages component-by-component as they're touched — don't bulk swap.

### Spacing & Shape

- Landing page uses `6px`–`8px` border-radius on cards/terminals, `3px`–`4px` on small elements (code tags, copy buttons, score bars)
- Older site-wide vars (`--radius-sm: 8px`, `--radius-md: 12px`) are larger — new work should trend smaller and tighter
- Landing sections: generous vertical spacing (80–100px padding-bottom)

---

## CSS Rules

**ALL styles go in `globals.css`.** No separate CSS files per component.

**Class naming:** `.pillBtn`, `.card`, `.tag`, `.site-nav`, `.landing-*`, `.promptBlock*`

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
