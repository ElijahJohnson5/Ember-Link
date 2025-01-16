import { ClientMessage, PresenceState } from '@ember-link/protocol';
import $, { ObservableReadonly, type Observable as Signal } from 'oby';

export const outdatedTimeout = 30000;

export type PresenceEvents = {
  update: (state: PresenceState) => void;
};

export interface MetaClientState {
  clock: number;
  lastUpdated: number;
}
export class ManagedPresence {
  public state: Signal<PresenceState>;

  private meta: MetaClientState;
  private clock: ObservableReadonly<number>;
  // TODO cleanup interval when destroyed or something like that
  private checkInterval: ReturnType<typeof setInterval>;

  constructor(initialPresence?: PresenceState) {
    this.state = $<PresenceState>(initialPresence ?? { custom: {} }, {
      equals: () => false
    });
    this.meta = {
      clock: 0,
      lastUpdated: new Date().getTime()
    };

    $.effect(() => {
      this.state();

      this.meta.clock++;
      this.meta.lastUpdated = new Date().getTime();
    });

    this.checkInterval = setInterval(() => {
      const now = new Date().getTime();

      if (this.state && outdatedTimeout / 2 <= now - this.meta.lastUpdated) {
        this.state((oldState) => {
          return { ...oldState };
        });
      }
    }, outdatedTimeout / 10);
  }

  getNewPresenceMessage(): Extract<ClientMessage, { type: 'presence' }> {
    return {
      clock: this.meta.clock,
      data: this.state(),
      type: 'presence'
    };
  }
}
