/* eslint-disable @typescript-eslint/no-explicit-any */
import type { Channel } from '@ember-link/core';
import { ObservableV2 } from 'lib0/observable';
import type { Doc } from 'yjs';

type MetaClientState = {
  clock: number;
  lastUpdated: number;
};

const Y_PRESENCE_KEY = '__yjs';
const Y_PRESENCE_ID_KEY = '__yjs_clientid';

interface Updates {
  added: number[];
  updated: number[];
  removed: number[];
}

export class Awareness extends ObservableV2<{
  change: (updates: Updates, type: string) => void;
  update: (updates: Updates, type: string) => void;
  destroy: (awareness: Awareness) => void;
}> {
  private channel: Channel;
  public doc: Doc;
  public states: Map<number, unknown> = new Map();
  public emberClientToClientMap: Map<string, number> = new Map();

  public meta: Map<number, MetaClientState> = new Map();

  public _checkInterval: number = 0;

  private readonly unsubscribers: Array<() => void> = [];

  constructor(doc: Doc, channel: Channel) {
    super();
    this.doc = doc;
    this.channel = channel;
    // Add the clientId to presence so we can map it to our own clientId later
    this.channel.updatePresence({
      [Y_PRESENCE_ID_KEY]: this.doc.clientID
    });

    this.unsubscribers.push(
      this.channel.events.others.subscribe('join', (user) => {
        if (!user[Y_PRESENCE_KEY]) {
          return;
        }

        const yjsClientId = user[Y_PRESENCE_ID_KEY] as number;

        this.emberClientToClientMap.set(user.clientId, yjsClientId);

        const updates: Updates = {
          added: [yjsClientId],
          updated: [],
          removed: []
        };

        this.emit('change', [updates, 'presence']);
        this.emit('update', [updates, 'presence']);
      })
    );

    this.unsubscribers.push(
      this.channel.events.others.subscribe('leave', (user) => {
        const yjsClientId = this.emberClientToClientMap.get(user.clientId);
        if (yjsClientId !== undefined) {
          const updates: Updates = {
            added: [],
            updated: [],
            removed: [yjsClientId]
          };

          this.emit('change', [updates, 'presence']);
          this.emit('update', [updates, 'presence']);
        }
      })
    );

    this.unsubscribers.push(
      this.channel.events.others.subscribe('update', (user) => {
        const yjsClientId = this.emberClientToClientMap.get(user.clientId);
        if (yjsClientId !== undefined) {
          const updates: Updates = {
            added: [],
            updated: [yjsClientId],
            removed: []
          };

          this.emit('change', [updates, 'presence']);
          this.emit('update', [updates, 'presence']);
        }
      })
    );
  }

  destroy(): void {
    this.emit('destroy', [this]);
    this.unsubscribers.forEach((unsub) => unsub());
    this.setLocalState(null);
    super.destroy();
  }

  getLocalState(): any | null {
    const presence = this.channel.getPresence();
    if (
      !presence ||
      Object.keys(presence).length === 0 ||
      typeof presence[Y_PRESENCE_KEY] === 'undefined'
    ) {
      return null;
    }
    return presence[Y_PRESENCE_KEY] as any | null;
  }

  setLocalState(state: any | null): void {
    const presence = this.channel.getPresence();
    if (state === null) {
      if (!presence) {
        return;
      }
      this.channel.updatePresence({ [Y_PRESENCE_KEY]: null });
      this.emit('update', [{ added: [], updated: [], removed: [this.doc.clientID] }, 'local']);
      return;
    }

    const yPresence = presence?.[Y_PRESENCE_KEY];
    const added = yPresence === undefined ? [this.doc.clientID] : [];
    const updated = yPresence === undefined ? [] : [this.doc.clientID];
    this.channel.updatePresence({
      [Y_PRESENCE_KEY]: {
        ...(yPresence || {}),
        ...(state || {})
      }
    });
    this.emit('update', [{ added, updated, removed: [] }, 'local']);
  }

  setLocalStateField(field: string, value: any | null): void {
    const presence = this.channel.getPresence()?.[Y_PRESENCE_KEY];
    const update = { [field]: value };
    this.channel.updatePresence({
      [Y_PRESENCE_KEY]: { ...(presence || {}), ...update }
    });
  }

  getStates(): Map<number, unknown> {
    const others = this.channel.getOthers();
    const states = others.reduce((acc, otherUser) => {
      const otherPresence = otherUser[Y_PRESENCE_KEY];
      const otherClientId = otherUser[Y_PRESENCE_ID_KEY] as number | undefined;
      if (otherPresence !== undefined && otherClientId !== undefined) {
        // set states of map clientId to yjs presence
        acc.set(otherClientId, otherPresence || {});
      }
      return acc;
    }, new Map<number, unknown>());

    // add this client's yjs presence to states (local client not represented in others)
    const localPresence = this.channel.getPresence()?.[Y_PRESENCE_KEY];
    if (localPresence !== undefined) {
      states.set(this.doc.clientID, localPresence);
    }
    return states;
  }
}
