import { ManagedSocket } from "./socket-client.js";

interface SpaceConfig {
  channelName: string;
  baseUrl: string;
}

export type Channel = {
  events: {};
};

export function createChannel(config: SpaceConfig): Channel {
  const managedSocket = new ManagedSocket(
    `${config.baseUrl}/channel/${config.channelName}`,
  );

  managedSocket.events.subscribe("message", (e) => {
    console.log(e.data);
  });

  managedSocket.connect();

  return;
}
