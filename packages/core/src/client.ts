// What the api should look like

import { IStorageProvider } from '@ember-link/storage';
import { Channel, ChannelConfig, createChannel } from './channel';
import { DefaultCustomMessageData, DefaultPresence } from './index';
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

export interface CreateClientOptions {
  baseUrl: string;
  authEndpoint?: AuthEndpoint;
  multiTenant?: {
    tenantId: string;
  };
  jwtSignerPublicKey?: string;
}

type JoinChannel<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> = <S extends IStorageProvider>(
  channelName: string,
  options?: ChannelConfig<S, P>['options']
) => { channel: Channel<P, C>; leave: () => void };

export interface EmberClient<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> {
  joinChannel: JoinChannel<P, C>;
}

export function createClient<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({ baseUrl, authEndpoint, jwtSignerPublicKey, multiTenant }: CreateClientOptions): EmberClient {
  const channels = new Map<string, { channel: Channel<P, C>; unsubs: Set<() => void> }>();
  const auth = createAuth({
    authEndpoint,
    jwtSignerPublicKey,
    multiTenant,
    onAuthenticated: (_value) => {
      // TODO: Set user
    }
  });

  function borrowChannel({ channel, unsubs }: { channel: Channel<P, C>; unsubs: Set<() => void> }) {
    const leave = () => {
      const selfLeave = leave;

      if (!unsubs.delete(selfLeave)) {
        console.warn(
          'This leave function was already called. Calling it more than once has no effect.'
        );
      } else {
        if (unsubs.size === 0) {
          channels.delete(channel.getName());
          channel.destroy();
        }
      }
    };

    unsubs.add(leave);
    return { channel, leave };
  }

  function joinChannel<S extends IStorageProvider>(
    channelName: string,
    options?: ChannelConfig<S, P>['options']
  ) {
    if (channels.has(channelName)) {
      return borrowChannel(channels.get(channelName)!);
    }

    const channel = createChannel<S, P, C>({
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

          if (authValue.token.tenantId) {
            url.searchParams.set('tenant_id', authValue.token.tenantId);
          }
        } else {
          if (authValue.tenantId) {
            url.searchParams.set('tenant_id', authValue.tenantId);
          }
        }

        if (options?.storageProvider?.type) {
          url.searchParams.set('storage_type', options.storageProvider.type);
        }

        url.pathname = '/ws';

        return new ws(url.toString());
      },
      options
    });

    const channelWithUnsubs = {
      channel,
      unsubs: new Set<() => void>()
    };

    channels.set(channelName, channelWithUnsubs);

    return borrowChannel(channelWithUnsubs);
  }

  return {
    joinChannel: joinChannel as JoinChannel
  };
}
