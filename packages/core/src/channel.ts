import { effect, signal } from 'alien-signals';
import { encodeAwarenessUpdate, Presence } from './presence.js';
import { ManagedSocket } from './socket-client.js';
import { createEventEmitter } from './event-emitter.js';

interface SpaceConfig {
  channelName: string;
  baseUrl: string;
}

export type Channel = {
  events: Record<string, unknown>;
};

export function createChannel(config: SpaceConfig): Channel {
  const managedSocket = new ManagedSocket(`${config.baseUrl}/channel/${config.channelName}`);

  const myPresence = signal({
    name: 'test'
  });

  const emitter = createEventEmitter<{
    myPresence: (value: { name: string }) => void;
  }>();

  effect(() => {
    emitter.emit('myPresence', myPresence.get());
  });

  const presence = new Presence();

  const observable = presence.observable();

  observable.subscribe('update', ({ added, removed, updated }, _origin) => {
    const changedClients = added.concat(updated).concat(removed);

    const update = encodeAwarenessUpdate(presence, changedClients);

    managedSocket.message(update);
  });

  presence.setLocalState({
    custom: {
      x: 50,
      y: 50
    }
  });

  managedSocket.events.subscribe('message', (e) => {
    console.log(e.data);
    //observable.subscribe("")
  });

  managedSocket.connect();

  myPresence.set({ name: 'test 2' });

  return;
}
