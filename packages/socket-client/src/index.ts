// import {
//   createMachine,
//   guard,
//   immediate,
//   interpret,
//   invoke,
//   reduce,
//   state,
//   transition,
// } from "robot3";

import {
  assign,
  createActor,
  createMachine,
  enqueueActions,
  EventObject,
  fromCallback,
  raise,
  sendTo,
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
        console.log("Creating websocket");
        sendBack({ type: "webSocketCreated", value: input.websocket });

        const openHandler = () => {
          sendBack({ type: "open" });
        };

        const errorHandler = (e) => {
          console.log(e);

          sendBack({ type: "error", value: e });
        };

        const closeHandler = (e: CloseEvent) => {
          console.log(e.code);
          console.log(e.reason);
          console.log(e.wasClean);

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
        close: { target: "Reconnecting" },
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
          guard: ({ context }) => {
            if (context.ws?.readyState === 1) {
              return false;
            }

            return true;
          },
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

// const openConnection = () => {
//   const machine = createMachine(
//     {
//       INITIAL: state(
//         transition(
//           "connect",
//           "INITIAL.URL_SET",
//           reduce((ctx: Context, ev: { url: string }) => ({
//             ...ctx,
//             url: ev.url,
//           })),
//         ),
//       ),
//       "INITIAL.URL_SET": state(
//         immediate(
//           "CREATE_WS",
//           guard((ctx: Context) => {
//             console.log(ctx);

//             return Boolean(ctx.url);
//           }),
//         ),
//         immediate("INITIAL"),
//       ),
//       CREATE_WS: state(
//         immediate(
//           "TEST",
//           reduce((ctx: Context, e) => {
//             console.log(this);

//             return {
//               ...ctx,
//               ws: createWebsocket(ctx, e),
//             };
//           }),
//         ),
//       ),
//       TEST: state(),
//       OPEN: state(
//         transition("close", "CLOSED"),
//         transition("timeout", "RECONNECTING"),
//         transition("error", "RECONNECTING"),
//       ),
//       RECONNECTING: state(
//         transition("connected", "OPEN"),
//         transition("error", "RECONNECTING"),
//         transition("close", "CLOSED"),
//       ),
//       CLOSED: state(),
//     },
//     context,
//   );

//   const testService = interpret(machine, () => {
//     console.log(testService.machine);
//   });

//   return testService;
// };

await new Promise((resolve, reject) => {
  // openConnection().send({ type: "connect", url: "ws://localhost:9000" });

  const test = createActor(machine);

  test.subscribe((state) => {
    console.log(state.value);
  });

  test.start();

  test.send({ type: "connect", value: "ws://localhost:9000" });

  setTimeout(resolve, 30000);
});
