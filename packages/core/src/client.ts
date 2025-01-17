// What the api should look like

import { IStorageProvider } from '@ember-link/storage';
import { Channel, ChannelConfig, createChannel } from './channel.js';
import { DefaultPresence } from './index.js';

/*

const client = createClient({
  baseUrl: url,
  apiKey: apiKey,
});

const channel = client.joinChannel(channelName);


let unsub = channel.events.subscribe("presence", (presenceData) => {
  // Do stuff with the presence
});

*/

interface CreateClientOptions {
  baseUrl: string;
  apiKey?: string;
}

export function createClient<P extends Record<string, unknown> = DefaultPresence>({
  baseUrl
}: CreateClientOptions) {
  const channels = new Map<string, { channel: Channel; leave: () => void }>();

  function joinChannel<S extends IStorageProvider>(
    channelName: string,
    options?: ChannelConfig<S, P>['options']
  ) {
    if (channels.has(channelName)) {
      return channels.get(channelName)!;
    }

    const { channel, leave } = createChannel<S, P>({
      channelName,
      baseUrl,
      options
    });

    channels.set(channelName, { channel, leave });

    return { channel, leave };
  }

  return {
    joinChannel
  };
}
