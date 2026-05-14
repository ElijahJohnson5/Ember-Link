# **Type Augmentation**

Type augmentation lets you declare the shape of your `Presence` and `Custom` types once, globally, so that every API across every Ember Link SDK picks up those types automatically. This page documents the augmentation pattern, the per-call override mechanism, and the alternative for library authors who do not want to require their consumers to augment the global type.

## **The Augmentation Pattern**

Add the following `declare global` block to any `.ts` or `.d.ts` file in your project. Most projects place it in a file called `src/ember-link.d.ts`, or near the top of the module that calls `createClient`.

```typescript copyButton
declare global {
	interface EmberLink {
		Presence: {
			cursor: { x: number; y: number } | null;
			color: string;
		};

		Custom:
			| { kind: 'reaction'; emoji: string }
			| { kind: 'ping' };
	}
}

// The file must be treated as a module for the `declare global` block
// to take effect. An empty export below is enough.
export {};
```

After the augmentation is in place, every public Ember Link API picks up the declared types automatically.

```typescript
const [me, setMe] = useMyPresence();
//      ^? { cursor: { x: number; y: number } | null; color: string } | null

channel.events.subscribe('customMessage', (msg) => {
	if (msg.kind === 'reaction') {
		// `msg.emoji` is typed `string`.
	}
});
```

## **Recognised Keys**

Only `Presence` and `Custom` are read by the SDK today. The `EmberLink` interface uses a string index signature so additional keys can be added in future versions without breaking existing augmentations.

## **Per-Call Overrides**

A generic type argument may still be passed at any call site. The per-call generic replaces the augmented type for that single call rather than narrowing it, so it does not need to extend the augmented type. This is useful in tests and inside libraries that build on top of Ember Link.

```typescript
useMyPresence<{ typing: boolean }>();
```

## **Library Authors**

If you are publishing a library that wraps Ember Link and you do not want to require your consumers to augment the global `EmberLink` interface, use [`createEmberLinkContext<P, C>()`](/packages/react) instead. The factory returns a per-library `Provider`, `useClient`, `useChannel`, and matching hook set whose types are scoped to your library's `P` and `C` generics. This keeps the typing internal to your library and avoids any interaction with the global augmentation in the consumer's project.
