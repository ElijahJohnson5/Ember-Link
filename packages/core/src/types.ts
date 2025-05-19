export enum WebsocketCloseCodes {
  CLOSE_NORMAL = 1000,
  CLOSE_ABNORMAL = 1006,
  UNEXPECTED_CONDITION = 1011,
  TRY_AGAIN_LATER = 1013
}

export interface IWebSocketInstance {
  readonly CONNECTING: number; // 0
  readonly OPEN: number; // 1
  readonly CLOSING: number; // 2
  readonly CLOSED: number; // 3

  readonly readyState: number;

  addEventListener(type: "close", listener: (this: IWebSocketInstance, ev: CloseEvent) => unknown): void; // prettier-ignore
  addEventListener(type: "message", listener: (this: IWebSocketInstance, ev: MessageEvent) => unknown): void; // prettier-ignore
  addEventListener(type: "open" | "error", listener: (this: IWebSocketInstance, ev: Event) => unknown): void; // prettier-ignore

  removeEventListener(type: "close", listener: (this: IWebSocketInstance, ev: CloseEvent) => unknown): void; // prettier-ignore
  removeEventListener(type: "message", listener: (this: IWebSocketInstance, ev: MessageEvent) => unknown): void; // prettier-ignore
  removeEventListener(type: "open" | "error", listener: (this: IWebSocketInstance, ev: Event) => unknown): void; // prettier-ignore

  close(): void;
  send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void;
}

export interface IWebSocket {
  new (address: string): IWebSocketInstance;
}

export class WebSocketNotFoundError extends Error {
  constructor(reason: string) {
    super(reason);
  }
}
