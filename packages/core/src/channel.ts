import {
  createBufferedEventEmitter,
  createEventEmitter,
  Observable
} from '@ember-link/event-emitter';
import { ManagedPresence } from './presence';
import { ManagedSocket, Status } from './socket-client';
import {
  ServerMessage,
  type StorageSyncMessage,
  type StorageUpdateMessage
} from '@ember-link/protocol';
import $ from 'oby';
import { ManagedOthers, OtherEvents } from './others';
import { IStorage, IStorageProvider, MessageEvents } from '@ember-link/storage';
import { DefaultCustomMessageData, DefaultPresence, type User } from './index';
import { AuthValue } from './auth';

export interface ChannelConfig<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> {
  channelName: string;
  baseUrl: string;
  authenticate: () => Promise<AuthValue>;
  createWebSocket: (authValue: AuthValue) => WebSocket;
  options?: {
    initialPresence?: P;

    // TODO: Implement Loro crdt and Automerge
    // https://github.com/loro-dev/loro
    // https://automerge.org/
    storageProvider?: S;

    autoConnect?: boolean;
  };
}

export type Channel<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
> = {
  updatePresence: (state: P) => void;
  sendCustomMessage: (data: C) => void;
  hasStorage: () => boolean;
  getStorage: () => IStorage;
  getStatus: () => Status;
  getOthers: () => User<P>[];
  getName: () => string;
  getPresence: () => P | null;
  destroy: () => void;
  connect: () => void;
  updateYDoc: (data: StorageUpdateMessage) => void;
  syncYDoc: (data: StorageSyncMessage) => void;
  events: Observable<ChannelEvents<P, C>> & {
    others: Observable<OtherEvents<P>>;
    yjsProvider: Observable<YjsProviderEvents>;
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
  customMessage: (message: Extract<ServerMessage<P, C>, { type: 'custom' }>['data']) => void;
};

type YjsProviderEvents = {
  syncMessage: (message: StorageSyncMessage) => void;
  updateMessage: (message: StorageUpdateMessage) => void;
};

export function createChannel<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({ options, ...config }: ChannelConfig<S, P>): Channel<P, C> {
  const managedSocket = new ManagedSocket<P, C>({ ...config });

  const otherEventEmitter = createEventEmitter<OtherEvents<P>>();
  const managedOthers = new ManagedOthers<P>(otherEventEmitter);
  const participantId = $<string | null>(null);
  const status = $<Status>('initial');

  const eventEmitter = createBufferedEventEmitter<ChannelEvents<P, C>>();
  const presence = new ManagedPresence(options?.initialPresence);

  $.effect(() => {
    eventEmitter.emit('presence', presence.state());

    managedSocket.message(presence.getPresenceMessage());
  });

  $.effect(() => {
    eventEmitter.emit('others', managedOthers.signal());
  });

  const storageEventEmitter = createEventEmitter<MessageEvents>();
  const yjsProviderEventEmitter = createEventEmitter<YjsProviderEvents>();

  managedSocket.events.subscribe('message', (e) => {
    if (typeof e.data === 'string') {
      // We know the data is a string if we get here
      const message: ServerMessage<P, C> = JSON.parse(e.data as string);

      if (message.type === 'assignId') {
        participantId(message.id);
      } else if (message.type === 'presence') {
        managedOthers.setOther(message.id, message.clock, message.data);
      } else if (message.type === 'initialPresence') {
        for (const presence of message.presences) {
          managedOthers.setOther(presence.id, presence.clock, presence.data);
        }
      } else if (message.type === 'storageUpdate') {
        storage?.applyUpdate(Uint8Array.from(message.update));
      } else if (message.type === 'storageSync') {
        storageEventEmitter.emit('message', message);
      } else if (message.type === 'providerSync') {
        yjsProviderEventEmitter.emit('syncMessage', message);
      } else if (message.type === 'providerUpdate') {
        yjsProviderEventEmitter.emit('updateMessage', message);
      } else if (message.type === 'custom') {
        eventEmitter.emit('customMessage', message.data);
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
          type: 'storageSync',
          ...data
        });
      }
    });

    managedSocket.message(presence.getPresenceMessage());
  });

  const storage = options?.storageProvider?.getStorage();

  if (storage) {
    storage.events.subscribe('update', (event) => {
      managedSocket.message({
        type: 'storageUpdate',
        update: Array.from(event)
      });
    });
  }

  function getStorage() {
    if (storage) {
      return storage;
    }

    throw new Error('A storage provider must be configured to use storage');
  }

  function hasStorage() {
    return Boolean(storage);
  }

  eventEmitter.pause('status');

  // Resume status event emitter on next microtask so that consumers can set up listeners in time
  const timeout = setTimeout(() => {
    eventEmitter.resume('status');
  }, 0);

  function destroy() {
    clearTimeout(timeout);
    eventEmitter.emit('destroy');
    managedOthers.destroy();
    presence.destroy();
    managedSocket.destroy();
  }

  function updatePresence(state: P) {
    presence.state((oldState) => {
      return {
        ...oldState,
        ...state
      };
    });
  }

  function sendCustomMessage(data: C) {
    managedSocket.message({ type: 'custom', data });
  }

  function getStatus() {
    return status();
  }

  function syncYDoc(data: StorageSyncMessage) {
    managedSocket.message({
      type: 'providerSync',
      ...data
    });
  }

  function updateYDoc(data: StorageUpdateMessage) {
    managedSocket.message({
      type: 'providerUpdate',
      ...data
    });
  }

  const shouldConnect = options?.autoConnect ?? true;

  if (shouldConnect) {
    managedSocket.connect();
  }

  return {
    updatePresence,
    sendCustomMessage,
    getStorage,
    hasStorage,
    getStatus,
    getOthers: () => managedOthers.signal(),
    getPresence: () => presence.state(),
    getName: () => config.channelName,
    updateYDoc,
    syncYDoc,
    destroy,
    connect: () => managedSocket.connect(),
    events: {
      ...eventEmitter.observable,
      others: otherEventEmitter.observable,
      yjsProvider: yjsProviderEventEmitter.observable
    }
  };
}
