<script lang="ts" module>
	const name = 'EMBER_LINK_PROVIDER';

	export function getClientContext<
		P extends Record<string, unknown> = DefaultPresence,
		C extends Record<string, unknown> = DefaultCustomMessageData
	>(): EmberClient<P, C> {
		if (!hasContext(name)) {
			throw new Error('Get client context must be called inside of a client provider');
		}

		return getContext(name);
	}
</script>

<script
	lang="ts"
	generics="P extends Record<string, unknown> = DefaultPresence, C extends Record<string, unknown> = DefaultCustomMessageData"
>
	import { getContext, hasContext, setContext, type Snippet } from 'svelte';
	import {
		createClient,
		type CreateClientOptions,
		type DefaultCustomMessageData,
		type DefaultPresence,
		type EmberClient
	} from '@ember-link/core';

	type Props = { children: Snippet<[]> } & CreateClientOptions;

	const { children, ...clientOptions }: Props = $props();

	const client = createClient<P, C>(clientOptions);

	setContext(name, client);
</script>

{@render children?.()}
