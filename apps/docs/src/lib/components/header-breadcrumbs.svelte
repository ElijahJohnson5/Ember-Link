<script lang="ts">
	import * as Breadcrumb from '$lib/components/ui/breadcrumb';

	let { items }: { items: Array<{ title: string; url?: string }> } = $props();
</script>

<Breadcrumb.Root>
	<Breadcrumb.List>
		{#each items as item, index (item)}
			{#if index !== items.length - 1}
				<Breadcrumb.Item class="hidden md:block">
					<Breadcrumb.Link href={item.url}>{item.title}</Breadcrumb.Link>
				</Breadcrumb.Item>
				<Breadcrumb.Separator class="hidden md:block" />
			{:else}
				<Breadcrumb.Item>
					<Breadcrumb.Page>{item.title}</Breadcrumb.Page>
				</Breadcrumb.Item>
			{/if}
		{/each}
	</Breadcrumb.List>
</Breadcrumb.Root>
