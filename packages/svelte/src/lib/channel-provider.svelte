<script lang="ts" module>
	export { SvelteChannel, setChannelContext, getChannelContext } from './channel.svelte';
</script>

<script
	lang="ts"
	generics="S extends IStorageProvider, P extends Record<string, unknown> = DefaultPresence, C extends Record<string, unknown> = DefaultCustomMessageData"
>
	import { onDestroy, untrack, type Snippet } from 'svelte';
	import {
		type ChannelOptions,
		type DefaultCustomMessageData,
		type DefaultPresence,
		type IStorageProvider
	} from '@ember-link/core';
	import { setChannelContext } from './channel.svelte';

	type Props = {
		channelName: string;
		children?: Snippet<[]>;
	} & ChannelOptions<S, P>;

	const props: Props = $props();

	const ctx = setChannelContext<P, C>();

	// Seed the channel synchronously so SSR/prerender sees a populated
	// context. `untrack` keeps Svelte from flagging this top-level prop
	// read — the `$effect` below covers reactive updates on the client.
	untrack(() => {
		ctx.joinChannel<S>(props.channelName, extractOptions(props));
	});

	// React to channelName / options changes — joinChannel is idempotent
	// when nothing has changed, and tears down the old channel before
	// joining the new one when something has.
	$effect(() => {
		ctx.joinChannel<S>(props.channelName, extractOptions(props));
	});

	onDestroy(() => {
		ctx.destroy();
	});

	function extractOptions(p: Props): ChannelOptions<S, P> {
		const { channelName, children, ...rest } = p;
		void channelName;
		void children;
		return rest as ChannelOptions<S, P>;
	}
</script>

{@render props.children?.()}
