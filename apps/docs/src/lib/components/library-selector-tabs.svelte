<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { libraryOptions, selectedLibrary } from '$lib/library.svelte';
	import type { Snippet } from 'svelte';

	let { js, ts, react, svelte }: { js: Snippet; ts: Snippet; react: Snippet; svelte: Snippet } =
		$props();
</script>

<Tabs.Root value={selectedLibrary.current.value} class="library-selector-tabs relative">
	<Tabs.List class="bg-background px-0">
		{#each libraryOptions as library (library.value)}
			<Tabs.Trigger
				class="data-[state=active]:bg-shiki-background rounded-b-none"
				value={library.value}
				onclick={() => {
					selectedLibrary.current = library;
				}}
			>
				{#snippet child({ props })}
					<Button variant="ghost" {...props}>
						<library.icon />
						<span class="hidden md:block">
							{library.display}
						</span>
					</Button>
				{/snippet}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>
	<Tabs.Content class="mt-0" value="js">{@render js()}</Tabs.Content>
	<Tabs.Content class="mt-0" value="ts">{@render ts()}</Tabs.Content>
	<Tabs.Content class="mt-0" value="react">{@render react()}</Tabs.Content>
	<Tabs.Content class="mt-0" value="svelte">{@render svelte()}</Tabs.Content>
</Tabs.Root>
