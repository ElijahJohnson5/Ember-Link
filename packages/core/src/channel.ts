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
import $ from 'oby';
import { ManagedOthers, OtherEvents } from './others';
import { IStorage, IStorageProvider, MessageEvents } from '@ember-link/storage';
import { DefaultCustomMessageData, DefaultPresence, type User } from './index';
import { AuthValue } from './auth';
import { IWebSocketInstance } from './types';
import { watch } from 'alien-deepsignals';

export interface ChannelConfig<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> {
  channelName: string;
  baseUrl: string;
  authenticate: () => Promise<AuthValue>;
  createWebSocket: (authValue: AuthValue) => IWebSocketInstance;
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
  customMessage: (message: C) => void;
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

  const storage = options?.storageProvider?.getStorage();

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

  return {
    updatePresence,
    sendCustomMessage,
    getStorage,
    hasStorage,
    getStatus,
    getOthers: () => [...managedOthers.signal],
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
