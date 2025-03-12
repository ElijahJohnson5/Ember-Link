# @ember-link/core

## 0.0.8

### Patch Changes

- a7317ee: Update building of core to bundle types into one index.d.ts and index.d.cts to fix types
- 2895bb1: Actually fix the typing issues when used in another package
- a7317ee: Expose socket events in the channel events interface

## 0.0.7

### Patch Changes

- 5401880: Fix type setup in package json and the way we build things so that ESM and CJS work at the same time
- Updated dependencies [5401880]
  - @ember-link/event-emitter@0.0.7
  - @ember-link/protocol@0.0.7
  - @ember-link/storage@0.0.7
