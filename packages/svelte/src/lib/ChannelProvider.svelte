<script lang="ts" module>
  export function getChannelContext<P extends Record<string, unknown> = DefaultPresence>(channelName: string): Channel<P> {
    if (!hasContext(`EMBER_LINK_CHANNEL_${channelName}`)) {
      throw new Error("Could not find context, only use this inside of a channel provider");
    }

    return getContext(`EMBER_LINK_CHANNEL_${channelName}`);
  }
</script>

<script lang="ts" generics="S extends IStorageProvider, P extends Record<string, unknown> = DefaultPresence">
	import { getContext, hasContext, onDestroy, setContext, type Snippet } from "svelte";
  import { type ChannelOptions, type IStorageProvider, type DefaultPresence, type Channel } from '@ember-link/core';
	import { getClientContext } from "./ClientProvider.svelte";

  const { channelName, options, children }: { children: Snippet<[]>, channelName: string, options?: ChannelOptions<S, P> } = $props();

  const client = getClientContext();

  const { channel, leave } = client.joinChannel(channelName, options);

  setContext(`EMBER_LINK_CHANNEL_${channelName}`, channel);

  onDestroy(() => {
    leave();
  });
</script>


{@render children?.()}