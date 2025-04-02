<script lang="ts">
	import * as Breadcrumb from '$lib/components/ui/breadcrumb';

	let { items }: { items: Array<string> } = $props();
</script>

<Breadcrumb.Root>
	<Breadcrumb.List>
		{#each items as item, index (item)}
			{#if index !== items.length - 1}
				<Breadcrumb.Item class="hidden md:block">
					<Breadcrumb.Link>{item}</Breadcrumb.Link>
				</Breadcrumb.Item>
				<Breadcrumb.Separator class="hidden md:block" />
			{:else}
				<Breadcrumb.Item>
					<Breadcrumb.Page>{item}</Breadcrumb.Page>
				</Breadcrumb.Item>
			{/if}
		{/each}
	</Breadcrumb.List>
</Breadcrumb.Root>
