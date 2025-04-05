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
		type DefaultCustomMessageData
	} from '@ember-link/core';
	import { getClientContext } from './ember-link-provider.svelte';
	import { SvelteChannel } from '$lib/channel.svelte';

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
