declare global {
  export interface EmberLink {
    [key: string]: unknown;
  }
}

type ExtendableTypes = 'Presence';

type GetOverride<K extends ExtendableTypes> = unknown extends EmberLink[K]
  ? Record<string, unknown>
  : EmberLink[K];

export type DefaultPresence = GetOverride<'Presence'>;

export { createClient } from './client.js';
export { User } from './user.js';

// const client = createClient({
//   baseUrl: 'ws://localhost:9000'
// });

// const channel = client.joinChannel('test');

// channel.events.others.subscribe('join', () => {
//   console.log('User joined 1');
// });

// channel.events.subscribe('presence', () => {
//   console.log('Presence 1');
// });

// const client2 = createClient({
//   baseUrl: 'ws://localhost:9000'
// });

// const channel2 = client2.joinChannel('test');

// channel2.events.others.subscribe('join', () => {
//   console.log('User joined 2');
// });

// channel2.events.subscribe('presence', () => {
//   console.log('Presence 2');
// });
