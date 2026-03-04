# Parameter Explanation Panel — Design

## Goal

Surface rich, educational parameter explanations in the UI so users understand what each simulation slider controls, how it affects the model, and can see the impact visually.

## Scope

- Dedicated `ParameterInfoPanel` component (side panel, same style as `VariableInfoPanel`)
- Extended `ParameterInfo` data model with feedback loops, related variables, impact text
- Mini sparkline chart showing BAU vs. current scenario for the most relevant variable
- Chart highlighting: dim unrelated lines when a parameter panel is open
- Expose all 13 parameters as sliders (add 3 currently hidden ones to backend `parameter_descriptors()`)
- Info icon on each slider to open the panel

## Data Model

### Extended `ParameterInfo` interface

```ts
export interface ParameterInfo {
    name: string;
    unit: string;
    sector: string;
    beginner: string;         // plain-English explanation
    expert: string;           // equation-level / technical detail
    feedbackLoops: string[];  // IDs into feedbackLoops registry
    relatedVariables: string[];  // field paths into variableDescriptions
    impact: {
        increase: string;     // what happens when you increase this parameter
        decrease: string;     // what happens when you decrease this parameter
        sparklineVariable: string;  // field path of the variable to plot
    };
}
```

### Content authoring

All 13 parameters get `feedbackLoops`, `relatedVariables`, and `impact` data. The 12 existing entries are extended; 1 new entry added for any missing parameter.

### Backend changes

Add 3 parameters to `parameter_descriptors()` in `crates/world3-core/src/model/params.rs`:
- `service_depreciation_rate`
- `subsistence_food_per_capita`
- `initial_nnr_fraction`

## Stores

### New: `selectedParameterId`

```ts
// stores/info.ts
export const selectedParameterId = writable<string | null>(null);
```

Mutually exclusive with `selectedVariableId` — setting one clears the other.

### New: `highlightedVariables`

```ts
// stores/info.ts
export const highlightedVariables = derived(
    selectedParameterId,
    ($paramId) => {
        if (!$paramId) return new Set<string>();
        const info = parameterDescriptions[$paramId];
        return new Set(info?.relatedVariables ?? []);
    }
);
```

## UI Components

### `ParameterSlider.svelte` — info icon

Add `[i]` button next to label. Click sets `selectedParameterId`.

```
[i] [Label name]            [value] [unit]
[————————————————range input————————————————] [↺]
```

### `ParameterInfoPanel.svelte` — new component

340px fixed-right panel, slide-in animation, backdrop click to close. Structure:

```
panel-backdrop (click -> close)
info-panel
  panel-header
    h2: parameter name
    .meta: sector + unit
    close button
  panel-body
    section: beginner description
    section: toggle "Technical Detail" -> expert text
    section: "Impact"
      impact-card "Increase": impact.increase text
      impact-card "Decrease": impact.decrease text
      sparkline: mini D3 chart (300x80px)
        BAU line (dimmed) vs current scenario line (colored)
        Reads from simulationResults store (no extra API calls)
    section: "Feedback Loops" (if feedbackLoops.length > 0)
      loop-cards with type badge and description
    section: "Related Variables" (if relatedVariables.length > 0)
      clickable buttons -> switch to VariableInfoPanel
```

### Chart highlighting

When `highlightedVariables` is non-empty:
- Lines whose field path is in the set: full opacity, slightly thicker stroke
- All other lines: opacity 0.15
- When empty: all lines render at normal opacity

## Interaction Flow

1. User clicks `[i]` on a slider
2. `selectedParameterId` set, `selectedVariableId` cleared
3. `ParameterInfoPanel` slides in from right
4. `highlightedVariables` computed from parameter's `relatedVariables`
5. Chart dims unrelated lines, highlights related ones
6. Sparkline shows BAU vs current scenario for the `sparklineVariable`
7. User can click a related variable button -> switches to `VariableInfoPanel`
8. User clicks backdrop, presses Escape, or clicks another `[i]` -> panel closes/switches

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/content/variable-descriptions.ts` | Extend `ParameterInfo` interface, add `feedbackLoops`/`relatedVariables`/`impact` to all 13 entries |
| `frontend/src/lib/stores/info.ts` | Add `selectedParameterId`, `highlightedVariables` stores |
| `frontend/src/components/ParameterSlider.svelte` | Add info icon button |
| `frontend/src/components/ParameterInfoPanel.svelte` | New component |
| `frontend/src/components/ParameterSliders.svelte` | Import and render `ParameterInfoPanel` |
| `frontend/src/components/VariableInfoPanel.svelte` | Clear `selectedParameterId` when `selectedVariableId` is set |
| Chart components | Read `$highlightedVariables` for line dimming |
| `crates/world3-core/src/model/params.rs` | Add 3 parameters to `parameter_descriptors()` |

## Out of Scope

- Simulation controls (start/end year, time step) do not get explanation panels
- No new API endpoints — sparkline uses existing `simulationResults` store data
- No mobile-specific layout changes (existing responsive behavior applies)
