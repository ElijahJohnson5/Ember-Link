import { ClientMessage } from '@ember-link/protocol';
import { effect, signal } from 'alien-signals';
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
  public state: { (): P; (value: P): void };

  private meta: MetaClientState;
  // TODO cleanup interval when destroyed or something like that
  private checkInterval: ReturnType<typeof setInterval>;

  constructor(initialPresence?: P) {
    this.state = signal<P>(initialPresence ?? ({} as P));
    this.meta = {
      clock: 0,
      lastUpdated: new Date().getTime()
    };

    effect(() => {
      this.state();

      this.meta.clock++;
      this.meta.lastUpdated = new Date().getTime();
    });

    this.checkInterval = setInterval(() => {
      const now = new Date().getTime();

      if (this.state && outdatedTimeout / 2 >= now - this.meta.lastUpdated) {
        this.state({ ...this.state() });
      }
    }, outdatedTimeout / 10);
  }

  getPresenceMessage(): Extract<ClientMessage, { tag: 'ClientPresenceMessage' }> {
    return {
      val: {
        clock: this.meta.clock,
        presence: JSON.stringify(this.state())
      },
      tag: 'ClientPresenceMessage'
    };
  }

  destroy() {
    clearInterval(this.checkInterval);
  }
}
