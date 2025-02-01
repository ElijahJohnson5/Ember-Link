// What the api should look like

import { IStorageProvider } from '@ember-link/storage';
import { Channel, ChannelConfig, createChannel } from './channel';
import { DefaultPresence } from './index';
import { AuthEndpoint, createAuth } from './auth';

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
  authEndpoint: AuthEndpoint;
}

export function createClient<P extends Record<string, unknown> = DefaultPresence>({
  baseUrl,
  authEndpoint
}: CreateClientOptions) {
  const channels = new Map<string, { channel: Channel; leave: () => void }>();
  const auth = createAuth({
    authEndpoint,
    onAuthenticated: () => {
      // TODO: Set user
    }
  });

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
      authenticate: async () => {
        return auth.requestAuth(channelName);
      },
      options
    });

    channels.set(channelName, { channel, leave });

    return { channel, leave };
  }

  return {
    joinChannel
  };
}
