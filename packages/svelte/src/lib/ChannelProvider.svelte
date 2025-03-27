<script lang="ts" module>
	export function getChannelContext<
		P extends Record<string, unknown> = DefaultPresence,
		C extends Record<string, unknown> = DefaultCustomMessageData
	>(channelName: string): Channel<P, C> {
		if (!hasContext(`EMBER_LINK_CHANNEL_${channelName}`)) {
			throw new Error(
				'Could not find context, only use this inside of a channel provider with the same channel name given'
			);
		}

		return getContext(`EMBER_LINK_CHANNEL_${channelName}`);
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
		type DefaultCustomMessageData
	} from '@ember-link/core';
	import { getClientContext } from './ClientProvider.svelte';

	const {
		channelName,
		options,
		children
	}: { children: Snippet<[]>; channelName: string; options?: ChannelOptions<S, P> } = $props();

	const client = getClientContext<P, C>();

	const { channel, leave } = client.joinChannel(channelName, options);

	setContext(`EMBER_LINK_CHANNEL_${channelName}`, channel);

	onDestroy(() => {
		leave();
	});
</script>

{@render children?.()}
