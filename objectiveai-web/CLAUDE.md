# ObjectiveAI Web (`objectiveai-web`)

Next.js / React App Router. Maya's primary workspace.

---

## Visual Direction

Dark, authoritative, brutalist-influenced. Infrastructure aesthetic, not consumer. "Supreme Court energy but not scary." Black and white as a statement of seriousness. Grain as a medium, not a filter.

**The landing page is the reference implementation.** When building or updating any page, match the landing page's palette, typography feel, and spacing — not the older site-wide CSS variables.

---

## Design System

### Landing Page Palette (the canonical direction)

The landing page scopes its own vars on `.landing` in `globals.css`. These represent where we're going:

| Variable | Value | Usage |
|----------|-------|-------|
| `--l-bg` | `#0a0a0a` | Page background — near-black |
| `--l-bg-raised` | `#111111` | Cards, raised surfaces |
| `--l-bg-code` | `#0d1117` | Terminal blocks, code backgrounds |
| `--l-border` | `#1e1e1e` | Subtle dividers |
| `--l-border-bright` | `#2a2a2a` | More visible borders |
| `--l-text` | `#e0e0e0` | Primary body text |
| `--l-text-dim` | `#6b6b6b` | Secondary text, descriptions |
| `--l-text-muted` | `#4a4a4a` | Labels, captions, lowest emphasis |
| `--l-green` | `#3fb950` | Status indicators, accent (GitHub green) |
| Headings / emphasis | `#ffffff` | Pure white for h1, h2, strong, commands |

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
- `design-tokens.css` light theme vars — overridden by `globals.css`, ignore them.

### Typography

| Context | What we use now | Direction |
|---------|----------------|-----------|
| Body | `system-ui` (set in `globals.css` body) | DM Sans |
| Monospace | `--font-geist-mono` (loaded in `layout.tsx`) | JetBrains Mono |

The landing page already uses `JetBrains Mono` in the PromptBlock and bottom CTA, with Geist Mono as fallback. Migrate component-by-component as pages are touched — don't bulk swap.

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
→ Hero: badge + descriptor + headline + subtitle + CTA buttons + proof point + terminal + email form
→ The Problem: logprobs insight block with comparison + score bars
→ How It Works: three numbered steps
→ Use Cases: 2x2 grid
→ Browse: link to /functions
→ Bottom CTA: compact PromptBlock
```

### Key Components
- **PromptBlock** (`components/PromptBlock.tsx`) — Terminal-style CLI install with copy button. Default and compact variants.
- **Terminal block** — Custom `.landing-terminal` with dot bar, controlled by `CLI_LIVE` flag.
- **Email form** — Buttondown integration for CLI launch notifications.

### What It Does NOT Have
- Sign-up / login buttons (it does have "Sign up free" CTA linking to NextAuth)
- Feature bullets or "why choose us"
- Testimonials, social proof, pricing tiers
- Decorative gradients, glows, color splashes

### Animation
- Scroll-driven fade-ins via IntersectionObserver (no libraries)
- Copy button: `scale(1.05)` + green border flash, CSS transition only
- Status badge: subtle green pulse animation
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

Check `objectiveai-web/planning/` before design decisions — moodboard, color system, wireframes, logo assets, design guidelines, and CTA strategy docs.

---

## Navigation

```
Functions → Browse, Profiles
Ensembles → Browse, LLMs
Information → Team, Docs, Legal
```
