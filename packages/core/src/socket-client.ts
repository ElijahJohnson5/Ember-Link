import {
  and,
  assign,
  createActor,
  EventObject,
  fromCallback,
  fromPromise,
  setup,
  spawnChild,
  StateFrom,
  stopChild
} from 'xstate';
import { createBufferedEventEmitter, Observable } from '@ember-link/event-emitter';
import { ClientMessage } from '@ember-link/protocol';
import { DefaultPresence } from '.';
import { AuthFailedError, AuthValue } from './auth';
import { WebSocketNotFoundError } from './client';

const calcBackoff = (attempt: number, randSeed: number, maxVal = 30000): number => {
  if (attempt === 0) {
    return 0;
  }
  return Math.min(maxVal, attempt ** 2 * 1000) + 2000 * randSeed;
};

export type Status =
  | 'initial'
  | 'connected'
  | 'reconnecting'
  | 'connecting'
  | 'closed'
  | 'disconnected';

interface Context {
  ws?: WebSocket;
  reconnectAttempt: number;
  authAttempt: number;
  successCount: number;
  randSeed: number;
  authValue: AuthValue | null;
}

type MessageEventMap = {
  message: (event: MessageEvent<string | Blob>) => void;
  open: () => void;
  disconnect: () => void;
  statusChange: (status: Status) => void;
};

interface WebSocketCallbackInput {
  createWebSocket: SocketOptions['createWebSocket'];
  authValue: AuthValue;
}

type Events =
  | { type: 'connect' }
  | { type: 'open' }
  | { type: 'pong' }
  | { type: 'disconnect' }
  | { type: 'destroy' }
  | { type: 'close'; value: CloseEvent }
  | { type: 'error'; value: Event | Error }
  | { type: 'webSocketCreated'; value: WebSocket }
  | { type: 'message'; value: string | ArrayBufferLike | Blob | ArrayBufferView };

function createWebSocketStateMachine({ authenticate, createWebSocket }: SocketOptions) {
  const eventEmitter = createBufferedEventEmitter<MessageEventMap>();
  eventEmitter.pause('message');

  const machine = setup({
    types: {
      context: {} as Context,
      events: {} as Events
    },
    actors: {
      authenticate: fromPromise(async () => {
        const data = await authenticate();

        return data;
      }),
      websocketCallback: fromCallback<EventObject, WebSocketCallbackInput>(
        ({ sendBack, input }) => {
          let websocket: WebSocket;

          try {
            websocket = input.createWebSocket(input.authValue);
          } catch (e) {
            sendBack({ type: 'error', value: e });
            return;
          }

          websocket.binaryType = 'arraybuffer';
          sendBack({ type: 'webSocketCreated', value: websocket });

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

          websocket.addEventListener('open', openHandler);
          websocket.addEventListener('error', errorHandler);
          websocket.addEventListener('close', closeHandler);
          websocket.addEventListener('message', messageHandler);

          return () => {
            websocket.removeEventListener('open', openHandler);
            websocket.removeEventListener('error', errorHandler);
            websocket.removeEventListener('close', closeHandler);
            websocket.removeEventListener('message', messageHandler);
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
      },
      authTimeout: ({ context }) => {
        const backoff = calcBackoff(context.authAttempt, context.randSeed);
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
            target: 'Auth'
          }
        }
      },
      Failed: {
        entry: [
          assign({
            reconnectAttempt: 0,
            successCount: 0
          })
        ],
        on: {
          connect: [
            {
              target: 'Auth',
              guard: and([({ context }) => Boolean(!context.authValue)])
            },
            {
              target: 'CreateWebSocket'
            }
          ]
        }
      },
      Auth: {
        invoke: {
          id: 'authenticate',
          src: 'authenticate',
          onDone: {
            target: 'CreateWebSocket',
            actions: assign({ authValue: ({ event }) => event.output })
          },
          onError: [
            // ORDER MATTERS
            {
              target: 'Failed',
              guard: ({ event }) => {
                if (event.error instanceof AuthFailedError) {
                  return true;
                }

                return false;
              }
            },
            {
              target: 'AuthBackoff'
            }
          ]
        },
        after: {
          10000: {
            target: 'AuthBackoff'
          }
        }
      },
      AuthBackoff: {
        entry: assign({
          authAttempt: ({ context }) => context.authAttempt + 1
        }),
        after: {
          authTimeout: {
            target: 'Auth'
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
              createWebSocket,
              authValue: context.authValue!
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
          error: [
            {
              target: 'Reconnecting',
              guard: ({ event }) => {
                if (event.value instanceof WebSocketNotFoundError) {
                  return false;
                }

                return true;
              }
            },
            {
              target: 'Failed'
            }
          ],
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
            successCount: ({ context, event }) => {
              if (event.type !== 'pong') {
                return context.successCount + 1;
              }

              return context.successCount;
            }
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
      authAttempt: 0,
      successCount: 0,
      randSeed: Math.random(),
      authValue: null
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
      case 'Auth':
      case 'AuthBackoff':
        return successCount > 0 ? 'reconnecting' : 'connecting';

      case 'Failed':
        return 'disconnected';

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

export interface SocketOptions {
  authenticate: () => Promise<AuthValue>;
  createWebSocket: (authValue: AuthValue) => WebSocket;
}

export class ManagedSocket<P extends Record<string, unknown> = DefaultPresence> {
  private machine: ReturnType<typeof createWebSocketStateMachine>['machine'];

  public readonly events: Observable<MessageEventMap>;

  constructor(options: SocketOptions) {
    const { machine, events } = createWebSocketStateMachine(options);

    this.machine = machine;
    this.machine.start();
    this.events = events;
  }

  connect() {
    this.machine.send({ type: 'connect' });
  }

  message(data: ClientMessage<P>) {
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
