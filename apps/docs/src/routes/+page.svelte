<script lang="ts">
	import Flame from '@lucide/svelte/icons/flame';
	import Example from './example.md';
</script>

<div class="prose mx-auto w-full max-w-4xl p-6 dark:prose-invert">
	<div class="flex flex-col items-center">
		<Flame size={60} class="mb-4" />
		<h1 class="mb-0">Ember Link</h1>
		<h2 class="m-0 p-2">The Open-Source SDK for Real-Time Collaboration</h2>
	</div>

	<p>
		Ember Link makes it effortless to add real-time collaboration to your apps. Whether you're
		building a document editor, whiteboard, or any interactive experience, Ember Link provides the
		tools you needm with the flexibility to self-host for full control over privacy and
		customization.
	</p>

	<h2>Built for Performance & Reliability</h2>
	<p>
		Made with Rust and TypeScript, Ember Link ensures strong type safety, high performance, and an
		easy-to-use API without compromising power.
	</p>

	<h2>Key Features:</h2>

	<ul class="list-none">
		<li>✅ User Presence Tracking - Know who's online and active in real time.</li>
		<li>✅ Custom Messaging - Send and receive real-time messages between clients.</li>
		<li>
			✅ Shared Storage with CRDTs - Conflict-free data synchronization for seamless collaboration.
		</li>
		<li>✅ Collaborative Editing - Synchronize edits across users with shared storage.</li>
	</ul>

	<h2>Quickstart</h2>

	<p>
		Ember Link is designed to be intuitive and developer-friendly. Here's how easy it is to
		integrate:
	</p>

	<Example />

	<p>
		Ember Link makes real-time collaboration powerful yet simple. <a href="/getting-started">
			Get started today
		</a>
		and bring seamless teamwork to your app! 🚀
	</p>
</div>
