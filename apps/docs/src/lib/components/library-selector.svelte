<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { selectedLibrary, libraryOptions } from '$lib/library.svelte';
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button variant="ghost" {...props}>
				<svelte:component this={selectedLibrary.current.icon} />
				{selectedLibrary.current.display}
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content>
		<DropdownMenu.Group>
			{#each libraryOptions as libraryOption (libraryOption.value)}
				<DropdownMenu.Item
					onclick={() => {
						selectedLibrary.current = libraryOption;
					}}
				>
					{#snippet child({ props })}
						<Button variant="ghost" {...props} class="flex w-full justify-start">
							<svelte:component this={libraryOption.icon} />
							{libraryOption.display}
						</Button>
					{/snippet}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
