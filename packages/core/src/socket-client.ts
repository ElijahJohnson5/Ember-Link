import {
  assign,
  createActor,
  emit,
  enqueueActions,
  EventObject,
  fromCallback,
  setup,
  spawnChild,
  stopChild,
} from "xstate";
import {
  createBufferedEventEmitter,
  createEventEmitter,
  Observable,
} from "./event-emitter.js";

const calcBackoff = (
  attempt: number,
  randSeed: number,
  maxVal = 30000,
): number => {
  if (attempt === 0) {
    return 0;
  }
  return Math.min(maxVal, attempt ** 2 * 1000) + 2000 * randSeed;
};

interface Context {
  url?: string;
  ws?: WebSocket;
  reconnectAttempt: number;
  randSeed: number;
}

type MessageEventMap = {
  message: (event: MessageEvent) => void;
};

type Events =
  | { type: "connect"; value: string }
  | { type: "open" }
  | { type: "pong" }
  | { type: "disconnect" }
  | { type: "destroy" }
  | { type: "close"; value: CloseEvent }
  | { type: "error"; value: Event }
  | { type: "webSocketCreated"; value: WebSocket }
  | { type: "message"; value: ArrayBuffer | SharedArrayBuffer };

function createWebSocketStateMachine() {
  const messageEventEmitter = createBufferedEventEmitter<MessageEventMap>();
  messageEventEmitter.pause("message");

  const machine = setup({
    types: {
      context: {} as Context,
      events: {} as Events,
      emitted: {} as
        | { type: "message"; event: MessageEvent }
        | { type: "open" },
    },
    actors: {
      websocketCallback: fromCallback<EventObject, { websocket: WebSocket }>(
        ({ sendBack, input }) => {
          input.websocket.binaryType = "arraybuffer";
          sendBack({ type: "webSocketCreated", value: input.websocket });

          const openHandler = () => {
            sendBack({ type: "open" });
          };

          const errorHandler = (e: Event) => {
            sendBack({ type: "error", value: e });
          };

          const closeHandler = (e: CloseEvent) => {
            sendBack({ type: "close", value: e });
          };

          const messageHandler = (e: MessageEvent) => {
            if (e.data === "pong") {
              return sendBack({ type: "pong" });
            }

            messageEventEmitter.emit("message", e);
          };

          input.websocket.addEventListener("open", openHandler);
          input.websocket.addEventListener("error", errorHandler);
          input.websocket.addEventListener("close", closeHandler);
          input.websocket.addEventListener("message", messageHandler);

          return () => {
            input.websocket.removeEventListener("open", openHandler);
            input.websocket.removeEventListener("error", errorHandler);
            input.websocket.removeEventListener("close", closeHandler);
            input.websocket.removeEventListener("message", messageHandler);
          };
        },
      ),
    },
    guards: {
      reconnectGuard: ({ context }) => {
        if (context.ws?.readyState === 1) {
          return false;
        }

        return true;
      },
    },
    delays: {
      reconnectTimeout: ({ context }) => {
        const backoff = calcBackoff(context.reconnectAttempt, context.randSeed);
        return backoff;
      },
    },
  }).createMachine({
    id: "ws",
    initial: "Initial",
    // Global handlers for any state
    on: {
      destroy: {
        target: ".Destroyed",
      },
    },
    states: {
      Initial: {
        on: {
          connect: {
            target: "CreateWebSocket",
            guard: ({ event }) => Boolean(event.value),
            actions: assign({
              url: ({ event }) => event.value,
            }),
          },
        },
      },
      CreateWebSocket: {
        entry: [
          // We want the callbacks to persit through states
          stopChild("websocket-callback"),
          spawnChild("websocketCallback", {
            id: "websocket-callback",
            input: ({ context }) => ({
              websocket: new WebSocket(context.url),
            }),
          }),
        ],
        on: {
          webSocketCreated: {
            actions: assign({
              ws: ({ event }) => {
                return event.value;
              },
            }),
          },
          open: {
            target: "Open",
          },
          error: { target: "Reconnecting" },
          close: {
            target: "Reconnecting",
            guard: "reconnectGuard",
          },
        },
      },
      Open: {
        entry: [
          assign({
            reconnectAttempt: 0,
          }),
          () => {
            messageEventEmitter.resume("message");
          },
        ],
        exit: () => {
          messageEventEmitter.pause("message");
        },
        on: {
          close: { target: "Reconnecting" },
          error: {
            target: "Reconnecting",
            guard: "reconnectGuard",
          },
          message: {
            actions: ({ context, event }) => {
              context.ws?.send(event.value);
            },
          },
          disconnect: {
            target: "Initial",
            actions: [
              stopChild("websocket-callback"),
              assign({
                ws: undefined,
              }),
            ],
          },
        },
        after: {
          4000: {
            target: "OpenWaitingPong",
            actions: ({ context }) => {
              context.ws?.send("ping");
            },
          },
        },
      },
      OpenWaitingPong: {
        entry: () => {
          messageEventEmitter.resume("message");
        },
        exit: () => {
          messageEventEmitter.pause("message");
        },
        on: {
          pong: {
            target: "Open",
          },
          after: {
            2000: {
              target: "Reconnecting",
            },
          },
          message: {
            actions: ({ context, event }) => {
              context.ws?.send(event.value);
            },
          },
          disconnect: {
            target: "Initial",
            actions: [
              stopChild("websocket-callback"),
              assign({
                ws: undefined,
              }),
            ],
          },
        },
      },
      Reconnecting: {
        entry: assign({
          reconnectAttempt: ({ context }) => context.reconnectAttempt + 1,
        }),
        after: {
          reconnectTimeout: {
            target: "CreateWebSocket",
          },
        },
      },
      Destroyed: {
        entry: stopChild("websocket-callback"),
        type: "final",
      },
    },

    context: {
      reconnectAttempt: 0,
      randSeed: Math.random(),
    },
  });

  const actor = createActor(machine);

  actor.subscribe((machine) => {
    // Create event emitters for status
  });

  return {
    machine: actor,
    events: {
      ...messageEventEmitter.observable,
    },
  };
}

export class ManagedSocket {
  private url: string;
  private machine: ReturnType<typeof createWebSocketStateMachine>["machine"];

  public readonly events: Observable<MessageEventMap>;

  constructor(url: string) {
    this.url = url;
    const { machine, events } = createWebSocketStateMachine();

    this.machine = machine;
    this.machine.start();
    this.events = events;
  }

  connect() {
    this.machine.send({ type: "connect", value: this.url });
  }

  disconnect() {
    this.machine.send({ type: "disconnect" });
  }

  destroy() {
    this.machine.send({ type: "destroy" });
    this.machine.stop();
  }
}

// const test = createActor(machine);

// test.subscribe((state) => {
//   console.log(state.value);
// });

// test.start();

// test.send({ type: "connect", value: "ws://localhost:9000" });

// export const isJoinMessage = (
//   message: FromClientMessage,
// ): message is JoinMessage => message.type === "join";

// export const isPeerMessage = (
//   message: FromServerMessage,
// ): message is PeerMessage => message.type === "peer";

// export const isErrorMessage = (
//   message: FromServerMessage,
// ): message is ErrorMessage => message.type === "error";

// export class SocketAdapter extends NetworkAdapter {
//   private url: string;
//   private stateMachine: Actor<typeof machine>;
//   private readyPromise: Promise<void>;

//   constructor(url: string) {
//     super();

//     this.url = url;
//     this.stateMachine = createActor(machine);

//     this.stateMachine.subscribe((state) => {
//       console.log(state.value);
//     });

//     this.readyPromise = new Promise<void>((resolve) => {
//       this.stateMachine.on("open", () => {
//         resolve();
//       });

//       setTimeout(() => {
//         resolve();
//       }, 1000);
//     });

//     this.stateMachine.start();
//   }

//   connect(peerId: PeerId, peerMetadata?: PeerMetadata): void {
//     this.peerId = peerId;
//     this.peerMetadata = peerMetadata;

//     this.stateMachine.on("message", ({ type, event }) =>
//       this.handleMessage(event),
//     );

//     this.stateMachine.on("open", () => {
//       this.send({
//         type: "join",
//         senderId: this.peerId,
//         peerMetadata: this.peerMetadata!,
//         supportedProtocolVersions: ["1"],
//       } as JoinMessage);
//     });

//     this.stateMachine.send({ type: "connect", value: this.url });
//   }

//   send(message: FromClientMessage): void {
//     if ("data" in message && message.data.byteLength === 0) {
//       throw new Error("Tried to send a zero-length message");
//     }

//     const encoded = cbor.encode(message);
//     this.stateMachine.send({ type: "message", value: toArrayBuffer(encoded) });
//   }
//   disconnect(): void {
//     this.stateMachine.send({ type: "disconnect" });
//   }

//   isReady() {
//     const snapshot = this.stateMachine.getSnapshot();

//     return snapshot.value === "Open" || snapshot.value === "OpenWaitingPong";
//   }

//   whenReady() {
//     return this.readyPromise;
//   }

//   handleMessage(event: MessageEvent<Uint8Array>) {
//     const message: FromServerMessage = cbor.decode(new Uint8Array(event.data));

//     if (event.data.byteLength === 0) {
//       throw new Error("received a zero-length message");
//     }

//     if (isPeerMessage(message)) {
//       const { peerMetadata } = message;

//       this.emit("peer-candidate", {
//         peerId: message.senderId,
//         peerMetadata: peerMetadata,
//       });
//     } else if (isErrorMessage(message)) {
//       console.log(message.message);
//     } else {
//       this.emit("message", message);
//     }
//   }
// }

// const repo = new Repo({
//   network: [new SocketAdapter("ws://localhost:9000")],
// });

// const toArrayBuffer = (bytes: Uint8Array) => {
//   const { buffer, byteOffset, byteLength } = bytes;
//   return buffer.slice(byteOffset, byteOffset + byteLength);
// };
