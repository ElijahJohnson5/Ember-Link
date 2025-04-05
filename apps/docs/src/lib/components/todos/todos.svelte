<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import X from '@lucide/svelte/icons/x';
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();

	const storage = channel.getStorage();

	const todos = storage.getArray<{ text: string }>('todos');

	const someoneIsTyping = $derived(channel.others.some((other) => other.isTyping));

	let todoValue = $state('');
</script>

<div class="relative flex w-full flex-grow flex-col p-6">
	<div class="absolute right-2 top-2">
		<Badge>
			{channel.status.toUpperCase()}
		</Badge>
	</div>
	<div class="self-end py-2">
		{#if channel.others.length === 1}
			There is {channel.others.length} other user online
		{:else}
			There are {channel.others.length} other users online
		{/if}
	</div>
	<div class="flex space-x-2">
		<Input
			type="text"
			placeholder="Write a todo"
			bind:value={todoValue}
			onkeydown={(e) => {
				if (e.key === 'Enter') {
					channel.updatePresence({ isTyping: false });
					todos.push({ text: todoValue });
					todoValue = '';
				} else {
					channel.updatePresence({ isTyping: true });
				}
			}}
			onfocus={() =>
				channel.updatePresence({
					isTyping: true
				})}
			onblur={() =>
				channel.updatePresence({
					isTyping: false
				})}
		/>
		<Button
			variant="secondary"
			onclick={() => {
				channel.updatePresence({ isTyping: false });
				todos.push({ text: todoValue });
				todoValue = '';
			}}
		>
			Create
		</Button>
	</div>

	<div class="opacity-0" class:opacity-100={someoneIsTyping}>Somone is typing...</div>

	{#each todos.current as todo, index (index)}
		<div class="flex items-center">
			{todo.text}

			<Button
				size="icon"
				variant="ghost"
				class="ml-auto"
				onclick={() => {
					todos.delete(index, 1);
				}}
			>
				<X />
			</Button>
		</div>
	{/each}
</div>
