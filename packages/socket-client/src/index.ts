import {
  assign,
  createActor,
  EventObject,
  fromCallback,
  setup,
  spawnChild,
  stopChild,
} from "xstate";

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

const machine = setup({
  types: {
    context: {} as {
      url?: string;
      ws?: WebSocket;
      reconnectAttempt: number;
      randSeed: number;
    },
    events: {} as
      | { type: "connect"; value: string }
      | { type: "open" }
      | { type: "close"; value: CloseEvent }
      | { type: "error"; value: Event }
      | { type: "webSocketCreated"; value: WebSocket },
  },
  actors: {
    websocketCallback: fromCallback<EventObject, { websocket: WebSocket }>(
      ({ sendBack, input }) => {
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

        input.websocket.addEventListener("open", openHandler);
        input.websocket.addEventListener("error", errorHandler);
        input.websocket.addEventListener("close", closeHandler);

        return () => {
          input.websocket.removeEventListener("open", openHandler);
          input.websocket.removeEventListener("error", errorHandler);
          input.websocket.removeEventListener("close", closeHandler);
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
    }
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
          guard: "reconnectGuard"
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
          guard: "reconnectGuard"
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

const test = createActor(machine);

test.subscribe((state) => {
  console.log(state.value);
});

test.start();

test.send({ type: "connect", value: "ws://localhost:9000" });

