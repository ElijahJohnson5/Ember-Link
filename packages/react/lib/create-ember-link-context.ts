import type {
  Channel,
  ChannelConfig,
  CreateClientOptions,
  DefaultCustomMessageData,
  DefaultPresence,
  EmberClient,
  Status,
  User
} from '@ember-link/core';
import type { ComponentType, PropsWithChildren } from 'react';
import { EmberLinkProvider, useClient, useClientOrNull } from './ember-link-provider';
import {
  ChannelProvider,
  useChannel,
  useChannelOrNull,
  useMyPresence,
  useCustomMessage
} from './channel-provider';
import { useOthers } from './others';
import { useStatus } from './status';
import {
  useArrayStorage,
  useMapStorage,
  type ArrayStorageHookResult,
  type MapStorageHookResult
} from './storage';

/**
 * The typed bundle returned by `createEmberLinkContext<P, C>()`. Every
 * member here is exactly the same runtime function as the top-level
 * export from `@ember-link/react` — only the types differ. Pass your
 * Presence and Custom shapes as the generic parameters and use the
 * returned bundle anywhere you'd use the unscoped equivalents.
 */
export interface EmberLinkContextApi<
  P extends Record<string, unknown>,
  C extends Record<string, unknown>
> {
  EmberLinkProvider: ComponentType<PropsWithChildren<CreateClientOptions>>;
  ChannelProvider: (
    props: PropsWithChildren<{
      channelName: string;
      options?: ChannelConfig<P>['options'];
    }>
  ) => ReturnType<typeof ChannelProvider>;
  useClient: () => EmberClient<P, C>;
  useClientOrNull: () => EmberClient<P, C> | null;
  useChannel: () => Channel<P, C>;
  useChannelOrNull: () => Channel<P, C> | null;
  useMyPresence: () => readonly [P | null, (presence: P) => void];
  useOthers: () => User<P>[];
  useStatus: () => Status;
  useCustomMessage: (callback: (message: C) => void) => (data: C) => void;
  useArrayStorage: <T>(name: string) => ArrayStorageHookResult<T>;
  useMapStorage: <K extends string, V>(name: string) => MapStorageHookResult<K, V>;
}

/**
 * Returns a Presence/Custom-typed bundle of the Ember Link React API.
 *
 * Intended for **library authors** who build on top of `@ember-link/react`
 * and want to expose typed hooks/providers without forcing their
 * consumers to globally augment the `EmberLink` interface. The runtime
 * is identical to the top-level exports — only the TypeScript types
 * are scoped.
 *
 * ```ts
 * // acme-chat/index.ts
 * import { createEmberLinkContext } from '@ember-link/react';
 *
 * type ChatPresence = { isTyping: boolean };
 * type ChatCustom = { kind: 'reaction'; emoji: string };
 *
 * const ctx = createEmberLinkContext<ChatPresence, ChatCustom>();
 *
 * export const ChatProvider = ctx.EmberLinkProvider;
 * export const useChatPresence = ctx.useMyPresence;
 * // ...
 * ```
 *
 * For app-level (not library) typing, prefer module augmentation via
 * the global `EmberLink` interface — see the JSDoc on it in
 * `@ember-link/core` for an example.
 */
export function createEmberLinkContext<
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(): EmberLinkContextApi<P, C> {
  return {
    EmberLinkProvider: EmberLinkProvider as ComponentType<PropsWithChildren<CreateClientOptions>>,
    ChannelProvider: ChannelProvider as EmberLinkContextApi<P, C>['ChannelProvider'],
    useClient: () => useClient<P, C>(),
    useClientOrNull: () => useClientOrNull<P, C>(),
    useChannel: () => useChannel<P, C>(),
    useChannelOrNull: () => useChannelOrNull<P, C>(),
    useMyPresence: () => useMyPresence<P, C>(),
    useOthers: () => useOthers<P, C>(),
    useStatus: () => useStatus<P, C>(),
    useCustomMessage: (callback) => useCustomMessage<P, C>(callback),
    useArrayStorage: <T>(name: string) => useArrayStorage<T, P, C>(name),
    useMapStorage: <K extends string, V>(name: string) => useMapStorage<K, V, P, C>(name)
  };
}
