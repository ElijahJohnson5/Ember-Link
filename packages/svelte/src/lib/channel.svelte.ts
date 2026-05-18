import { getContext, hasContext, setContext } from 'svelte';
import { SvelteStorage } from '$lib/storage.svelte';
import {
	type Channel,
	type ChannelOptions,
	type DefaultCustomMessageData,
	type DefaultPresence,
	type Status,
	type User
} from '@ember-link/core';
import { getEmberLinkContext } from './ember-link-context.svelte';

const CTX_KEY = Symbol('ember-link-channel');

/**
 * Reactive wrapper around a single channel. `others`, `myPresence`,
 * `status`, and `storage` are `$state` fields, so accessing them from a
 * `.svelte` template (or inside `$derived` / `$effect`) triggers updates.
 *
 * One instance lives per `setChannelContext` call. The instance can swap
 * its underlying channel (via `joinChannel`) when the parent's
 * channelName or options change, so consumers don't have to remount.
 */
export class SvelteChannel<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> {
	others: User<P>[] = $state([]);
	myPresence: P | null = $state(null);
	status: Status = $state('initial');
	storage: SvelteStorage | null = $state(null);

	#channel: Channel<P, C> | null = null;
	#leave: (() => void) | null = null;
	#currentName: string | null = null;
	#currentOptions: ChannelOptions<P> | null = null;
	#unsubscribers: Array<() => void> = [];

	/**
	 * Join (or re-join) a channel. Idempotent: if called again with the
	 * same name and options, does nothing. If called with different
	 * arguments, leaves the previous channel before joining the new one.
	 *
	 * Designed to be called from within `$effect` so prop changes on the
	 * <ChannelProvider> propagate cleanly.
	 */
	joinChannel(channelName: string, options: ChannelOptions<P>): void {
		const normalized = options ?? {};
		if (
			channelName === this.#currentName &&
			this.#currentOptions &&
			shallowEqualOptions(normalized, this.#currentOptions)
		) {
			return;
		}

		this.#teardown();

		const client = getEmberLinkContext<P, C>().client;
		const { channel, leave } = client.joinChannel(channelName, options);

		this.#channel = channel;
		this.#leave = leave;
		this.#currentName = channelName;
		this.#currentOptions = normalized as ChannelOptions<P>;

		const rawStorage = channel.getStorage();
		this.storage = rawStorage ? new SvelteStorage(rawStorage) : null;
		this.others = channel.getOthers();
		this.myPresence = channel.getPresence();
		this.status = channel.getStatus();

		this.#unsubscribers.push(
			channel.events.subscribe('others', (others) => {
				this.others = others;
			}),
			channel.events.subscribe('presence', (presence) => {
				this.myPresence = presence;
			}),
			channel.events.subscribe('status', (status) => {
				this.status = status;
			})
		);
	}

	updatePresence(state: P) {
		this.#channel?.updatePresence(state);
	}

	getRawChannel(): Channel<P, C> {
		if (!this.#channel) {
			throw new Error('Channel not joined yet — call `joinChannel(...)` first.');
		}
		return this.#channel;
	}

	getStorage(): SvelteStorage {
		if (!this.storage) {
			throw new Error('A storage provider must be configured to use storage');
		}
		return this.storage;
	}

	destroy() {
		this.#teardown();
	}

	#teardown() {
		for (const unsub of this.#unsubscribers) unsub();
		this.#unsubscribers = [];
		this.#leave?.();
		this.#leave = null;
		this.#channel = null;
		this.#currentName = null;
		this.#currentOptions = null;
		this.storage = null;
		this.others = [];
		this.myPresence = null;
		this.status = 'initial';
	}
}

/**
 * Register an empty `SvelteChannel` in the component's Svelte context.
 * Call `ctx.joinChannel(name, options)` from a `$effect` to wire it up
 * to reactive props — or use `<ChannelProvider>` for the declarative form.
 */
export function setChannelContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(): SvelteChannel<P, C> {
	const ctx = new SvelteChannel<P, C>();
	setContext(CTX_KEY, ctx);
	return ctx;
}

export function getChannelContext<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(): SvelteChannel<P, C> {
	if (!hasContext(CTX_KEY)) {
		throw new Error(
			'No channel context found. Call `setChannelContext()` in an ancestor component, or wrap with `<ChannelProvider>`.'
		);
	}
	return getContext(CTX_KEY);
}

function shallowEqualOptions<T extends object>(a: T, b: T): boolean {
	if (a === b) return true;
	const aKeys = Object.keys(a) as Array<keyof T>;
	const bKeys = Object.keys(b) as Array<keyof T>;
	if (aKeys.length !== bKeys.length) return false;
	for (const k of aKeys) {
		if (a[k] !== b[k]) return false;
	}
	return true;
}
