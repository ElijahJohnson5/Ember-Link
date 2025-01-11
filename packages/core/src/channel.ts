import { Observable } from './event-emitter.js';
import { ManagedPresence, PresenceEvents } from './presence.js';
import { ManagedSocket } from './socket-client.js';
import { ServerMessage } from '@ember-link/protocol';

interface SpaceConfig {
  channelName: string;
  baseUrl: string;
}

export type Channel = {
  events: {
    myPresence: Observable<PresenceEvents>;
  };
};

export function createChannel(config: SpaceConfig): { channel: Channel; leave: () => void } {
  const managedSocket = new ManagedSocket(`${config.baseUrl}/channel/${config.channelName}`);

  const presence = new ManagedPresence();

  managedSocket.events.subscribe('message', (e) => {
    if (e.type === 'text') {
      // We know the data is a string if we get here
      const message: ServerMessage = JSON.parse(e.data as string);

      if (message.type === 'updatePresence') {
        presence.update({});
      }

      console.log(message);
    }
  });

  managedSocket.connect();

  managedSocket.events.subscribe('open', () => {
    managedSocket.message({
      client_id: 123,
      type: 'newPresence'
    });
  });

  function leave() {}

  return {
    channel: {
      events: {
        myPresence: presence.events
      }
    },
    leave
  };
}
