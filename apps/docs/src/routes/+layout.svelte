<script lang="ts">
	import '../app.postcss';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import { Separator } from '$lib/components/ui/separator';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Button } from '$lib/components/ui/button';
	import { toggleMode, ModeWatcher } from 'mode-watcher';
	import Sun from '@lucide/svelte/icons/sun';
	import MoonStar from '@lucide/svelte/icons/moon-star';
	import type { Snippet } from 'svelte';
	import type { NavData } from '$lib/types';
	import { page } from '$app/state';
	import HeaderBreadcrumbs from '$lib/components/header-breadcrumbs.svelte';
	import LibrarySelector from '$lib/components/library-selector.svelte';
	import Discord from '$lib/components/icons/discord.svelte';
	import Github from '$lib/components/icons/github.svelte';

	let { children }: { children: Snippet<[]> } = $props();

	const navData = $state({
		navMain: [
			{
				title: 'Getting Started',
				url: '/getting-started',
				items: [
					{
						title: 'Support',
						url: '/support'
					}
				]
			},
			{
				title: 'Examples',
				items: [
					{
						title: 'Cursors',
						url: '/cursors'
					},
					{
						title: 'Todos',
						url: '/todos'
					},
					{
						title: 'Collaborative Editing',
						url: '/collaborative'
					}
				]
			},
			{
				title: 'Packages',
				url: '/packages',
				items: [
					{
						title: '@ember-link/core',
						url: '/packages/core'
					},
					{
						title: '@ember-link/react',
						url: '/packages/react'
					},
					{
						title: '@ember-link/svelte',
						url: '/packages/svelte'
					},
					{
						title: '@ember-link/storage',
						url: '/packages/storage'
					},
					{
						title: '@ember-link/yjs-storage',
						url: '/packages/yjs-storage'
					},
					{
						title: '@ember-link/yjs-provider',
						url: '/packages/yjs-provider'
					},
					{
						title: '@ember-link/protocol',
						url: '/packages/protocol'
					}
				]
			}
		]
	}) satisfies NavData;

	const breadcrumbItems = $derived.by(() => {
		let items: Array<{ title: string; url?: string }> = [];

		navData.navMain.forEach((mainItem) => {
			const currentItem = (mainItem.items ?? []).find((item) => {
				return item.url === page.url.pathname;
			});

			if (mainItem.url === page.url.pathname) {
				items.push({ url: mainItem.url, title: mainItem.title });
				return;
			}

			if (!currentItem) {
				return;
			}

			items.push({ url: mainItem.url, title: mainItem.title });
			items.push({ url: currentItem.url, title: currentItem.title });
		});

		return items;
	});
</script>

<svelte:head>
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
	<link rel="icon" href="/favicon.jpg" type="image/jpg" sizes="16x16" />
	<title>Ember Link</title>
	<meta
		name="description"
		content="Integrate real-time collaboration into any application effortlessly"
	/>
	<meta
		name="keywords"
		content="Ember Link, documentation, getting started, integration guide, real-time, collaboration"
	/>
	<meta property="og:title" content="Getting Started with EmberLink" />
	<meta property="og:description" content="The Open-Source SDK for Real-Time Collaboration" />
</svelte:head>

<ModeWatcher />

<Sidebar.Provider>
	<AppSidebar data={navData} />
	<Sidebar.Inset class="w-[calc(100%-16rem)]">
		<header
			class="sticky top-0 z-10 flex h-16 shrink-0 items-center gap-2 border-b bg-background px-4"
		>
			<Sidebar.Trigger class="-ml-1" />
			<Separator orientation="vertical" class="mr-2 h-4" />
			<HeaderBreadcrumbs items={breadcrumbItems} />

			<div class="ml-auto">
				<LibrarySelector />

				<Button size="icon" variant="ghost" onclick={toggleMode} aria-label="Toggle dark mode">
					<Sun
						class="h-[1.2rem] w-[1.2rem] -rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
					/>
					<MoonStar
						class="light:rotate-90 absolute h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:scale-0"
					/>
				</Button>

				<Button
					size="icon"
					variant="ghost"
					href="https://discord.gg/YU2wGQtgE7"
					target="_blank"
					aria-label="Join Discord"
				>
					<Discord />
				</Button>

				<Button
					size="icon"
					variant="ghost"
					href="https://github.com/ElijahJohnson5/Ember-Link"
					target="_blank"
					aria-label="Github Repo"
				>
					<Github />
				</Button>
			</div>
		</header>
		<div class="flex w-full flex-grow">
			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
