<script lang="ts" module>
	export {
		EmberLinkContext,
		setEmberLinkContext,
		getEmberLinkContext,
		getClientContext
	} from './ember-link-context.svelte';
</script>

<script
	lang="ts"
	generics="P extends Record<string, unknown> = DefaultPresence, C extends Record<string, unknown> = DefaultCustomMessageData"
>
	import { onDestroy, untrack, type Snippet } from 'svelte';
	import {
		type CreateClientOptions,
		type DefaultCustomMessageData,
		type DefaultPresence
	} from '@ember-link/core';
	import { setEmberLinkContext } from './ember-link-context.svelte';

	type Props = { children?: Snippet<[]> } & CreateClientOptions;

	const props: Props = $props();

	// Seed the context with the initial options (also covers SSR, where
	// `$effect` does not run). `untrack` keeps Svelte from flagging this
	// as a non-reactive capture — the `$effect` below picks up changes.
	const ctx = setEmberLinkContext<P, C>(() => untrack(() => extractOptions(props)));

	// React to prop changes by re-running setOptions. Reading `props.x`
	// inside the effect is what makes those fields reactive.
	$effect(() => {
		ctx.setOptions(extractOptions(props));
	});

	onDestroy(() => {
		ctx.destroy();
	});

	function extractOptions(p: Props): CreateClientOptions {
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const { children, ...rest } = p;
		return rest;
	}
</script>

{@render props.children?.()}
