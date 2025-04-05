import { IStorageProvider } from '@ember-link/storage';
import { ChannelConfig } from './channel';

declare global {
  export interface EmberLink {
    [key: string]: unknown;
  }
}

type ExtendableTypes = 'Presence' | 'Custom';

type GetOverride<K extends ExtendableTypes> = unknown extends EmberLink[K]
  ? Record<string, unknown>
  : EmberLink[K];

export type DefaultPresence = GetOverride<'Presence'>;

export type DefaultCustomMessageData = GetOverride<'Custom'>;

export type ChannelOptions<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> = ChannelConfig<S, P>['options'];
export { IStorageProvider };

export { createClient, CreateClientOptions, EmberClient } from './client';
export { User } from './user';
export { Channel, ChannelConfig } from './channel';
export * from '@ember-link/storage';
