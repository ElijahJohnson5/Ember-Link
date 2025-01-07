export enum ServerCode {
  YDOC_UPDATE = 0,
  JOIN_CHANNEL = 1,
}

export type YDocUpdate = {
  readonly type: ServerCode.YDOC_UPDATE;
  readonly update: Array<number>;
  readonly stateVector: string | null;
};
