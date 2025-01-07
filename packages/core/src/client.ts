// What the api should look like

import { Channel, createChannel } from "./channel.js";

/*

const client = createClient({
  baseUrl: url,
  apiKey: apiKey,
});

const channel = client.joinChannel(channelName);


let unsub = channel.events.subscribe("presence", (presenceData) => {
  // Do stuff with the presence
});




*/

interface CreateClientOptions {
  baseUrl: string;
  apiKey?: string;
}

export function createClient({ baseUrl, apiKey }: CreateClientOptions) {
  const channels = new Map<string, Channel>();

  function joinChannel(channelName: string) {
    if (channels.has(channelName)) {
      return channels.get(channelName);
    }

    const channel = createChannel({
      channelName,
      baseUrl,
    });

    channels.set(channelName, channel);

    return channel;
  }

  return {
    joinChannel,
  };
}