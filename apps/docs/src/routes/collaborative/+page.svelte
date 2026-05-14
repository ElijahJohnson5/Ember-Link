<script lang="ts">
	import CollaborativeExample from '$lib/components/collaborative/index.svelte';
	import { Button } from '$lib/components/ui/button';
</script>

<div class="flex h-full w-full flex-col">
	<div class="prose mx-auto w-full max-w-4xl px-6 pt-6 dark:prose-invert">
		<h1><strong>Collaborative Editor</strong></h1>
		<p>
			The two Tiptap editors below sync against the same underlying <code>Y.Doc</code> through <a
				href="/packages/yjs-provider">@ember-link/yjs-provider</a
			>. Type into either editor and the other editor receives the changes through Tiptap's
			<code>Collaboration</code> extension. The <code>CollaborationCaret</code> extension renders
			the remote cursor and selection of the other editor on top of the local text.
		</p>
		<p>
			The Yjs provider is the right tool when the application's text data is already managed by an
			editor that speaks Yjs, because the editor synchronizes against the document directly without
			going through the higher-level <code>ArrayStorage</code> and <code>MapStorage</code> APIs.
			For non-editor application state, use <a href="/concepts/storage">Concepts → Storage</a> and
			the storage hooks instead.
		</p>
	</div>

	<div>
		<Button
			variant="link"
			class="px-6 pt-2"
			target="_blank"
			href="https://github.com/ElijahJohnson5/Ember-Link/tree/main/apps/docs/src/lib/components/collaborative"
		>
			View the code here
		</Button>
	</div>

	<div class="flex w-full flex-grow justify-center p-6 pt-2">
		<div
			class="flex w-full flex-col divide-y-2 border-2 md:flex-row md:justify-center md:divide-x-2 md:divide-y-0"
		>
			<CollaborativeExample />
			<CollaborativeExample />
		</div>
	</div>
</div>
