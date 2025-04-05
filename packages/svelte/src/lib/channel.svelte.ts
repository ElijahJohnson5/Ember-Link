import { SvelteStorage } from '$lib/storage.svelte';
import type {
	Channel,
	DefaultCustomMessageData,
	DefaultPresence,
	Status,
	User
} from '@ember-link/core';

export class SvelteChannel<P extends DefaultPresence, C extends DefaultCustomMessageData> {
	others: User<P>[] = $state([]);
	myPresence: P | null = $state(null);
	storage: SvelteStorage | null = null;
	status: Status = $state('initial');
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

		this.channel.events.subscribe('status', (status) => {
			this.status = status;
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
