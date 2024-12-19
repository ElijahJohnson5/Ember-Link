import { ManagedSocket } from "./socket-client.js";

interface SpaceConfig {
  spaceId: string;
  baseUrl: string;
}

export function createSpace(config: SpaceConfig) {
  const managedSocket = new ManagedSocket(config.baseUrl);

  managedSocket.events.message.subscribe("message", (e) => {
    console.log(e.data);
  });

  managedSocket.connect();
}
