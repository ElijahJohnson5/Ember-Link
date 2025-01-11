import { createClient } from './client.js';

const client = createClient({
  baseUrl: 'ws://localhost:9000'
});

const channel = client.joinChannel('test');

channel.events.myPresence.subscribe('update', () => {
  console.log('Update');
});
