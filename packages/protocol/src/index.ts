export * from './generated';

export enum WebSocketCloseCode {
  TokenNotFound = 4000,
  InvalidToken = 4001,
  InvalidSignerKey = 4002,
  ChannelCreationFailed = 4003,
  MissingTenantId = 4004
}
