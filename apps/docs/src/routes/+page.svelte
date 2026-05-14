<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import CursorExample from '$lib/components/cursor/index.svelte';
	import HomeQuickstart from '$lib/components/home-quickstart.svelte';
	import HomeInstall from '$lib/components/home-install.svelte';
	import Flame from '@lucide/svelte/icons/flame';
	import Users from '@lucide/svelte/icons/users';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import Database from '@lucide/svelte/icons/database';
	import PencilLine from '@lucide/svelte/icons/pencil-line';
	import Zap from '@lucide/svelte/icons/zap';
	import Server from '@lucide/svelte/icons/server';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Github from '$lib/components/icons/github.svelte';

	const features = [
		{
			icon: Users,
			title: 'Presence',
			desc: 'Know who is online and where their cursor is. Sub-second fanout, optional throttling, smooth client-side interpolation.'
		},
		{
			icon: Database,
			title: 'Shared storage',
			desc: 'Yjs-backed CRDTs out of the box. Conflict-free merges across any number of clients with arbitrary network conditions.'
		},
		{
			icon: PencilLine,
			title: 'Collaborative editing',
			desc: 'Drop-in support for Tiptap, BlockNote, ProseMirror, and any other editor that speaks Yjs. Live cursors and selections included.'
		},
		{
			icon: MessageSquare,
			title: 'Custom messages',
			desc: 'Typed pub/sub between clients in a channel. Bring your own message shape; the SDK handles delivery and ordering.'
		},
		{
			icon: Zap,
			title: 'Rust-powered',
			desc: 'Built on Tokio + Axum on the server, with a Cloudflare Workers + Durable Objects target for global edge deployment.'
		},
		{
			icon: Server,
			title: 'Self-hostable',
			desc: 'One small Docker image, or `wrangler deploy` to your own Cloudflare account. MIT-licensed, no vendor lock-in.'
		}
	] as const;
</script>

<svelte:head>
	<title>Ember Link: open-source real-time collaboration SDK</title>
	<meta
		name="description"
		content="Add presence, cursors, shared storage, and custom messages to your app. Self-hostable. Built in Rust."
	/>
</svelte:head>

<!--
	Single flex-col wrapper so the multiple <section> blocks below
	don't lay out as horizontal columns. The docs +layout.svelte slot
	is wrapped in `<div class="flex w-full flex-grow">` (default
	flex-direction: row), so without this wrapper each <section>
	would render side-by-side instead of stacked.
-->
<div class="flex w-full flex-col">
	<!-- ─── HERO ─────────────────────────────────────────────────────── -->
	<section class="border-b">
		<div class="mx-auto grid w-full max-w-6xl gap-10 px-6 py-12 lg:grid-cols-[1.1fr_1fr] lg:py-20">
			<div class="flex flex-col items-start justify-center">
				<Badge variant="outline" class="mb-4">
					<Flame size={12} class="mr-1.5" />
					v0.1 · Open Source
				</Badge>
				<h1 class="mb-3 text-balance text-4xl font-bold tracking-tight lg:text-6xl">
					Real-time collaboration, your stack.
				</h1>
				<p class="mb-6 max-w-xl text-balance text-lg text-muted-foreground">
					Ember Link is an open-source SDK for presence, cursors, shared storage, and custom
					messages. Drop it into a React or Svelte app, or self-host the entire server in Rust on
					Tokio or Cloudflare Workers.
				</p>

				<div class="mb-8 flex flex-wrap gap-3">
					<Button href="/getting-started" size="lg">
						Get started
						<ArrowRight size={16} class="ml-1" />
					</Button>
					<Button
						href="https://github.com/ElijahJohnson5/Ember-Link"
						variant="outline"
						size="lg"
						target="_blank"
					>
						<Github />
						GitHub
					</Button>
				</div>

				<div class="w-full max-w-xl">
					<HomeInstall />
				</div>
			</div>

			<!--
				Right side: two side-by-side live cursor demos so a single
				visitor can move their cursor in one panel and see their own
				cursor appear in the other (in addition to any other live
				viewers). Both panels join the same sandbox channel.
			-->
			<div class="flex flex-col">
				<div class="mb-2 flex items-center justify-between">
					<span class="text-xs text-muted-foreground">Live demo</span>
					<span class="text-xs text-muted-foreground">Move your cursor here</span>
				</div>
				<div class="flex flex-col divide-y-2 overflow-hidden rounded-lg border-2">
					<CursorExample />
					<CursorExample />
				</div>
			</div>
		</div>
	</section>

	<!-- ─── QUICKSTART ─────────────────────────────────────────────────── -->
	<section class="border-b">
		<div class="mx-auto w-full max-w-6xl px-6 py-12 lg:py-16">
			<div class="mb-8 max-w-2xl">
				<h2 class="mb-2 text-3xl font-bold tracking-tight">Three lines to your first channel.</h2>
				<p class="text-muted-foreground">
					Pick your stack. The SDK gives you a typed client, reactive presence, and ref-counted
					channels with zero hand-rolled WebSocket plumbing.
				</p>
			</div>
			<div class="rounded-lg border bg-card">
				<HomeQuickstart />
			</div>
		</div>
	</section>

	<!-- ─── FEATURES ──────────────────────────────────────────────────── -->
	<section class="border-b">
		<div class="mx-auto w-full max-w-6xl px-6 py-12 lg:py-16">
			<div class="mb-8 max-w-2xl">
				<h2 class="mb-2 text-3xl font-bold tracking-tight">What's in the box.</h2>
				<p class="text-muted-foreground">
					The core primitives every collaborative app needs, with sensible defaults and escape
					hatches when you need them.
				</p>
			</div>
			<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each features as feature (feature.title)}
					<div class="rounded-lg border bg-card p-5 transition-colors hover:bg-accent/30">
						<feature.icon size={22} class="mb-3 text-foreground" />
						<h3 class="mb-1.5 font-semibold">{feature.title}</h3>
						<p class="text-sm text-muted-foreground">{feature.desc}</p>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<!-- ─── EXAMPLES ──────────────────────────────────────────────────── -->
	<section class="border-b">
		<div class="mx-auto w-full max-w-6xl px-6 py-12 lg:py-16">
			<div class="mb-8 max-w-2xl">
				<h2 class="mb-2 text-3xl font-bold tracking-tight">See it in action.</h2>
				<p class="text-muted-foreground">
					Live demos running against a real Ember Link server. Open any of them in two tabs to
					collaborate with yourself.
				</p>
			</div>
			<div class="grid gap-4 md:grid-cols-3">
				<a
					href="/cursors"
					class="group rounded-lg border bg-card p-5 transition-colors hover:bg-accent/30"
				>
					<h3 class="mb-1.5 font-semibold group-hover:underline">Live cursors</h3>
					<p class="text-sm text-muted-foreground">
						Throttled presence sends with rAF-lerped rendering. Figma-style smooth motion at ~30Hz
						of network traffic.
					</p>
				</a>
				<a
					href="/todos"
					class="group rounded-lg border bg-card p-5 transition-colors hover:bg-accent/30"
				>
					<h3 class="mb-1.5 font-semibold group-hover:underline">Shared todos</h3>
					<p class="text-sm text-muted-foreground">
						A todo list backed by a Yjs document. Conflict-free across tabs, persisted on the
						server.
					</p>
				</a>
				<a
					href="/collaborative"
					class="group rounded-lg border bg-card p-5 transition-colors hover:bg-accent/30"
				>
					<h3 class="mb-1.5 font-semibold group-hover:underline">Collaborative editor</h3>
					<p class="text-sm text-muted-foreground">
						Tiptap with live cursors and shared selections. Demonstrates the Yjs-provider
						integration.
					</p>
				</a>
			</div>
		</div>
	</section>

	<!-- ─── BOTTOM CTA ────────────────────────────────────────────────── -->
	<section>
		<div class="mx-auto w-full max-w-4xl px-6 py-16 text-center lg:py-24">
			<h2 class="mb-3 text-3xl font-bold tracking-tight lg:text-4xl">Ready to ship real-time?</h2>
			<p class="mx-auto mb-6 max-w-2xl text-muted-foreground">
				A single Docker image gets you a production server in under a minute. The SDKs handle
				everything else.
			</p>
			<div class="flex flex-wrap justify-center gap-3">
				<Button href="/getting-started" size="lg">
					Get started
					<ArrowRight size={16} class="ml-1" />
				</Button>
				<Button href="/packages" variant="outline" size="lg">Browse the API</Button>
			</div>
		</div>
	</section>
</div>
