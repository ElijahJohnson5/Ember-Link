import {
  createBufferedEventEmitter,
  createEventEmitter,
  Observable
} from '@ember-link/event-emitter';
import { ManagedPresence } from './presence';
import { ManagedSocket, Status } from './socket-client';
import {
  decodeServerMessage,
  ServerMessage,
  type StorageSyncMessage,
  type StorageUpdateMessage
} from '@ember-link/protocol';
import { ManagedOthers, OtherEvents } from './others';
import { IStorage, IStorageProvider, MessageEvents } from '@ember-link/storage';
import { DefaultCustomMessageData, DefaultPresence, type User } from './index';
import { AuthValue } from './auth';
import { IWebSocketInstance } from './types';
import { watch } from 'alien-deepsignals';
import { signal, effect } from 'alien-signals';

export interface ChannelConfig<P extends Record<string, unknown> = DefaultPresence> {
  channelName: string;
  baseUrl: string;
  authenticate: () => Promise<AuthValue>;
  createWebSocket: (authValue: AuthValue) => IWebSocketInstance;
  options?: {
    initialPresence?: P;

    // TODO: Implement Loro crdt and Automerge
    // https://github.com/loro-dev/loro
    // https://automerge.org/
    storageProvider?: IStorageProvider;

    autoConnect?: boolean;

    /**
     * Minimum interval (ms) between presence sends. The latest state is
     * coalesced into a single message per interval — first update sends
     * immediately, subsequent updates within the window are deferred and
     * the most recent one fires on a trailing timer. 0 or omitted means
     * no throttling (every updatePresence call is sent). Pass 16 for
     * ~60Hz, 33 for ~30Hz.
     */
    presenceThrottle?: number;
  };
}

export type Channel<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> = {
  updatePresence: (state: P) => void;
  sendCustomMessage: (data: C) => void;
  hasStorage: () => boolean;
  /**
   * Returns the configured storage handle, or `null` if no
   * `storageProvider` was supplied at client or channel creation.
   */
  getStorage: () => IStorage | null;
  getStatus: () => Status;
  getOthers: () => User<P>[];
  getName: () => string;
  getPresence: () => P | null;
  destroy: () => void;
  connect: () => void;
  events: Observable<ChannelEvents<P, C>> & {
    others: Observable<OtherEvents<P>>;
  };
};

type ChannelEvents<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> = {
  presence: (self: P) => void;
  status: (status: Status) => void;
  others: (others: User<P>[]) => void;
  destroy: () => void;
  customMessage: (message: C) => void;
};

type YjsProviderEvents = {
  syncMessage: (message: StorageSyncMessage) => void;
  updateMessage: (message: StorageUpdateMessage) => void;
};

/**
 * Hooks the Yjs provider package uses to talk to a channel. Not part of the
 * public API — consume via {@link getChannelInternals} from
 * `@ember-link/core` (marked `@internal`).
 */
export interface ChannelInternals {
  yjs: {
    events: Observable<YjsProviderEvents>;
    sync: (data: StorageSyncMessage) => void;
    update: (data: StorageUpdateMessage) => void;
  };
}

type AnyChannel = Channel<Record<string, unknown>, Record<string, unknown>>;

const channelInternals = new WeakMap<AnyChannel, ChannelInternals>();

/**
 * @internal Used by `@ember-link/yjs-provider`. Not stable API.
 */
export function getChannelInternals(channel: AnyChannel): ChannelInternals | undefined {
  return channelInternals.get(channel);
}

interface ThrottledSender {
  (): void;
  cancel: () => void;
}

function createThrottledSender(send: () => void, intervalMs?: number): ThrottledSender {
  if (!intervalMs || intervalMs <= 0) {
    const fn = (() => send()) as ThrottledSender;
    fn.cancel = () => {};
    return fn;
  }

  let timer: ReturnType<typeof setTimeout> | null = null;
  let lastFire = 0;

  const fn = (() => {
    const now = Date.now();
    const elapsed = now - lastFire;

    if (elapsed >= intervalMs) {
      // Leading edge — fire right away
      lastFire = now;
      send();
      return;
    }

    // Trailing edge — schedule the latest state to fire when the window expires
    if (timer === null) {
      timer = setTimeout(() => {
        timer = null;
        lastFire = Date.now();
        send();
      }, intervalMs - elapsed);
    }
  }) as ThrottledSender;

  fn.cancel = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  };

  return fn;
}

export function createChannel<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({ options, ...config }: ChannelConfig<P>): Channel<P, C> {
  const managedSocket = new ManagedSocket<P, C>({ ...config });

  const otherEventEmitter = createEventEmitter<OtherEvents<P>>();
  const managedOthers = new ManagedOthers<P>(otherEventEmitter);
  const participantId = signal<string | null>(null);
  const status = signal<Status>('initial');

  const eventEmitter = createBufferedEventEmitter<ChannelEvents<P, C>>();
  const presence = new ManagedPresence(options?.initialPresence);

  const sendPresenceNow = () => managedSocket.message(presence.getPresenceMessage());
  const sendPresence = createThrottledSender(sendPresenceNow, options?.presenceThrottle);

  effect(() => {
    eventEmitter.emit('presence', presence.state());

    sendPresence();
  });

  watch(managedOthers.signal, (state: User<P>[]) => {
    eventEmitter.emit('others', state);
  });

  const storageEventEmitter = createEventEmitter<MessageEvents>();
  const yjsProviderEventEmitter = createEventEmitter<YjsProviderEvents>();

  managedSocket.events.subscribe('message', (e) => {
    let message: ServerMessage | null;

    if (typeof e.data === 'string') {
      try {
        message = JSON.parse(e.data);
      } catch {
        return;
      }
    } else {
      try {
        message = decodeServerMessage(new Uint8Array(e.data));
      } catch (e) {
        return;
      }
    }

    if (message) {
      if (message.tag === 'AssignIdMessage') {
        participantId(message.val.id);
      } else if (message.tag === 'ServerPresenceMessage') {
        managedOthers.setOther(
          message.val.id,
          message.val.clock,
          message.val.presence ? (JSON.parse(message.val.presence) as P) : null
        );
      } else if (message.tag === 'InitialPresenceMessage') {
        for (const presence of message.val.presences) {
          managedOthers.setOther(
            presence.id,
            presence.clock,
            presence.presence ? (JSON.parse(presence.presence) as P) : null
          );
        }
      } else if (message.tag === 'StorageUpdateMessage') {
        storage?.applyUpdate(new Uint8Array(message.val.update));
      } else if (message.tag === 'StorageSyncMessage') {
        storageEventEmitter.emit('message', message.val);
      } else if (message.tag === 'ProviderSyncMessage') {
        yjsProviderEventEmitter.emit('syncMessage', message.val);
      } else if (message.tag === 'ProviderUpdateMessage') {
        yjsProviderEventEmitter.emit('updateMessage', message.val);
      } else if (message.tag === 'CustomMessage') {
        eventEmitter.emit('customMessage', JSON.parse(message.val.message));
      }
    }
  });

  managedSocket.events.subscribe('disconnect', () => {
    // Clear the others when we disconnect
    managedOthers.clear();
  });

  managedSocket.events.subscribe('statusChange', (newStatus) => {
    status(newStatus);
    eventEmitter.emit('status', newStatus);
  });

  managedSocket.events.subscribe('open', () => {
    options?.storageProvider?.sync(storageEventEmitter.observable, {
      message: (data) => {
        managedSocket.message({
          tag: 'StorageSyncMessage',
          val: {
            ...data
          }
        });
      }
    });

    managedSocket.message(presence.getPresenceMessage());
  });

  const storage = options?.storageProvider?.getStorage() ?? null;

  if (storage) {
    storage.events.subscribe('update', (event) => {
      managedSocket.message({
        tag: 'StorageUpdateMessage',
        val: {
          update: event.buffer as ArrayBuffer
        }
      });
    });
  }

  function getStorage(): IStorage | null {
    return storage;
  }

  function hasStorage() {
    return storage !== null;
  }

  eventEmitter.pause('status');

  // Resume status event emitter on next microtask so that consumers can set up listeners in time
  const timeout = setTimeout(() => {
    eventEmitter.resume('status');
  }, 0);

  function destroy() {
    clearTimeout(timeout);
    sendPresence.cancel();
    eventEmitter.emit('destroy');
    managedOthers.destroy();
    presence.destroy();
    managedSocket.destroy();
  }

  function updatePresence(state: P) {
    const oldState = presence.state();

    presence.state({ ...oldState, ...state });
  }

  function sendCustomMessage(data: C) {
    managedSocket.message({ tag: 'CustomMessage', val: { message: JSON.stringify(data) } });
  }

  function getStatus() {
    return status();
  }

  function syncYDoc(data: StorageSyncMessage) {
    managedSocket.message({
      tag: 'ProviderSyncMessage',
      val: {
        ...data
      }
    });
  }

  function updateYDoc(data: StorageUpdateMessage) {
    managedSocket.message({
      tag: 'ProviderUpdateMessage',
      val: {
        ...data
      }
    });
  }

  const shouldConnect = options?.autoConnect ?? true;

  if (shouldConnect) {
    managedSocket.connect();
  }

  const channel: Channel<P, C> = {
    updatePresence,
    sendCustomMessage,
    getStorage,
    hasStorage,
    getStatus,
    getOthers: () => managedOthers.signal as User<P>[],
    getPresence: () => presence.state(),
    getName: () => config.channelName,
    destroy,
    connect: () => managedSocket.connect(),
    events: {
      ...eventEmitter.observable,
      others: otherEventEmitter.observable
    }
  };

  channelInternals.set(channel as unknown as AnyChannel, {
    yjs: {
      events: yjsProviderEventEmitter.observable,
      sync: syncYDoc,
      update: updateYDoc
    }
  });

  return channel;
}
