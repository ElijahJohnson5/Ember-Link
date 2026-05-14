# **SDKs**

Ember Link is published as a set of focused packages on npm. This page lists every package the project ships, explains the relationship between them, and links to the dedicated reference page for each.

## **Client Packages**

These packages run in the browser and connect to a running Ember Link server. Most applications install exactly one of them.

### **[@ember-link/core](/packages/core)**

The foundational package that powers every framework integration. `@ember-link/core` provides the `Client` and `Channel` abstractions, manages the WebSocket connection, handles authentication, and exposes the event subscription API. Reach for `@ember-link/core` directly when you are building a custom framework integration or when you are using plain JavaScript or TypeScript without a UI framework.

### **[@ember-link/react](/packages/react)**

The official React integration. `@ember-link/react` re-exports everything from `@ember-link/core` and adds `EmberLinkProvider`, `ChannelProvider`, and hooks (`useChannel`, `useOthers`, `useMyPresence`, `useStatus`, `useCustomMessage`, `useArrayStorage`, `useMapStorage`) that handle subscription, cleanup, and reactivity for you.

### **[@ember-link/svelte](/packages/svelte)**

The official Svelte integration. `@ember-link/svelte` re-exports everything from `@ember-link/core` and adds `EmberLinkProvider`, `ChannelProvider`, and `getChannelContext()`. The `SvelteChannel` returned by `getChannelContext()` wraps the core channel in `$state`-backed reactive properties so values can be interpolated directly into a Svelte template.

## **Storage Packages**

These packages add a shared CRDT document to a channel. Storage is opt-in.

### **[@ember-link/storage](/packages/storage)**

The base interfaces and types for collaborative storage. `@ember-link/storage` defines `IStorageProvider`, `IStorage`, `ArrayStorage`, `MapStorage`, and the `StorageEvent` shape. The package is consumed by both the framework integrations and the storage backends, and is published separately so that custom storage backends can implement it without depending on the rest of the SDK.

### **[@ember-link/yjs-storage](/packages/yjs-storage)**

A storage backend built on Yjs. Call `createYJSStorageProvider()` and pass the result as the `storageProvider` option on `joinChannel` to enable shared `ArrayStorage` and `MapStorage` on the channel.

### **[@ember-link/yjs-provider](/packages/yjs-provider)**

A Yjs `Doc` provider for applications that need to integrate a Yjs-aware editor (Tiptap, BlockNote, ProseMirror, and so on). Call `getYjsProviderForChannel(channel)` to obtain a memoized `EmberLinkYjsProvider` bound to that channel. The provider exposes the underlying `Y.Doc` so editor extensions that already speak Yjs can sync directly.

## **Protocol Package**

### **[@ember-link/protocol](/packages/protocol)**

The wire-protocol definitions shared between the client SDK and the Rust server. `@ember-link/protocol` is consumed internally by `@ember-link/core`. Most applications do not need to import it directly. It is published primarily so that authors of custom server implementations or non-JS clients can match the on-wire schema.

## **Building a Custom Integration**

Bindings for other frameworks (Vue, Solid, Qwik, and so on) can be built on top of `@ember-link/core` directly. The two existing framework integrations (`@ember-link/react` and `@ember-link/svelte`) are good references for the shape of a binding.

## **Need Help?**

- The [Getting Started](/getting-started) guide walks through the first end-to-end client.
- The [Concepts](/concepts) section documents the primitives that the SDKs are built around.
- The Ember Link community on [Discord](https://discord.gg/YU2wGQtgE7) is the fastest way to ask a question.
