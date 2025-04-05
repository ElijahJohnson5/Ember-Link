# @ember-link/core

## 0.0.11

### Patch Changes

- 3ffbcd9: Add support for YJS provider package, fix small bug in presence, fix leaking setIntervals in others and presence. Borrow channels instead of always creating a new one
- 3ffbcd9: Update Storage to have iterators for map methods
  Update yjs storage to implement the new iterators
  Update core to be able to tell if storage is specified
  Update svelte package for convinence to use with storage methods
- Updated dependencies [3ffbcd9]
- Updated dependencies [3ffbcd9]
  - @ember-link/storage@0.0.9
  - @ember-link/protocol@0.0.9

## 0.0.10

### Patch Changes

- 870b2ae: Remove stray console log and expose others and presence as getters on channel
- a9bee5f: Add an autoconnect option and some small type fixes

## 0.0.9

### Patch Changes

- 54fa001: Update types for ember client to be correctly generic over custom message as well
- 368df3b: Add new custom message type, enable broadcast with api to server
- de13924: Fix status event emitter so consumers can actually get the initial statuses
- Updated dependencies [368df3b]
  - @ember-link/protocol@0.0.8
  - @ember-link/storage@0.0.8

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
