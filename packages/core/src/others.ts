import { ObservableReadonly } from 'oby';
import { MetaClientState, outdatedTimeout } from './presence.js';
import { ReactiveMap } from './reactive-map.js';
import $ from 'oby';
import { Emitter } from '@ember-link/event-emitter';
import { PresenceState } from '@ember-link/protocol';

export type OtherEvents = {
  leave: (user: Record<string, unknown>) => void;
  join: (user: Record<string, unknown>) => void;
  update: (user: Record<string, unknown>) => void;
  reset: () => void;
};

export class ManagedOthers {
  private emitter: Emitter<OtherEvents>;
  // TODO handle cleaning up the interval
  private checkInterval: ReturnType<typeof setInterval>;

  readonly states: ReactiveMap<string, PresenceState>;
  readonly meta: ReactiveMap<string, MetaClientState>;

  public readonly signal: ObservableReadonly<PresenceState[]>;

  constructor(emitter: Emitter<OtherEvents>) {
    this.states = new ReactiveMap();
    this.meta = new ReactiveMap();

    this.emitter = emitter;
    this.signal = $.memo(() => {
      return Array.from(this.states.entries()).map(([clientId, state]) => {
        return state;
      });
    });

    this.checkInterval = setInterval(() => {
      const now = new Date().getTime();

      this.meta.forEach((meta, clientId) => {
        if (outdatedTimeout / 2 <= now - meta.lastUpdated && this.states.has(clientId)) {
          const state = this.states.get(clientId);

          this.states.delete(clientId);
          this.emitter.emit('leave', { ...state, clientId: clientId });
        }
      });
    }, outdatedTimeout / 10);
  }

  setOther(clientId: string, clock: number, state: PresenceState | null) {
    const clientMeta = this.meta.get(clientId);
    const prevState = this.states.get(clientId);
    const currClock = clientMeta === undefined ? 0 : clientMeta.clock;

    if (currClock < clock || (currClock === clock && !state && prevState)) {
      if (!state) {
        this.states.delete(clientId);
      } else {
        this.states.set(clientId, state);
      }

      this.meta.set(clientId, {
        clock,
        lastUpdated: new Date().getTime()
      });

      if (clientMeta === undefined && state !== null) {
        this.emitter.emit('join', { ...state, clientId: clientId });
      } else if (clientMeta !== undefined && state === null) {
        this.emitter.emit('leave', { ...state, clientId: clientId });
      } else if (state !== null) {
        this.emitter.emit('update', { ...state, clientId: clientId });
      }
    }
  }

  clear() {
    this.meta.clear();
    this.states.clear();
    this.emitter.emit('reset');
  }
}
