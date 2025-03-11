<script lang="ts" module>
  export const name = "EMBER_LINK_CLIENT";

  export function getClientContext<P extends Record<string, unknown> = DefaultPresence>(): EmberClient<P> {
    if (!hasContext(name)) {
      throw new Error('Get client context must be called inside of a client provider')
    }

    return getContext(name);
  }
</script>

<script lang="ts">
	import { getContext, hasContext, setContext, type Snippet } from "svelte";
  import { type EmberClient, type CreateClientOptions, type DefaultPresence, createClient } from "@ember-link/core";

  const { clientOptions, children }: { clientOptions: CreateClientOptions, children: Snippet<[]> } = $props();

  const client = createClient(clientOptions)

  setContext(name, client);
</script>


{@render children?.()}