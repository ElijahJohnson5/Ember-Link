import {
  Actor,
  assign,
  createActor,
  emit,
  EventObject,
  fromCallback,
  setup,
  spawnChild,
  StateMachine,
  stopChild,
} from "xstate";
import {
  cbor,
  Message,
  NetworkAdapter,
  PeerId,
  PeerMetadata,
} from "@automerge/automerge-repo/slim";
import { Repo } from "@automerge/automerge-repo";
import {
  ErrorMessage,
  FromServerMessage,
  PeerMessage,
  ProtocolV1,
} from "@automerge/automerge-repo-network-websocket";
import {
  FromClientMessage,
  JoinMessage,
} from "@automerge/automerge-repo-network-websocket";

const toArrayBuffer = (bytes: Uint8Array) => {
  const { buffer, byteOffset, byteLength } = bytes;
  return buffer.slice(byteOffset, byteOffset + byteLength);
};

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

type Events =
  | { type: "connect"; value: string }
  | { type: "open" }
  | { type: "pong" }
  | { type: "disconnect" }
  | { type: "close"; value: CloseEvent }
  | { type: "error"; value: Event }
  | { type: "webSocketCreated"; value: WebSocket }
  | { type: "message"; value: ArrayBuffer | SharedArrayBuffer };

const machine = setup({
  types: {
    context: {} as Context,
    events: {} as Events,
    emitted: {} as { type: "message"; event: MessageEvent } | { type: "test" },
  },
  actors: {
    websocketCallback: fromCallback<EventObject, { websocket: WebSocket }>(
      ({ sendBack, input, emit }) => {
        input.websocket.binaryType = "arraybuffer";
        sendBack({ type: "webSocketCreated", value: input.websocket });

        const openHandler = () => {
          sendBack({ type: "open" });
          emit({ type: "test" });
        };

        const errorHandler = (e: Event) => {
          sendBack({ type: "error", value: e });
        };

        const closeHandler = (e: CloseEvent) => {
          sendBack({ type: "close", value: e });
        };

        const messageHandler = (e: MessageEvent) => {
          if (e.data === "pong") {
            sendBack({ type: "pong" });
          }

          emit({ type: "message", event: e } as EventObject);
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
        test: {
          actions: () => {
            console.log("ASDASDAS");
          },
        },
      },
    },
    Open: {
      entry: assign({
        reconnectAttempt: 0,
      }),
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
  },

  context: {
    reconnectAttempt: 0,
    randSeed: Math.random(),
  },
});

// const test = createActor(machine);

// test.subscribe((state) => {
//   console.log(state.value);
// });

// test.start();

// test.send({ type: "connect", value: "ws://localhost:9000" });

export const isJoinMessage = (
  message: FromClientMessage,
): message is JoinMessage => message.type === "join";

export const isPeerMessage = (
  message: FromServerMessage,
): message is PeerMessage => message.type === "peer";

export const isErrorMessage = (
  message: FromServerMessage,
): message is ErrorMessage => message.type === "error";

export class SocketAdapter extends NetworkAdapter {
  private url: string;
  private stateMachine: Actor<typeof machine>;
  private readyPromise: Promise<void>;

  constructor(url: string) {
    super();

    this.url = url;
    this.stateMachine = createActor(machine);

    this.stateMachine.subscribe((state) => {
      console.log(state.value);
    });

    this.readyPromise = new Promise<void>((resolve) => {
      this.stateMachine.on("test", () => {
        console.log("TTTTTTTTTTTTTTTT");
        resolve();
      });

      setTimeout(() => {
        resolve();
      }, 1000);
    });

    this.stateMachine.start();
  }

  connect(peerId: PeerId, peerMetadata?: PeerMetadata): void {
    this.peerId = peerId;
    this.peerMetadata = peerMetadata;

    this.stateMachine.on("message", ({ type, event }) =>
      this.handleMessage(event),
    );

    this.stateMachine.on("test", () => {
      console.log("OPEN");
    });

    this.stateMachine.send({ type: "connect", value: this.url });

    // console.log("Sending join message");

    // this.send({
    //   type: "join",
    //   senderId: this.peerId,
    //   peerMetadata: this.peerMetadata!,
    //   supportedProtocolVersions: ["1"],
    // });
  }

  send(message: FromClientMessage): void {
    if ("data" in message && message.data.byteLength === 0) {
      throw new Error("Tried to send a zero-length message");
    }

    const encoded = cbor.encode(message);
    this.stateMachine.send({ type: "message", value: toArrayBuffer(encoded) });
  }
  disconnect(): void {
    this.stateMachine.send({ type: "disconnect" });
  }

  isReady() {
    const snapshot = this.stateMachine.getSnapshot();

    return snapshot.value === "Open" || snapshot.value === "OpenWaitingPong";
  }

  whenReady() {
    return this.readyPromise;
  }

  handleMessage(event: MessageEvent<Uint8Array>) {
    const message: FromServerMessage = cbor.decode(new Uint8Array(event.data));

    if (event.data.byteLength === 0) {
      throw new Error("received a zero-length message");
    }

    if (isPeerMessage(message)) {
      const { peerMetadata } = message;

      this.emit("peer-candidate", {
        peerId: message.senderId,
        peerMetadata: peerMetadata,
      });
    } else if (isErrorMessage(message)) {
      console.log(message.message);
    } else {
      this.emit("message", message);
    }
  }
}

const repo = new Repo({
  network: [new SocketAdapter("ws://localhost:9000")],
});
