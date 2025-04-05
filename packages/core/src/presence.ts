import { ClientMessage } from '@ember-link/protocol';
import $, { type Observable as Signal } from 'oby';
import { DefaultPresence } from '.';

export const outdatedTimeout = 30000;

export type PresenceEvents<P extends Record<string, unknown> = DefaultPresence> = {
  update: (state: P) => void;
};

export interface MetaClientState {
  clock: number;
  lastUpdated: number;
}

export class ManagedPresence<P extends Record<string, unknown> = DefaultPresence> {
  public state: Signal<P>;

  private meta: MetaClientState;
  // TODO cleanup interval when destroyed or something like that
  private checkInterval: ReturnType<typeof setInterval>;

  constructor(initialPresence?: P) {
    this.state = $<P>(initialPresence ?? ({} as P), {
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

      if (this.state && outdatedTimeout / 2 >= now - this.meta.lastUpdated) {
        this.state((oldState) => {
          return { ...oldState };
        });
      }
    }, outdatedTimeout / 10);
  }

  getPresenceMessage(): Extract<ClientMessage<P, Record<string, unknown>>, { type: 'presence' }> {
    return {
      clock: this.meta.clock,
      data: this.state(),
      type: 'presence'
    };
  }

  destroy() {
    clearInterval(this.checkInterval);
  }
}
