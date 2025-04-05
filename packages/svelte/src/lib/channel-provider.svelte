<script lang="ts" module>
	const name = 'EMBER_LINK_CHANNEL';

	export function getChannelContext<
		P extends Record<string, unknown> = DefaultPresence,
		C extends Record<string, unknown> = DefaultCustomMessageData
	>(): SvelteChannel<P, C> {
		if (!hasContext(name)) {
			throw new Error('Could not find context, only use this inside of a channel provider');
		}

		return getContext(name);
	}

	class SvelteChannel<P extends DefaultPresence, C extends DefaultCustomMessageData> {
		others: User<P>[] = $state([]);
		myPresence: P | null = $state(null);
		storage: SvelteStorage | null = null;
		channel: Channel<P, C>;

		constructor(channel: Channel<P, C>) {
			this.channel = channel;

			if (channel.hasStorage()) {
				this.storage = new SvelteStorage(channel.getStorage());
			}

			this.channel.events.subscribe('others', (others) => {
				this.others = others;
			});

			this.channel.events.subscribe('presence', (presence) => {
				this.myPresence = presence;
			});
		}

		updatePresence(state: P) {
			this.channel.updatePresence(state);
		}

		getRawChannel() {
			return this.channel;
		}

		getStorage() {
			if (this.storage) {
				return this.storage;
			}

			throw new Error('A storage provider must be configured to use storage');
		}
	}

	class SvelteStorage {
		storage: IStorage;

		constructor(storage: IStorage) {
			this.storage = storage;
		}

		getArray<T>(name: string): SvelteArrayStorage<T> {
			const array = this.storage.getArray<T>(name);

			return new SvelteArrayStorage<T>(array);
		}

		getMap<K extends string, V>(name: string): SvelteMapStorage<K, V> {
			const map = this.storage.getMap<K, V>(name);

			return new SvelteMapStorage(map);
		}
	}

	class SvelteArrayStorage<T> {
		public inner: ArrayStorage<T>;
		public current = $state<Array<T>>([]);

		constructor(arrayStorage: ArrayStorage<T>) {
			this.inner = arrayStorage;

			this.current = [...this.inner];

			arrayStorage.subscribe(() => {
				this.current = [...this.inner];
			});
		}
	}

	class SvelteMapStorage<K extends string, V> {
		public inner: MapStorage<K, V>;
		public current = new SvelteMap<K, V>();

		constructor(mapStorage: MapStorage<K, V>) {
			this.inner = mapStorage;

			for (const [key, value] of this.inner) {
				this.current.set(key, value);
			}

			mapStorage.subscribe(() => {
				this.current.clear();
				for (const [key, value] of this.inner) {
					this.current.set(key, value);
				}
			});
		}
	}
</script>

<script
	lang="ts"
	generics="S extends IStorageProvider, P extends Record<string, unknown> = DefaultPresence, C extends Record<string, unknown> = DefaultCustomMessageData"
>
	import { getContext, hasContext, onDestroy, setContext, type Snippet } from 'svelte';
	import {
		type ChannelOptions,
		type IStorageProvider,
		type DefaultPresence,
		type Channel,
		type DefaultCustomMessageData,
		type User,
		type ArrayStorage,
		type MapStorage,
		type IStorage
	} from '@ember-link/core';
	import { getClientContext } from './ember-link-provider.svelte';
	import { SvelteMap } from 'svelte/reactivity';

	type Props = {
		channelName: string;
		children: Snippet<[]>;
	} & ChannelOptions<S, P>;

	const { channelName, children, ...options }: Props = $props();

	const client = getClientContext<P, C>();

	const { channel, leave } = client.joinChannel(channelName, options);

	setContext(name, new SvelteChannel<P, C>(channel));

	onDestroy(() => {
		leave();
	});
</script>

{@render children?.()}
