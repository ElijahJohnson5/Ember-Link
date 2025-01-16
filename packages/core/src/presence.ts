/**
 * @module awareness-protocol
 */

import { ClientMessage, PresenceState } from '@ember-link/protocol';
import $, { ObservableReadonly, type Observable as Signal } from 'oby';

export const outdatedTimeout = 30000;

/**
 * The Awareness class implements a simple shared state protocol that can be used for non-persistent data like awareness information
 * (cursor, username, status, ..). Each client can update its own local state and listen to state changes of
 * remote clients. Every client may set a state of a remote peer to `null` to mark the client as offline.
 *
 * Each client is identified by a unique client id (something we borrow from `doc.clientID`). A client can override
 * its own state by propagating a message with an increasing timestamp (`clock`). If such a message is received, it is
 * applied if the known state of that client is older than the new state (`clock < newClock`). If a client thinks that
 * a remote client is offline, it may propagate a message with
 * `{ clock: currentClientClock, state: null, client: remoteClient }`. If such a
 * message is received, and the known clock of that client equals the received clock, it will override the state with `null`.
 *
 * Before a client disconnects, it should propagate a `null` state with an updated clock.
 *
 * Awareness states must be updated every 30 seconds. Otherwise the Awareness instance will delete the client state.
 *
 * @extends {Observable<string>}
 */

// export interface PresenceState {
//   custom?: Record<string, unknown>;
// }

type PresenceEvents2 = {
  destroy: () => void;
  update: (
    clients: { updated: number[]; added: number[]; removed: number[] },
    origin: string
  ) => void;
  change: (
    clients: { updated: number[]; added: number[]; removed: number[] },
    origin: string
  ) => void;
};

export type PresenceEvents = {
  update: (state: PresenceState) => void;
};

export interface MetaClientState {
  clock: number;
  lastUpdated: number;
}

// function createPresence(initialPresence: PresenceState) {
//   const state = signal<PresenceState>(initialPresence);
//   const emitter = createEventEmitter<PresenceEvents>();

//   effect(() => {
//     emitter.emit('update', state.get());
//   });

//   return {
//     state,
//     events: emitter.observable
//   };
// }

export class ManagedPresence {
  public state: Signal<PresenceState>;

  private meta: MetaClientState;
  private clock: ObservableReadonly<number>;
  // TODO cleanup interval when destroyed or something like that
  private checkInterval: ReturnType<typeof setInterval>;

  constructor(initialPresence?: PresenceState) {
    this.state = $(initialPresence ?? { custom: {} });
    this.meta = {
      clock: 0,
      lastUpdated: new Date().getTime()
    };

    $.effect(() => {
      const value = this.state();

      this.meta.clock++;
    });

    this.checkInterval = setInterval(() => {
      const now = new Date().getTime();

      if (this.state() && outdatedTimeout / 2 <= now - this.meta.lastUpdated) {
        this.refreshLocalTimer();
      }
    }, outdatedTimeout);
  }

  getNewPresenceMessage(): Extract<ClientMessage, { type: 'presence' }> {
    return {
      clock: this.meta.clock,
      data: this.state(),
      type: 'presence'
    };
  }

  private refreshLocalTimer() {
    const clock = this.meta.clock + 1;
    // Update state to the current version to emit an event
    this.state((oldState) => ({ ...oldState }));

    this.meta = {
      clock,
      lastUpdated: new Date().getTime()
    };
  }
}

// export class Presence2 {
//   clientId: number;
//   states: Map<number, PresenceState>;
//   meta: Map<number, MetaClientState>;
//   emitter: Emitter<PresenceEvents2>;
//   private checkInterval: ReturnType<typeof setInterval>;

//   constructor() {
//     this.clientId = uint53();
//     this.emitter = createEventEmitter();

//     this.states = new Map();
//     this.meta = new Map();

//     this.checkInterval = setInterval(
//       () => {
//         const now = time.getUnixTime();
//         if (
//           this.getLocalState() !== null &&
//           outdatedTimeout / 2 <= now - this.meta.get(this.clientId).lastUpdated
//         ) {
//           // renew local clock
//           this.setLocalState(this.getLocalState());
//         }
//         const remove: number[] = [];
//         this.meta.forEach((meta, clientid) => {
//           if (
//             clientid !== this.clientId &&
//             outdatedTimeout <= now - meta.lastUpdated &&
//             this.states.has(clientid)
//           ) {
//             remove.push(clientid);
//           }
//         });
//         if (remove.length > 0) {
//           removeAwarenessStates(this, remove, 'timeout');
//         }
//       },
//       math.floor(outdatedTimeout / 10)
//     );

//     this.setLocalState({});
//   }

//   observable(): Observable<PresenceEvents2> {
//     return {
//       subscribe: this.emitter.subscribe,
//       subscribeOnce: this.emitter.subscribeOnce,
//       waitUntil: this.emitter.waitUntil
//     };
//   }

//   destroy() {
//     this.emitter.emit('destroy');
//     this.setLocalState(null);
//     clearInterval(this.checkInterval);
//   }

//   getLocalState() {
//     return this.states.get(this.clientId) || null;
//   }

//   setLocalState(state: PresenceState | null) {
//     const clientId = this.clientId;
//     const currLocalMeta = this.meta.get(clientId);
//     const clock = currLocalMeta === undefined ? 0 : currLocalMeta.clock + 1;
//     const prevState = this.states.get(clientId);
//     if (state === null) {
//       this.states.delete(clientId);
//     } else {
//       this.states.set(clientId, state);
//     }
//     this.meta.set(clientId, {
//       clock,
//       lastUpdated: time.getUnixTime()
//     });
//     const added = [];
//     const updated = [];
//     const filteredUpdated = [];
//     const removed = [];
//     if (state === null) {
//       removed.push(clientId);
//     } else if (prevState == null) {
//       if (state != null) {
//         added.push(clientId);
//       }
//     } else {
//       updated.push(clientId);
//       if (!f.equalityDeep(prevState, state)) {
//         filteredUpdated.push(clientId);
//       }
//     }
//     if (added.length > 0 || filteredUpdated.length > 0 || removed.length > 0) {
//       this.emitter.emit('change', { added, updated: filteredUpdated, removed }, 'local');
//     }
//     this.emitter.emit('update', { added, updated, removed }, 'local');
//   }

//   /**
//    * @param {string} field
//    * @param {any} value
//    */
//   setLocalStateField(field, value) {
//     const state = this.getLocalState();
//     if (state !== null) {
//       this.setLocalState({
//         ...state,
//         [field]: value
//       });
//     }
//   }

//   /**
//    * @return {Map<number,Object<string,any>>}
//    */
//   getStates() {
//     return this.states;
//   }
// }

// /**
//  * Mark (remote) clients as inactive and remove them from the list of active peers.
//  * This change will be propagated to remote clients.
//  *
//  * @param {Awareness} awareness
//  * @param {Array<number>} clients
//  * @param {any} origin
//  */
// export const removeAwarenessStates = (awareness, clients, origin) => {
//   const removed = [];
//   for (let i = 0; i < clients.length; i++) {
//     const clientID = clients[i];
//     if (awareness.states.has(clientID)) {
//       awareness.states.delete(clientID);
//       if (clientID === awareness.clientID) {
//         const curMeta = /** @type {MetaClientState} */ awareness.meta.get(clientID);
//         awareness.meta.set(clientID, {
//           clock: curMeta.clock + 1,
//           lastUpdated: time.getUnixTime()
//         });
//       }
//       removed.push(clientID);
//     }
//   }
//   if (removed.length > 0) {
//     awareness.emit('change', [{ added: [], updated: [], removed }, origin]);
//     awareness.emit('update', [{ added: [], updated: [], removed }, origin]);
//   }
// };

// /**
//  * @param {Awareness} awareness
//  * @param {Array<number>} clients
//  * @return {Uint8Array}
//  */
// export const encodeAwarenessUpdate = (
//   presence: Presence2,
//   clients: number[],
//   states = presence.states
// ) => {
//   const len = clients.length;
//   const encoder = encoding.createEncoder();
//   encoding.writeVarUint(encoder, len);
//   for (let i = 0; i < len; i++) {
//     const clientID = clients[i];
//     const state = states.get(clientID) || null;
//     const clock = presence.meta.get(clientID).clock;
//     encoding.writeVarUint(encoder, clientID);
//     encoding.writeVarUint(encoder, clock);
//     encoding.writeVarString(encoder, JSON.stringify(state));
//   }
//   return encoding.toUint8Array(encoder);
// };

// export const modifyAwarenessUpdate = (
//   update: Uint8Array,
//   modify: (state: PresenceState | null) => PresenceState | null
// ) => {
//   const decoder = decoding.createDecoder(update);
//   const encoder = encoding.createEncoder();
//   const len = decoding.readVarUint(decoder);
//   encoding.writeVarUint(encoder, len);
//   for (let i = 0; i < len; i++) {
//     const clientID = decoding.readVarUint(decoder);
//     const clock = decoding.readVarUint(decoder);
//     const state: PresenceState | null = JSON.parse(decoding.readVarString(decoder));
//     const modifiedState = modify(state);
//     encoding.writeVarUint(encoder, clientID);
//     encoding.writeVarUint(encoder, clock);
//     encoding.writeVarString(encoder, JSON.stringify(modifiedState));
//   }
//   return encoding.toUint8Array(encoder);
// };

// export const applyPresenceUpdate = (presence: Presence2, update: Uint8Array, origin: string) => {
//   const decoder = decoding.createDecoder(update);
//   const timestamp = time.getUnixTime();
//   const added = [];
//   const updated = [];
//   const filteredUpdated = [];
//   const removed = [];
//   const len = decoding.readVarUint(decoder);
//   for (let i = 0; i < len; i++) {
//     const clientId = decoding.readVarUint(decoder);
//     let clock = decoding.readVarUint(decoder);
//     const state: PresenceState | null = JSON.parse(decoding.readVarString(decoder));
//     const clientMeta = presence.meta.get(clientId);
//     const prevState = presence.states.get(clientId);
//     const currClock = clientMeta === undefined ? 0 : clientMeta.clock;
//     if (
//       currClock < clock ||
//       (currClock === clock && state === null && presence.states.has(clientId))
//     ) {
//       if (state === null) {
//         // never let a remote client remove this local state
//         if (clientId === presence.clientId && presence.getLocalState() != null) {
//           // remote client removed the local state. Do not remote state. Broadcast a message indicating
//           // that this client still exists by increasing the clock
//           clock++;
//         } else {
//           presence.states.delete(clientId);
//         }
//       } else {
//         presence.states.set(clientId, state);
//       }
//       presence.meta.set(clientId, {
//         clock,
//         lastUpdated: timestamp
//       });
//       if (clientMeta === undefined && state !== null) {
//         added.push(clientId);
//       } else if (clientMeta !== undefined && state === null) {
//         removed.push(clientId);
//       } else if (state !== null) {
//         if (!f.equalityDeep(state, prevState)) {
//           filteredUpdated.push(clientId);
//         }
//         updated.push(clientId);
//       }
//     }
//   }
//   if (added.length > 0 || filteredUpdated.length > 0 || removed.length > 0) {
//     presence.emitter.emit(
//       'change',
//       {
//         added,
//         updated: filteredUpdated,
//         removed
//       },
//       origin
//     );
//   }
//   if (added.length > 0 || updated.length > 0 || removed.length > 0) {
//     presence.emitter.emit(
//       'update',
//       {
//         added,
//         updated,
//         removed
//       },
//       origin
//     );
//   }
// };
