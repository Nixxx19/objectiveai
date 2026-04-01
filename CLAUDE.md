# ObjectiveAI

**Ensemble LLM scoring pipelines as an API.**

ObjectiveAI runs ensembles of LLMs that vote with weighted probabilities to produce confidence-scored outputs. Instead of asking one model for an answer, multiple models deliberate and converge into structured numeric scores.

**Tagline:** "Your agent's advisory board."

**API:** https://api.objective-ai.io
**Web:** https://objective-ai.io

---

## Team

| Person | Role | Domain |
|--------|------|--------|
| Maya Gore | Co-Founder & CPO | Product, web, creative direction, content, biz dev |
| Ronald Riggles III | Co-Founder & CEO | Backend, API, SDK, CLI, Rust, deployment |

- Discord is the primary channel (2,000-char limit, no Nitro)
- Frame messages collaboratively ("Claude and Maya were wondering...") not assertively
- Don't ask Ronald things determinable from the codebase
- Ronald controls deployment — no auto-deploy on push

---

## Repository Structure

```
objectiveai/
├── CLAUDE.md                       ← You are here
├── objectiveai-web/                # Next.js web platform (Maya's domain)
│   └── CLAUDE.md                   # Design system, brand, CSS, components
├── objectiveai-api/                # Rust API server (Ronald's domain)
├── objectiveai-rs/                 # Rust SDK (Ronald's domain)
├── objectiveai-js/                 # TypeScript SDK (npm: objectiveai)
├── objectiveai-cli/                # CLI agent (Ronald's domain)
├── objectiveai-rs-wasm-js/         # WASM bindings
├── objectiveai-scripts/            # Utility scripts (see its README.md)
└── coding-agent-scratch/           # Scratch folder for SDK testing
```

---

## Product Voice

**How to describe ObjectiveAI (in order of preference):**
1. Ensemble LLM scoring pipelines as an API
2. Multiple LLMs vote with weighted probabilities to produce confidence-scored outputs
3. Your agent's advisory board — collective judgment, not single-model guessing

**Never describe it as:** a chatbot, assistant, AI tool for end users, model router, or load balancer. Never say "AI-powered" — it IS AI infrastructure.

**Copy tone:** Direct, technical, confident. Infrastructure language, not marketing language. Think Stripe docs, not Jasper AI landing page. No exclamation marks. No "unlock the power of." No "supercharge your workflow."

---

## Core Concepts

**Ensemble LLM:** A fully-specified config of one upstream LLM (model, prompt, decoding params, output mode). Content-addressed via XXHash3-128.

**Ensemble:** A collection of Ensemble LLMs used together for voting. Immutable. Does NOT contain weights.

**Weights:** Execution-time parameters controlling each LLM's influence. External to Ensembles. Learnable via Profiles.

**Vector Completions:** The core primitive. Produces scores, not text. Each LLM votes, votes combine using weights, returns a normalized score vector.

**Functions:** Composable scoring pipelines. Data in → Score(s) out. Hosted on GitHub as `function.json`. There is no server-side creation — the API fetches from GitHub at execution time.

**Profiles:** Learned weights for Functions. GitHub-hosted as `profile.json`.

---

## Rules

### Branch & Deploy
- Maya works on `maya/web-v2`
- Deployment: Google Cloud Build → Docker → Cloud Run
- Ronald manually triggers builds — no auto-deploy on push
- Always commit and push — never leave work uncommitted

### npm
- **Always run npm commands from the workspace root**, not from package directories

### Code Changes
- When "standardizing" or "applying patterns," preserve existing functionality. Never remove features unless told to.
- After declaring multi-page work "complete," enumerate all affected pages and verify each was touched.

### Scope Boundaries
- Web work stays in `objectiveai-web/`
- Never modify files outside your working package unless told to
- Don't guess at Ronald's backend — ask rather than assume

### Merging from Main
Any merge from `main` warrants full review. Check for API/SDK/design system changes, test affected pages, verify no regressions.

### Testing
- No network-hitting tests — mock API responses or use local test data
