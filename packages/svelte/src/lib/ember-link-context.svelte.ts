import { getContext, hasContext, setContext } from 'svelte';
import {
	createClient,
	type CreateClientOptions,
	type DefaultCustomMessageData,
	type DefaultPresence,
	type EmberClient
} from '@ember-link/core';

const CTX_KEY = Symbol('ember-link');

/**
 * Reactive wrapper around the Ember Link client. Stored in Svelte context
 * so consumers can read the current client via `getEmberLinkContext()`
 * (or `getClientContext()` for the shorthand). The wrapper recreates its
 * underlying client when options change, so changing e.g. `baseUrl` on the
 * provider swaps the connection transparently.
 */
export class EmberLinkContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> {
	client: EmberClient<P, C> = $state(null as unknown as EmberClient<P, C>);
	#currentOptions: CreateClientOptions | null = null;

	constructor(initialOptions: CreateClientOptions) {
		this.setOptions(initialOptions);
	}

	setOptions(options: CreateClientOptions) {
		if (this.#currentOptions && shallowEqualOptions(options, this.#currentOptions)) return;
		this.client?.destroy?.();
		this.client = createClient<P, C>(options);
		this.#currentOptions = options;
	}

	destroy() {
		this.client?.destroy?.();
	}
}

/**
 * Create and register an Ember Link context in the current component's
 * Svelte context tree. Call this from a `+layout.svelte` script (or any
 * ancestor of components that need the client) — or use `<EmberLinkProvider>`
 * for the declarative form.
 */
export function setEmberLinkContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(
	initialOptions: CreateClientOptions | (() => CreateClientOptions)
): EmberLinkContext<P, C> {
	const options =
		typeof initialOptions === 'function' ? initialOptions() : initialOptions;
	const ctx = new EmberLinkContext<P, C>(options);
	setContext(CTX_KEY, ctx);
	return ctx;
}

/**
 * Retrieve the Ember Link context from an ancestor's setEmberLinkContext
 * call. Throws if none has been set.
 */
export function getEmberLinkContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(): EmberLinkContext<P, C> {
	if (!hasContext(CTX_KEY)) {
		throw new Error(
			'No Ember Link context found. Call `setEmberLinkContext(...)` in an ancestor component, or wrap with `<EmberLinkProvider>`.'
		);
	}
	return getContext(CTX_KEY);
}

/**
 * Shorthand for `getEmberLinkContext().client`. Useful when you only need
 * the client and don't care about the surrounding wrapper.
 */
export function getClientContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(): EmberClient<P, C> {
	return getEmberLinkContext<P, C>().client;
}

function shallowEqualOptions(a: CreateClientOptions, b: CreateClientOptions): boolean {
	if (a === b) return true;
	const aKeys = Object.keys(a) as Array<keyof CreateClientOptions>;
	const bKeys = Object.keys(b) as Array<keyof CreateClientOptions>;
	if (aKeys.length !== bKeys.length) return false;
	for (const k of aKeys) {
		if (a[k] !== b[k]) return false;
	}
	return true;
}
