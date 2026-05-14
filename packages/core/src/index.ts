import { IStorageProvider } from '@ember-link/storage';
import { ChannelConfig } from './channel';

/**
 * Type-level interface for declaring app-wide custom shapes for
 * `Presence` and `Custom` (custom messages).
 *
 * Declare this once anywhere in your app (commonly in a `.d.ts` at the
 * src root) and every public API in Ember Link — `createClient`,
 * `useMyPresence`, `<ChannelProvider>`, `getChannelContext`, etc. —
 * will default to your shapes without you having to thread generic
 * parameters through every call site:
 *
 * ```ts
 * declare global {
 *   interface EmberLink {
 *     Presence: { cursor: { x: number; y: number } | null };
 *     Custom:
 *       | { kind: 'ping' }
 *       | { kind: 'reaction'; emoji: string };
 *   }
 * }
 * ```
 *
 * You can still pass a generic explicitly at any call site to override
 * for that call (e.g. `useMyPresence<OtherPresence>()`) — the per-call
 * generic doesn't need to extend the augmented type.
 *
 * If you're a library author building **on top of** Ember Link and
 * don't want to force app-wide augmentation on your consumers, use
 * `createEmberLinkContext<P, C>()` (from `@ember-link/react`) to scope
 * the types to your library instead.
 *
 * Only `Presence` and `Custom` are consumed today, but the index
 * signature allows future keys without a breaking change.
 */
declare global {
  export interface EmberLink {
    [key: string]: unknown;
  }
}

type ExtendableTypes = 'Presence' | 'Custom';

type GetOverride<K extends ExtendableTypes> = unknown extends EmberLink[K]
  ? Record<string, unknown>
  : EmberLink[K];

/**
 * Resolves to the user-declared `EmberLink['Presence']` if they've
 * augmented the global `EmberLink` interface, otherwise to a permissive
 * `Record<string, unknown>`. Used as the default for every `P` generic
 * across the SDKs.
 */
export type DefaultPresence = GetOverride<'Presence'>;

/**
 * Resolves to the user-declared `EmberLink['Custom']` if they've
 * augmented the global `EmberLink` interface, otherwise to a permissive
 * `Record<string, unknown>`. Used as the default for every `C` generic
 * across the SDKs.
 */
export type DefaultCustomMessageData = GetOverride<'Custom'>;

export type ChannelOptions<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> = ChannelConfig<S, P>['options'];
export { IStorageProvider };

export { createClient, CreateClientOptions, EmberClient } from './client';
export { User } from './user';
export { Channel, ChannelConfig } from './channel';
export { Status } from './socket-client';
export * from '@ember-link/storage';
