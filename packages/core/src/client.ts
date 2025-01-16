// What the api should look like

import { IStorageProvider } from '@ember-link/storage';
import { Channel, ChannelConfig, createChannel } from './channel.js';

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

export function createClient({ baseUrl }: CreateClientOptions) {
  const channels = new Map<string, Channel>();

  function joinChannel<S extends IStorageProvider>(
    channelName: string,
    options?: ChannelConfig<S>['options']
  ) {
    if (channels.has(channelName)) {
      return channels.get(channelName)!;
    }

    const { channel, leave } = createChannel<S>({
      channelName,
      baseUrl,
      options
    });

    channels.set(channelName, channel);

    return channel;
  }

  return {
    joinChannel
  };
}
