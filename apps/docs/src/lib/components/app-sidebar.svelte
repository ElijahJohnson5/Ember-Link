<script lang="ts">
	import { page } from '$app/state';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import GalleryVerticalEnd from '@lucide/svelte/icons/gallery-vertical-end';
	import type { ComponentProps } from 'svelte';
	import { type NavData } from '$lib/types';
	import Button from '$lib/components/ui/button/button.svelte';

	type Props = ComponentProps<typeof Sidebar.Root> & {
		data: NavData;
	};

	let { ref = $bindable(null), data, ...restProps }: Props = $props();
</script>

<Sidebar.Root bind:ref {...restProps}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg">
					{#snippet child({ props })}
						<a href="/" {...props}>
							<div
								class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
							>
								<GalleryVerticalEnd class="size-4" />
							</div>
							<div class="flex flex-col gap-0.5 leading-none">
								<span class="font-semibold">Documentation</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		{#each data.navMain as mainItem (mainItem.title)}
			<Sidebar.Group>
				<Sidebar.GroupLabel class="justify-start text-sm text-primary">
					{#snippet child({ props })}
						{#if mainItem.url}
							<Button variant="link" {...props} href={mainItem.url}>
								{mainItem.title}
							</Button>
						{:else}
							<div {...props}>
								{mainItem.title}
							</div>
						{/if}
					{/snippet}
				</Sidebar.GroupLabel>
				<Sidebar.Menu>
					<Sidebar.MenuItem>
						{#if mainItem.items?.length}
							<Sidebar.MenuSub class="mx-1">
								{#each mainItem.items as item (item.title)}
									<Sidebar.MenuSubItem>
										<Sidebar.MenuSubButton isActive={page.url.pathname === item.url}>
											{#snippet child({ props })}
												<a href={item.url} {...props}>{item.title}</a>
											{/snippet}
										</Sidebar.MenuSubButton>
									</Sidebar.MenuSubItem>
								{/each}
							</Sidebar.MenuSub>
						{/if}
					</Sidebar.MenuItem>
				</Sidebar.Menu>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>
	<Sidebar.Rail />
</Sidebar.Root>
