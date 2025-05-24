import { ObservableReadonly } from 'oby';
import { MetaClientState, outdatedTimeout } from './presence';
import { ReactiveMap } from './reactive-map';
import $ from 'oby';
import { computed } from 'alien-signals';
import { Emitter } from '@ember-link/event-emitter';
import { DefaultPresence } from './index';
import { User } from './user';
import { DeepSignal, deepSignal, Shallow, shallow } from 'alien-deepsignals';

export type OtherEvents<P extends Record<string, unknown> = DefaultPresence> = {
  leave: (user: User<P>) => void;
  join: (user: User<P>) => void;
  update: (user: User<P>) => void;
  reset: () => void;
};

export class ManagedOthers<P extends Record<string, unknown> = DefaultPresence> {
  private emitter: Emitter<OtherEvents>;
  // TODO handle cleaning up the interval
  private checkInterval: ReturnType<typeof setInterval>;

  readonly states: Map<string, number>;
  readonly meta: Map<string, MetaClientState>;

  public readonly signal: DeepSignal<Shallow<User<P>>[]>;

  constructor(emitter: Emitter<OtherEvents>) {
    this.states = new Map();
    this.meta = new Map();

    this.emitter = emitter;
    this.signal = deepSignal<Shallow<User<P>>[]>([]);

    // this.signal = computed(() => {
    //   return Array.from(this.states.entries()).map(([clientId, state]) => {
    //     return { ...state, clientId: clientId };
    //   });
    // });

    this.checkInterval = setInterval(() => {
      const now = new Date().getTime();

      this.meta.forEach((meta, clientId) => {
        if (outdatedTimeout / 2 <= now - meta.lastUpdated && this.states.has(clientId)) {
          const state = this.states.get(clientId)!;

          this.states.delete(clientId);
          this.emitter.emit('leave', { ...this.signal[state], clientId: clientId });
        }
      });
    }, outdatedTimeout / 10);
  }

  setOther(clientId: string, clock: number, state: P | null) {
    const clientMeta = this.meta.get(clientId);
    const prevState = this.states.get(clientId);
    const currClock = clientMeta === undefined ? 0 : clientMeta.clock;

    if (currClock < clock || (currClock === clock && !state && prevState)) {
      if (!state) {
        this.states.delete(clientId);
        if (prevState) {
          this.signal.splice(prevState, 1);
        }
      } else {
        if (prevState) {
          // @ts-ignore
          this.signal[prevState] = shallow({ ...state, clientId });
        } else {
          const length = this.signal.push(shallow({ ...state, clientId }));
          this.states.set(clientId, length - 1);
        }
      }

      this.meta.set(clientId, {
        clock,
        lastUpdated: new Date().getTime()
      });

      if (clientMeta === undefined && state !== null) {
        this.emitter.emit('join', { ...state, clientId: clientId });
      } else if (clientMeta !== undefined && state === null) {
        this.emitter.emit('leave', { clientId: clientId });
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

  destroy() {
    clearInterval(this.checkInterval);
  }
}
