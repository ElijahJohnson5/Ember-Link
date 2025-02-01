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

export class WebSocketNotFoundError extends Error {
  constructor(reason: string) {
    super(reason);
  }
}

interface CreateClientOptions {
  baseUrl: string;
  authEndpoint?: AuthEndpoint;
}

export function createClient<P extends Record<string, unknown> = DefaultPresence>({
  baseUrl,
  authEndpoint
}: CreateClientOptions) {
  const channels = new Map<string, { channel: Channel<P>; leave: () => void }>();
  const auth = createAuth({
    authEndpoint,
    onAuthenticated: (value) => {
      // TODO: Set user
      console.log(value);
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
        return auth.getAuthValue(channelName);
      },
      createWebSocket: (authValue) => {
        const ws = typeof WebSocket === 'undefined' ? undefined : WebSocket;

        if (!ws) {
          throw new WebSocketNotFoundError(
            'Could not find websocket to be able to create one, polyfills will be allowed soon'
          );
        }

        const url = new URL(baseUrl);

        url.searchParams.set('channel_name', channelName);

        if (authValue.type === 'private') {
          url.searchParams.set('token', authValue.token.raw);
        }

        return new ws(url.toString());
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
