import { createEventEmitter, Observable } from './event-emitter.js';
import { ManagedPresence } from './presence.js';
import { ManagedSocket } from './socket-client.js';
import { ServerMessage, PresenceState } from '@ember-link/protocol';
import $ from 'oby';
import { ManagedOthers, OtherEvents } from './others.js';
import { IStorageProvider } from '@ember-link/storage';

interface ChannelConfig<S extends IStorageProvider> {
  channelName: string;
  baseUrl: string;
  // TODO: Implement Loro crdt and Automerge
  // https://github.com/loro-dev/loro
  // https://automerge.org/
  storageProvider?: S;
}

export type Channel = {
  updatePresence: (state: PresenceState) => void;
  events: Observable<ChannelEvents> & {
    others: Observable<OtherEvents>;
  };
};

type ChannelEvents = {
  presence: (self: PresenceState) => void;
};

export function createChannel<S extends IStorageProvider>({
  ...config
}: ChannelConfig<S>): {
  channel: Channel;
  leave: () => void;
} {
  const managedSocket = new ManagedSocket(`${config.baseUrl}/channel/${config.channelName}`);

  const otherEventEmitter = createEventEmitter<OtherEvents>();
  const managedOthers = new ManagedOthers(otherEventEmitter);
  const participantId = $<string | null>(null);

  const eventEmitter = createEventEmitter<ChannelEvents>();
  const presence = new ManagedPresence();

  $.effect(() => {
    eventEmitter.emit('presence', presence.state());
  });

  managedSocket.events.subscribe('message', (e) => {
    if (typeof e.data === 'string') {
      // We know the data is a string if we get here
      const message: ServerMessage = JSON.parse(e.data as string);

      if (message.type === 'assignId') {
        participantId(message.id);
      } else if (message.type === 'newPresence') {
        managedOthers.setOther(message.id, message.clock, message.data);
      } else if (message.type === 'initialPresence') {
        for (const presence of message.presences) {
          managedOthers.setOther(presence.id, presence.clock, presence.data);
        }
      }
    }
  });

  managedSocket.events.subscribe('disconnect', () => {
    // Clear the others when we disconnect
    managedOthers.clear();
  });

  managedSocket.events.subscribe('open', () => {
    managedSocket.message(presence.getNewPresenceMessage());
  });

  managedSocket.connect();

  function leave() {}

  function updatePresence(state: PresenceState) {
    presence.state(state);
    managedSocket.message(presence.getNewPresenceMessage());
  }

  return {
    channel: {
      updatePresence,
      events: { ...eventEmitter.observable, others: otherEventEmitter.observable }
    },
    leave
  };
}
