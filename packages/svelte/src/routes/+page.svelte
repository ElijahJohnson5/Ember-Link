<script lang="ts">
	import { getChannelContext } from '$lib/channel-provider.svelte';

	const channel = getChannelContext();
</script>

<div>
	{#each channel.others as other (other.clientId)}
		<div>{other.clientId}</div>
	{/each}
</div>
