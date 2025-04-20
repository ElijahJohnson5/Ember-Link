# Ember Link Packages

Welcome to the **Ember Link SDK** a powerful toolkit for building real-time, collaborative apps using your favorite frontend framework. Each package in the SDK serves a specific purpose and is designed to integrate cleanly whether you're using **React**, **Svelte**, or building your own custom integration.

---

## Core Packages

These are the foundational packages that power the Ember Link ecosystem.

### [@ember-link/core](/packages/core)

Handles WebSocket connections, authentication, presence, and channel management. This is the low-level client used by all framework-specific wrappers.

- Lightweight and extensible
- Provides `Channel` and `Client` abstractions
- Used by both React and Svelte integrations

### [@ember-link/storage](/packages/storage)

Defines the reactive CRDT-based storage layer used in shared real-time data.

- Pluggable storage architecture
- Built-in `ArrayStorage` and `MapStorage`
- Designed for framework-level wrappers

---

## ⚛️ Framework Integrations

Use these packages to connect Ember Link into your frontend app using idiomatic patterns.

### [@ember-link/react](/packages/react)

The official **React** integration for Ember Link.

- Provides hooks like `useOthers()`, `useMyPresence()`, and `useCustomMessage()`
- Uses context providers (`EmberLinkProvider`, `ChannelProvider`) to wrap your app
- CRDT storage compatible via hooks or core access

### [@ember-link/svelte](/packages/svelte)

The official **Svelte** integration for Ember Link.

- Provides context-aware helpers like `getChannelContext()`
- Fully reactive access to presence, others, and CRDTs using `$state`
- Designed for idiomatic usage with minimal boilerplate

---

## Building Custom Adapters?

Want to add support for Vue, Solid, or a headless custom implementation? Start with [@ember-link/core](/packages/core).

> We’d love to feature your adapter in the community sectionPRs welcome!

---

## 📚 Need Help?

- [Quick Start Guide](/getting-started)
- [Join the Discord](https://discord.gg/YU2wGQtgE7) (← example)

---

Stay synced ✨  
— The Ember Link Team
