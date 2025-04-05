<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import X from '@lucide/svelte/icons/x';
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();

	const storage = channel.getStorage();

	const todos = storage.getArray<{ text: string }>('todos');

	const someoneIsTyping = $derived(channel.others.some((other) => other.isTyping));

	let todoValue = $state('');
</script>

<div class="flex w-full flex-grow flex-col p-6">
	<div class="self-end">
		{#if channel.others.length === 1}
			There is {channel.others.length} other user online
		{:else}
			There are {channel.others.length} other users online
		{/if}
	</div>
	<Input
		type="text"
		placeholder="Write a todo"
		bind:value={todoValue}
		onkeydown={(e) => {
			if (e.key === 'Enter') {
				channel.updatePresence({ isTyping: false });
				todos.inner.push({ text: todoValue });
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

	<div class="opacity-0" class:opacity-100={someoneIsTyping}>Somone is typing...</div>

	{#each todos.current as todo, index (index)}
		<div class="flex items-center">
			{todo.text}

			<Button
				size="icon"
				variant="ghost"
				class="ml-auto"
				onclick={() => {
					todos.inner.delete(index, 1);
				}}
			>
				<X />
			</Button>
		</div>
	{/each}
</div>
