<script lang="ts">
	import { getChannelContext } from '$lib/index.js';

	const channel = getChannelContext();
</script>

<div>
	{#each channel.others as other (other.clientId)}
		<div>{other.clientId}</div>
	{/each}
</div>
