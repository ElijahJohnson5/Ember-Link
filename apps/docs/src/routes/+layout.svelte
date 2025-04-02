<script lang="ts">
	import '../app.css';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import { Separator } from '$lib/components/ui/separator';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Button } from '$lib/components/ui/button';
	import { toggleMode, ModeWatcher } from 'mode-watcher';
	import { Sun, MoonStar } from '@lucide/svelte';
	import type { Snippet } from 'svelte';
	import type { NavData } from '$lib/types';
	import { page } from '$app/state';
	import HeaderBreadcrumbs from '$lib/components/header-breadcrumbs.svelte';
	import LibrarySelector from '$lib/components/library-selector.svelte';

	let { children }: { children: Snippet<[]> } = $props();

	const navData = $state({
		navMain: [
			{
				title: 'Getting Started',
				items: [
					{
						title: 'Installation & Usage',
						url: '/installation'
					},
					{
						title: 'Project Structure',
						url: '#'
					}
				]
			},
			{
				title: 'Building Your Application',
				items: [
					{
						title: 'Routing',
						url: '#'
					},
					{
						title: 'Data Fetching',
						url: '#'
					},
					{
						title: 'Rendering',
						url: '#'
					},
					{
						title: 'Caching',
						url: '#'
					},
					{
						title: 'Styling',
						url: '#'
					},
					{
						title: 'Optimizing',
						url: '#'
					},
					{
						title: 'Configuring',
						url: '#'
					},
					{
						title: 'Testing',
						url: '#'
					},
					{
						title: 'Authentication',
						url: '#'
					},
					{
						title: 'Deploying',
						url: '#'
					},
					{
						title: 'Upgrading',
						url: '#'
					},
					{
						title: 'Examples',
						url: '#'
					}
				]
			},
			{
				title: 'API Reference',
				items: [
					{
						title: 'Components',
						url: '#'
					},
					{
						title: 'File Conventions',
						url: '#'
					},
					{
						title: 'Functions',
						url: '#'
					},
					{
						title: 'next.config.js Options',
						url: '#'
					},
					{
						title: 'CLI',
						url: '#'
					},
					{
						title: 'Edge Runtime',
						url: '#'
					}
				]
			},
			{
				title: 'Architecture',
				items: [
					{
						title: 'Accessibility',
						url: '#'
					},
					{
						title: 'Fast Refresh',
						url: '#'
					},
					{
						title: 'Svelte Compiler',
						url: '#'
					},
					{
						title: 'Supported Browsers',
						url: '#'
					},
					{
						title: 'Rollup',
						url: '#'
					}
				]
			}
		]
	}) satisfies NavData;

	const breadcrumbItems = $derived.by(() => {
		let items: Array<string> = [];

		navData.navMain.forEach((mainItem) => {
			const currentItem = mainItem.items.find((item) => {
				return item.url === page.url.pathname;
			});

			if (!currentItem) {
				return;
			}

			items.push(mainItem.title);
			items.push(currentItem.title);
		});

		return items;
	});
</script>

<ModeWatcher />

<Sidebar.Provider>
	<AppSidebar data={navData} />
	<Sidebar.Inset>
		<header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
			<Sidebar.Trigger class="-ml-1" />
			<Separator orientation="vertical" class="mr-2 h-4" />
			<HeaderBreadcrumbs items={breadcrumbItems} />

			<div class="ml-auto">
				<LibrarySelector />

				<Button variant="ghost" onclick={toggleMode}>
					<Sun
						class="h-[1.2rem] w-[1.2rem] -rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
					/>
					<MoonStar
						class="light:rotate-90 absolute h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:scale-0"
					/>
				</Button>
			</div>
		</header>
		<div class="flex">
			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
