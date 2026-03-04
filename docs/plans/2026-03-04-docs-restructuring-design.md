# Documentation Restructuring — Design

**Date:** 2026-03-04
**Status:** Approved

## Goal

Consolidate scattered documentation (requirements.md, 4 plan files, README bloat) into a clean structure with bidirectional traceability between requirements and architecture.

## File Changes

| Action | File | What happens |
|--------|------|-------------|
| Create | `docs/product-requirements.md` | Merge `requirements.md` + extract lasting requirements from plan files |
| Create | `docs/architecture.md` | System design from CLAUDE.md architecture sections + design docs |
| Rewrite | `README.md` | Slim to: pitch, quick start, documentation index (~40 lines) |
| Edit | `CLAUDE.md` | Keep dev conventions/commands/gotchas. Remove architecture prose (link to architecture.md). Add traceability section. Update repo structure. |
| Delete | `requirements.md` | Moved to `docs/product-requirements.md` |
| Delete | `docs/plans/2026-03-04-historical-data-overlay-design.md` | Absorbed into architecture.md + product-requirements.md |
| Delete | `docs/plans/2026-03-04-historical-data-overlay-plan.md` | Implementation complete |
| Delete | `docs/plans/2026-03-04-parameter-explanation-design.md` | Absorbed into architecture.md + product-requirements.md |
| Delete | `docs/plans/2026-03-04-parameter-explanation-plan.md` | Implementation complete |

## Traceability Model

Bidirectional links between requirements and architecture:

- **product-requirements.md**: Each REQ gets a `Components:` field listing architecture elements it maps to.
- **architecture.md**: Each component/subsystem lists the REQs it implements with `Implements: REQ-NNN` annotations.

Example requirement:
```markdown
- [x] **REQ-008: WebSocket streaming simulation**
  - *Context:* Real-time parameter adjustment requires streaming simulation steps.
  - *Components:* `world3-api` WebSocket handler, `frontend` WS client, simulation store
  - *Done:* WebSocket handler with mpsc channel, 50ms debounce, auto-reconnect.
```

Example architecture section:
```markdown
### WebSocket Streaming
Implements: REQ-008, REQ-023

Sessions stream simulation steps via mpsc channel...
```

## CLAUDE.md Traceability Section

Short section after "Current Objective" enforcing the workflow:

```markdown
## Requirements & Architecture Traceability

- Product requirements with stable IDs: `docs/product-requirements.md`
- System architecture and component design: `docs/architecture.md`

**When to update these docs:**
- Adding a new feature → create a new REQ-NNN in product-requirements.md first
- Changing system design (new crate, new API endpoint, new store) → update architecture.md
- Completing a REQ → mark it done and verify Components field is accurate
- Writing a design doc in docs/plans/ → reference the REQ-NNN it addresses
```

## architecture.md Structure

1. **Overview** — 1-paragraph system description + stack
2. **Component Map** — table: component → purpose → crate/directory → REQs
3. **Simulation Engine (world3-core)** — WorldState, sectors, solver, lookup tables. Implements: REQ-001, REQ-002, REQ-005
4. **API Server (world3-api)** — REST, WebSocket, historical data, AppState. Implements: REQ-007, REQ-008, REQ-012
5. **Frontend (SvelteKit + D3)** — Env config, stores, charts, content system, panels. Implements: REQ-009, REQ-020–025
6. **Data Pipeline (world3-ingestion)** — DataSource trait, fallback chain, mapping. Implements: REQ-013, REQ-014
7. **Historical Data Overlay** — CSV format, backend parser, API, rendering. Implements: REQ-012
8. **Parameter Explanation Panels** — Extended ParameterInfo, stores, panel, highlighting. Implements: REQ-020 (extension)
9. **CLI (world3-cli)** — Implements: REQ-003, REQ-004, REQ-006
10. **Deployment** — Implements: REQ-016

Each section covers: purpose, key design decisions, data flow, REQ traceability.

## README.md Target

~40 lines: project pitch (3-4 sentences), quick start (5 lines), documentation index table, license.

## Out of Scope

- No content changes to existing docs (model-guide.md, api-server.md, etc.)
- No code changes
- No new requirements — just reorganizing existing ones
