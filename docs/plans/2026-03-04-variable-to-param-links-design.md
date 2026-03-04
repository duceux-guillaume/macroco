# Design: Related Parameters in Variable Info Panel

## Problem
ParameterInfoPanel links to variables via `relatedVariables`, but VariableInfoPanel has no reverse link to show which parameters affect a given variable.

## Solution
Auto-derive related parameters by inverting the existing `parameterDescriptions[*].relatedVariables` mapping. Display them in VariableInfoPanel using the existing `RelatedVars` pill-button component.

## Changes

### 1. New utility: `getRelatedParameters(variablePath)`
- Location: `frontend/src/lib/content/variable-descriptions.ts`
- Iterates all `parameterDescriptions` entries
- Returns `Array<{path: string; name: string}>` for params whose `relatedVariables` includes the variable path
- Memoized (static data, computed once per variable path)

### 2. VariableInfoPanel update
- Compute `relatedParams` via `$derived` using `getRelatedParameters(variableId)`
- Render a second `RelatedVars` component with heading "Related Parameters"
- `onselect` calls `selectedParameterId.set(key)` (not `selectedVariableId`)

### 3. No changes to
- `VariableInfo` interface (no new fields)
- `RelatedVars` component (already generic)
- `ParameterInfoPanel`, `InfoPanelShell`, `FeedbackLoops`
- Store mutual exclusion logic (already bidirectional)

## Data flow
```
VariableInfoPanel
  → $derived: relatedParams = getRelatedParameters(variableId)
  → <RelatedVars vars={relatedParams} onselect={selectParameter} />
      → click pill → selectedParameterId.set(key)
          → mutual exclusion clears selectedVariableId
          → ParameterInfoPanel opens
```
