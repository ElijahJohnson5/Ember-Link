<script lang="ts">
	import { ChannelProvider } from '@ember-link/svelte';
	import Channel from './Channel.svelte';
</script>

<ChannelProvider channelName="test">
	<Channel />
</ChannelProvider>
