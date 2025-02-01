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

export { createClient } from './client';
export { User } from './user';
