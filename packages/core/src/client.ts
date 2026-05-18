import { IStorageProvider } from '@ember-link/storage';
import { Channel, ChannelConfig, createChannel } from './channel';
import { DefaultCustomMessageData, DefaultPresence } from './index';
import { AuthEndpoint, AuthValue, createAuth } from './auth';
import { IWebSocket, IWebSocketInstance, WebSocketNotFoundError } from './types';
import type { StorageType } from '@ember-link/protocol';

/**
 * Inputs the default {@link defaultSocketFactory} consumes. Custom factories
 * receive the same payload, so they can reuse `channelName`, `authValue`,
 * and `storageType` without re-deriving them.
 */
export interface SocketFactoryParams {
  baseUrl: string;
  channelName: string;
  authValue: AuthValue;
  storageType?: StorageType;
  polyfills?: { websocket?: IWebSocket };
}

export type SocketFactory = (params: SocketFactoryParams) => IWebSocketInstance;

/**
 * Default WebSocket factory used by `createClient` when no `socketFactory`
 * override is supplied. Exported so custom factories can compose with it —
 * e.g. derive the URL, then wrap the returned socket with telemetry.
 */
export const defaultSocketFactory: SocketFactory = ({
  baseUrl,
  channelName,
  authValue,
  storageType,
  polyfills
}) => {
  const ws = polyfills?.websocket
    ? polyfills.websocket
    : typeof WebSocket === 'undefined'
      ? undefined
      : WebSocket;

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

  if (storageType) {
    url.searchParams.set('storage_type', storageType);
  }

  url.pathname = '/ws';

  return new ws(url.toString());
};

export interface CreateClientOptions {
  baseUrl: string;
  authEndpoint?: AuthEndpoint;
  multiTenant?: {
    tenantId: string;
  };
  polyfills?: {
    websocket?: IWebSocket;
  };
  jwtSignerPublicKey?: string;

  /**
   * Custom transport factory. Useful for attaching auth headers, routing
   * through a proxy, or wrapping the returned socket with instrumentation.
   * Compose with {@link defaultSocketFactory} to reuse the URL builder.
   */
  socketFactory?: SocketFactory;

  /**
   * Default storage provider applied to every `joinChannel` call that
   * doesn't supply its own. Per-channel `options.storageProvider` wins.
   */
  storageProvider?: IStorageProvider;

  /**
   * Default presence throttle (ms) applied to every `joinChannel` call.
   * Per-channel `options.presenceThrottle` wins. See
   * {@link ChannelConfig['options']['presenceThrottle']} for semantics.
   */
  presenceThrottle?: number;
}

type JoinChannel<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> = (
  channelName: string,
  options?: ChannelConfig<P>['options']
) => { channel: Channel<P, C>; leave: () => void };

export interface EmberClient<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> {
  joinChannel: JoinChannel<P, C>;
  /**
   * Tear down every channel this client created and close their
   * WebSocket connections. After `destroy()`, calling `joinChannel`
   * on the same client instance has undefined behavior — create a new
   * client instead.
   */
  destroy: () => void;
}

export function createClient<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({
  baseUrl,
  authEndpoint,
  jwtSignerPublicKey,
  multiTenant,
  polyfills,
  socketFactory,
  storageProvider: clientStorageProvider,
  presenceThrottle: clientPresenceThrottle
}: CreateClientOptions): EmberClient<P, C> {
  const channels = new Map<string, { channel: Channel<P, C>; unsubs: Set<() => void> }>();
  const auth = createAuth({
    authEndpoint,
    jwtSignerPublicKey,
    multiTenant,
    onAuthenticated: (_value) => {
      // TODO: Set user
    }
  });

  const factory = socketFactory ?? defaultSocketFactory;

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

  function joinChannel(channelName: string, options?: ChannelConfig<P>['options']) {
    if (channels.has(channelName)) {
      return borrowChannel(channels.get(channelName)!);
    }

    // Client-level defaults are merged in here; per-channel options always win.
    const mergedOptions: ChannelConfig<P>['options'] = {
      ...options,
      storageProvider: options?.storageProvider ?? clientStorageProvider,
      presenceThrottle: options?.presenceThrottle ?? clientPresenceThrottle
    };

    const channel = createChannel<P, C>({
      channelName,
      baseUrl,
      authenticate: async () => {
        return auth.getAuthValue(channelName);
      },
      createWebSocket: (authValue) =>
        factory({
          baseUrl,
          channelName,
          authValue,
          storageType: mergedOptions?.storageProvider?.type,
          polyfills
        }),
      options: mergedOptions
    });

    const channelWithUnsubs = {
      channel,
      unsubs: new Set<() => void>()
    };

    channels.set(channelName, channelWithUnsubs);

    return borrowChannel(channelWithUnsubs);
  }

  function destroy() {
    for (const { channel } of channels.values()) {
      channel.destroy();
    }
    channels.clear();
  }

  return {
    joinChannel: joinChannel as JoinChannel<P, C>,
    destroy
  };
}
