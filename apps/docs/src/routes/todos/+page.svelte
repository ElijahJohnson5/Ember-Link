<script lang="ts">
	import TodosExample from '$lib/components/todos/index.svelte';
	import { Button } from '$lib/components/ui/button';
</script>

<div class="flex h-full w-full flex-col">
	<div class="prose mx-auto w-full max-w-4xl px-6 pt-6 dark:prose-invert">
		<h1><strong>Shared Todos</strong></h1>
		<p>
			The two todo lists below are backed by the same shared <code>ArrayStorage</code> on a single
			channel. Adding, editing, or removing an item in one list is merged into the other list
			conflict-free, courtesy of the Yjs CRDT backend supplied by <a
				href="/packages/yjs-storage">@ember-link/yjs-storage</a
			>. The server persists every update, so refreshing the page reloads the full list, and a third
			client joining later receives the current state as a snapshot on connect.
		</p>
		<p>
			This demo exercises <a href="/concepts/storage">Concepts → Storage</a> end to end, including
			the <code>useArrayStorage</code> hook from <a href="/packages/react">@ember-link/react</a>.
		</p>
	</div>

	<div>
		<Button
			variant="link"
			class="px-6 pt-2"
			target="_blank"
			href="https://github.com/ElijahJohnson5/Ember-Link/tree/main/apps/docs/src/lib/components/todos"
		>
			View the code here
		</Button>
	</div>

	<div class="flex w-full justify-center p-6 pt-2">
		<div
			class="flex w-full flex-col divide-y-2 border-2 md:flex-row md:justify-center md:divide-x-2 md:divide-y-0"
		>
			<TodosExample />
			<TodosExample />
		</div>
	</div>
</div>
