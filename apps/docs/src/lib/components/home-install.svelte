<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';

	// Plain Svelte component (not mdsvex .md) so we know runes + bind:value
	// definitely apply. The .md path was confusing — both tab sets are now
	// fully in svelte-land.
	let active = $state('yarn');

	const commands = [
		{ value: 'yarn', label: 'yarn', cmd: 'yarn add @ember-link/react' },
		{ value: 'npm', label: 'npm', cmd: 'npm install @ember-link/react' },
		{ value: 'pnpm', label: 'pnpm', cmd: 'pnpm add @ember-link/react' }
	] as const;

	function copy(text: string, target: HTMLButtonElement) {
		navigator.clipboard.writeText(text);
		target.classList.add('copied');
		window.setTimeout(() => target.classList.remove('copied'), 2000);
	}
</script>

<Tabs.Root bind:value={active} class="install-tabs">
	<Tabs.List class="bg-background mb-0 h-auto justify-start gap-1 rounded-none border-b p-0">
		{#each commands as c (c.value)}
			<Tabs.Trigger
				value={c.value}
				class="data-[state=active]:border-foreground data-[state=active]:bg-shiki-background rounded-b-none border-b-2 border-transparent px-3 py-1.5 text-sm"
			>
				{c.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	{#each commands as c (c.value)}
		<Tabs.Content value={c.value} class="bg-shiki-background mt-0 rounded-b border p-3">
			<div class="flex items-center justify-between gap-3">
				<code class="text-sm">$ {c.cmd}</code>
				<button
					type="button"
					aria-label="Copy"
					class="text-xs text-muted-foreground transition-colors hover:text-foreground"
					onclick={(e) => copy(c.cmd, e.currentTarget)}
				>
					Copy
				</button>
			</div>
		</Tabs.Content>
	{/each}
</Tabs.Root>
