import {
  assign,
  createActor,
  EventObject,
  fromCallback,
  setup,
  spawnChild,
  StateFrom,
  stopChild
} from 'xstate';
import { createBufferedEventEmitter, Observable } from './event-emitter.js';
import { ClientMessage } from '@ember-link/protocol';

const calcBackoff = (attempt: number, randSeed: number, maxVal = 30000): number => {
  if (attempt === 0) {
    return 0;
  }
  return Math.min(maxVal, attempt ** 2 * 1000) + 2000 * randSeed;
};

export type Status = 'initial' | 'connected' | 'reconnecting' | 'connecting' | 'closed';

interface Context {
  url?: string;
  ws?: WebSocket;
  reconnectAttempt: number;
  successCount: number;
  randSeed: number;
}

type MessageEventMap = {
  message: (event: MessageEvent<string | Blob>) => void;
  open: () => void;
  disconnect: () => void;
  statusChange: (status: Status) => void;
};

type Events =
  | { type: 'connect'; value: string }
  | { type: 'open' }
  | { type: 'pong' }
  | { type: 'disconnect' }
  | { type: 'destroy' }
  | { type: 'close'; value: CloseEvent }
  | { type: 'error'; value: Event }
  | { type: 'webSocketCreated'; value: WebSocket }
  | { type: 'message'; value: string | ArrayBufferLike | Blob | ArrayBufferView };

function createWebSocketStateMachine() {
  const eventEmitter = createBufferedEventEmitter<MessageEventMap>();
  eventEmitter.pause('message');

  const machine = setup({
    types: {
      context: {} as Context,
      events: {} as Events
    },
    actors: {
      websocketCallback: fromCallback<EventObject, { websocket: WebSocket }>(
        ({ sendBack, input }) => {
          input.websocket.binaryType = 'arraybuffer';
          sendBack({ type: 'webSocketCreated', value: input.websocket });

          const openHandler = () => {
            sendBack({ type: 'open' });
          };

          const errorHandler = (e: Event) => {
            sendBack({ type: 'error', value: e });
          };

          const closeHandler = (e: CloseEvent) => {
            sendBack({ type: 'close', value: e });
          };

          const messageHandler = (e: MessageEvent) => {
            if (e.data === 'pong') {
              return sendBack({ type: 'pong' });
            }

            eventEmitter.emit('message', e);
          };

          input.websocket.addEventListener('open', openHandler);
          input.websocket.addEventListener('error', errorHandler);
          input.websocket.addEventListener('close', closeHandler);
          input.websocket.addEventListener('message', messageHandler);

          return () => {
            input.websocket.removeEventListener('open', openHandler);
            input.websocket.removeEventListener('error', errorHandler);
            input.websocket.removeEventListener('close', closeHandler);
            input.websocket.removeEventListener('message', messageHandler);
          };
        }
      )
    },
    guards: {
      reconnectGuard: ({ context }) => {
        if (context.ws?.readyState === 1) {
          return false;
        }

        return true;
      }
    },
    delays: {
      reconnectTimeout: ({ context }) => {
        const backoff = calcBackoff(context.reconnectAttempt, context.randSeed);
        return backoff;
      }
    }
  }).createMachine({
    id: 'ws',
    initial: 'Initial',
    // Global handlers for any state
    on: {
      destroy: {
        target: '.Destroyed'
      }
    },
    states: {
      Initial: {
        on: {
          connect: {
            target: 'CreateWebSocket',
            guard: ({ event }) => Boolean(event.value),
            actions: assign({
              url: ({ event }) => event.value
            })
          }
        }
      },
      CreateWebSocket: {
        entry: [
          // We want the callbacks to persit through states
          stopChild('websocket-callback'),
          spawnChild('websocketCallback', {
            id: 'websocket-callback',
            input: ({ context }) => ({
              websocket: new WebSocket(context.url)
            })
          })
        ],
        on: {
          webSocketCreated: {
            actions: assign({
              ws: ({ event }) => {
                return event.value;
              }
            })
          },
          open: {
            target: 'Open'
          },
          error: { target: 'Reconnecting' },
          close: {
            target: 'Reconnecting',
            guard: 'reconnectGuard'
          }
        }
      },
      Open: {
        entry: [
          assign({
            reconnectAttempt: 0,
            successCount: ({ context }) => context.successCount + 1
          }),
          () => {
            eventEmitter.resume('message');
          }
        ],
        exit: () => {
          eventEmitter.pause('message');
        },
        on: {
          close: { target: 'Reconnecting' },
          error: {
            target: 'Reconnecting',
            guard: 'reconnectGuard'
          },
          message: {
            actions: ({ context, event }) => {
              context.ws?.send(event.value);
            }
          },
          disconnect: {
            target: 'Initial',
            actions: [
              stopChild('websocket-callback'),
              assign({
                ws: undefined
              })
            ]
          }
        },
        after: {
          4000: {
            target: 'OpenWaitingPong',
            actions: ({ context }) => {
              context.ws?.send('ping');
            }
          }
        }
      },
      OpenWaitingPong: {
        entry: () => {
          eventEmitter.resume('message');
        },
        exit: () => {
          eventEmitter.pause('message');
        },
        on: {
          pong: {
            target: 'Open'
          },
          after: {
            2000: {
              target: 'Reconnecting'
            }
          },
          message: {
            actions: ({ context, event }) => {
              context.ws?.send(event.value);
            }
          },
          disconnect: {
            target: 'Initial',
            actions: [
              stopChild('websocket-callback'),
              assign({
                ws: undefined
              })
            ]
          }
        }
      },
      Reconnecting: {
        entry: assign({
          reconnectAttempt: ({ context }) => context.reconnectAttempt + 1
        }),
        after: {
          reconnectTimeout: {
            target: 'CreateWebSocket'
          }
        }
      },
      Destroyed: {
        entry: stopChild('websocket-callback'),
        type: 'final'
      }
    },

    context: {
      reconnectAttempt: 0,
      successCount: 0,
      randSeed: Math.random()
    }
  });

  function valueToStatus(value: StateFrom<typeof machine>['value'], successCount: number): Status {
    switch (value) {
      case 'Open':
      case 'OpenWaitingPong':
        return 'connected';
      case 'Initial':
        return 'initial';
      case 'CreateWebSocket':
      case 'Reconnecting':
        return successCount > 0 ? 'reconnecting' : 'connecting';
      case 'Destroyed':
        return 'closed';
    }
  }

  const actor = createActor(machine);

  let lastStatus: Status | null = null;

  actor.subscribe((snapshot) => {
    const currentStatus = valueToStatus(snapshot.value, snapshot.context.successCount);

    if (lastStatus !== currentStatus) {
      // Emit a change status event;
      eventEmitter.emit('statusChange', currentStatus);
    }

    // Create event emitters for status
    if (lastStatus === 'connected' && currentStatus !== 'connected') {
      eventEmitter.emit('disconnect');
    } else if (lastStatus !== 'connected' && currentStatus === 'connected') {
      eventEmitter.emit('open');
    }

    lastStatus = currentStatus;
  });

  return {
    machine: actor,
    events: eventEmitter.observable
  };
}

export class ManagedSocket {
  private url: string;
  private machine: ReturnType<typeof createWebSocketStateMachine>['machine'];

  public readonly events: Observable<MessageEventMap>;

  constructor(url: string) {
    this.url = url;
    const { machine, events } = createWebSocketStateMachine();

    this.machine = machine;
    this.machine.start();
    this.events = events;
  }

  connect() {
    this.machine.send({ type: 'connect', value: this.url });
  }

  message(data: ClientMessage) {
    this.machine.send({ type: 'message', value: JSON.stringify(data) });
  }

  disconnect() {
    this.machine.send({ type: 'disconnect' });
  }

  destroy() {
    this.machine.send({ type: 'destroy' });
    this.machine.stop();
  }
}
