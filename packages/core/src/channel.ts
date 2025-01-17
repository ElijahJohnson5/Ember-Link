import { createEventEmitter, Observable } from '@ember-link/event-emitter';
import { ManagedPresence } from './presence.js';
import { ManagedSocket } from './socket-client.js';
import { ServerMessage } from '@ember-link/protocol';
import $ from 'oby';
import { ManagedOthers, OtherEvents } from './others.js';
import { IStorage, IStorageProvider } from '@ember-link/storage';
import { DefaultPresence } from './index.js';

export interface ChannelConfig<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> {
  channelName: string;
  baseUrl: string;
  options?: {
    initialPresence?: P;

    // TODO: Implement Loro crdt and Automerge
    // https://github.com/loro-dev/loro
    // https://automerge.org/
    storageProvider?: S;
  };
}

export type Channel<P extends Record<string, unknown> = DefaultPresence> = {
  updatePresence: (state: P) => void;
  getStorage: () => IStorage;
  events: Observable<ChannelEvents> & {
    others: Observable<OtherEvents>;
  };
};

type ChannelEvents<P extends Record<string, unknown> = DefaultPresence> = {
  presence: (self: P) => void;
  others: (others: Array<P>) => void;
};

export function createChannel<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
>({
  ...config
}: ChannelConfig<S, P>): {
  channel: Channel;
  leave: () => void;
} {
  const managedSocket = new ManagedSocket(`${config.baseUrl}/channel/${config.channelName}`);

  const otherEventEmitter = createEventEmitter<OtherEvents>();
  const managedOthers = new ManagedOthers(otherEventEmitter);
  const participantId = $<string | null>(null);

  const eventEmitter = createEventEmitter<ChannelEvents>();
  const presence = new ManagedPresence(config.options?.initialPresence);

  $.effect(() => {
    eventEmitter.emit('presence', presence.state());

    managedSocket.message(presence.getPresenceMessage());
  });

  $.effect(() => {
    eventEmitter.emit('others', managedOthers.signal());
  });

  managedSocket.events.subscribe('message', (e) => {
    if (typeof e.data === 'string') {
      // We know the data is a string if we get here
      const message: ServerMessage<P> = JSON.parse(e.data as string);

      if (message.type === 'assignId') {
        participantId(message.id);
      } else if (message.type === 'presence') {
        managedOthers.setOther(message.id, message.clock, message.data);
      } else if (message.type === 'initialPresence') {
        for (const presence of message.presences) {
          managedOthers.setOther(presence.id, presence.clock, presence.data);
        }
      } else if (message.type === 'storageUpdate') {
        storage.applyUpdate(Uint8Array.from(message.update));
      }
    }
  });

  managedSocket.events.subscribe('disconnect', () => {
    // Clear the others when we disconnect
    managedOthers.clear();
  });

  managedSocket.events.subscribe('open', () => {
    managedSocket.message(presence.getPresenceMessage());
  });

  const storage = config.options?.storageProvider?.getStorage();

  if (storage) {
    storage.events.subscribe('update', (event) => {
      managedSocket.message({
        type: 'storageUpdate',
        update: Array.from(event)
      });
    });
  }

  managedSocket.connect();

  function getStorage() {
    if (storage) {
      return storage;
    }

    throw new Error('A storage provider must be configured to use storage');
  }

  function leave() {
    managedSocket.destroy();
  }

  function updatePresence(state: P) {
    presence.state(state);
  }

  return {
    channel: {
      updatePresence,
      getStorage,
      events: { ...eventEmitter.observable, others: otherEventEmitter.observable }
    },
    leave
  };
}
