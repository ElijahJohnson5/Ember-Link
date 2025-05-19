import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';
import { AuthFailedError, AuthValue } from '~/auth';
import { ManagedSocket, SocketOptions } from '~/socket-client';
import { WebSocketNotFoundError } from '~/types';
import WS from 'vitest-websocket-mock';
import { WebSocketCloseCode } from '@ember-link/protocol';

describe('ManagedSocket', () => {
  describe('machine', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.resetAllMocks();
    });

    const webSocketTest = test.extend<{ serverData: { server: WS; url: string } }>({
      // eslint-disable-next-line no-empty-pattern
      serverData: async ({}, use) => {
        let server: WS | null = null;
        let port = 1234;
        const attempt = 0;
        while (!server) {
          if (attempt < 10) {
            try {
              server = new WS(`ws://localhost:${port}`);
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
            } catch (e) {
              port++;
            }
          } else {
            server = new WS(`ws://localhost:${port}`);
          }
        }

        await use({ server, url: `ws://localhost:${port}` });

        server.close();
      }
    });

    test('starts in Initial state', () => {
      const managedSocket = new ManagedSocket({
        authenticate: vi.fn(),
        createWebSocket: vi.fn()
      });

      expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

      managedSocket.destroy();
    });

    describe('Auth', () => {
      test('on connect goes to Auth', () => {
        const managedSocket = new ManagedSocket({
          authenticate: vi.fn(),
          createWebSocket: vi.fn()
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

        managedSocket.connect();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Auth');

        managedSocket.destroy();
      });

      test('after 10 seconds in auth timesout and goes to AuthBackoff', () => {
        const managedSocket = new ManagedSocket({
          authenticate: vi.fn(),
          createWebSocket: vi.fn()
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

        managedSocket.connect();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Auth');

        vi.advanceTimersByTime(10_000);

        expect(managedSocket.machine.getSnapshot().value).toEqual('AuthBackoff');

        managedSocket.destroy();
      });

      test('goes to AuthBackoff when authenticate fails', async () => {
        const authenticate = vi.fn().mockRejectedValue(new Error('test'));

        const managedSocket = new ManagedSocket({
          authenticate,
          createWebSocket: vi.fn()
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

        managedSocket.connect();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Auth');

        await vi.waitUntil(() => expect(authenticate).toHaveBeenCalled());

        // Make sure the state machine transitions can occur
        await vi.runOnlyPendingTimersAsync();

        expect(managedSocket.machine.getSnapshot().value).toEqual('AuthBackoff');

        managedSocket.destroy();
      });

      test('goes to Failed when authenticate fails with AuthFailedError', async () => {
        const authenticate = vi.fn().mockRejectedValue(new AuthFailedError('test'));

        const managedSocket = new ManagedSocket({
          authenticate,
          createWebSocket: vi.fn()
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

        managedSocket.connect();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Auth');

        await vi.waitUntil(() => expect(authenticate).toHaveBeenCalled());

        // Make sure the state machine transitions can occur
        await vi.runOnlyPendingTimersAsync();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Failed');

        managedSocket.destroy();
      });
    });

    describe('CreateWebSocket', () => {
      function setupManagedSocket(
        createWebSocket: SocketOptions['createWebSocket']
      ): ManagedSocket {
        const authenticate = vi.fn<() => Promise<AuthValue>>().mockResolvedValue({
          token: {
            parsed: {
              payload: {
                exp: Date.now(),
                iat: Date.now(),
                uid: 'test'
              },
              protectedHeader: {
                alg: 'RS256'
              }
            },
            raw: 'testtoken',
            type: 'AuthToken'
          },
          type: 'private'
        });

        return new ManagedSocket({
          authenticate,
          createWebSocket
        });
      }

      webSocketTest(
        'goes to CreateWebSocket when authenticate succeeds',
        async ({ serverData: { url } }) => {
          const createWebSocket = vi
            .fn<(authValue: AuthValue) => WebSocket>()
            .mockReturnValue(new WebSocket(url));

          const managedSocket = setupManagedSocket(createWebSocket);

          managedSocket.connect();

          await vi.waitFor(() => {
            expect(createWebSocket).toHaveBeenCalled();
          });

          expect(managedSocket.machine.getSnapshot().value).toEqual('CreateWebSocket');
          expect(createWebSocket).toHaveBeenCalled();

          managedSocket.destroy();
        }
      );

      webSocketTest(
        'goes to Reconnecting when server closes connection right away',
        async ({ serverData: { server, url } }) => {
          server.on('connection', (client) => {
            client.close();
          });

          const createWebSocket = vi
            .fn<(authValue: AuthValue) => WebSocket>()
            .mockImplementation(() => new WebSocket(url));

          const managedSocket = setupManagedSocket(createWebSocket);

          managedSocket.connect();

          await vi.waitFor(() => {
            expect(createWebSocket).toHaveBeenCalled();
          });

          expect(managedSocket.machine.getSnapshot().value).toEqual('Reconnecting');

          managedSocket.destroy();
        }
      );

      webSocketTest('goes to Open once websocket is open', async ({ serverData: { url } }) => {
        const createWebSocket = vi
          .fn<(authValue: AuthValue) => WebSocket>()
          .mockImplementation(() => new WebSocket(url));

        const managedSocket = setupManagedSocket(createWebSocket);

        managedSocket.connect();

        await vi.waitFor(() => {
          expect(createWebSocket).toHaveBeenCalled();
        });

        await vi.waitFor(() => {
          expect(managedSocket.machine.getSnapshot().value).toEqual('Open');
        });

        managedSocket.destroy();
      });

      test('goes to Failed if the createWebSocket function throws WebSocketNotFoundError', async () => {
        const createWebSocket = vi.fn<() => WebSocket>().mockImplementation(() => {
          throw new WebSocketNotFoundError('Test websocket not found error');
        });

        const managedSocket = setupManagedSocket(createWebSocket);

        managedSocket.connect();

        // Make sure the state machine transitions can occur
        await vi.runOnlyPendingTimersAsync();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Failed');
        expect(createWebSocket).toHaveBeenCalled();

        managedSocket.destroy();
      });

      test('goes to Reconnecting if the createWebSocket function errors', async () => {
        const createWebSocket = vi.fn<() => WebSocket>().mockImplementation(() => {
          throw new Error('Test error');
        });

        const managedSocket = setupManagedSocket(createWebSocket);

        managedSocket.connect();

        await vi.waitFor(() => {
          expect(createWebSocket).toHaveBeenCalled();
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Reconnecting');
        expect(createWebSocket).toHaveBeenCalled();

        managedSocket.destroy();
      });
    });

    describe('Open', () => {
      const setupManagedSocket = async (url: string) => {
        const authenticate = vi.fn<() => Promise<AuthValue>>().mockResolvedValue({
          token: {
            parsed: {
              payload: {
                exp: Date.now(),
                iat: Date.now(),
                uid: 'test'
              },
              protectedHeader: {
                alg: 'RS256'
              }
            },
            raw: 'testtoken',
            type: 'AuthToken'
          },
          type: 'private'
        });

        const createWebSocket = vi
          .fn<(authValue: AuthValue) => WebSocket>()
          .mockImplementation(() => new WebSocket(url));

        const managedSocket = new ManagedSocket({
          authenticate,
          createWebSocket
        });

        managedSocket.connect();

        await vi.waitFor(() => {
          expect(managedSocket.machine.getSnapshot().value).toEqual('Open');
        });

        return managedSocket;
      };

      webSocketTest('goes to OpenWaitingPong after timeout', async ({ serverData: { url } }) => {
        const managedSocket = await setupManagedSocket(url);

        await vi.advanceTimersByTime(4000);

        expect(managedSocket.machine.getSnapshot().value).toEqual('OpenWaitingPong');

        managedSocket.destroy();
      });

      webSocketTest('goes to Initial when disconnect is sent', async ({ serverData: { url } }) => {
        const managedSocket = await setupManagedSocket(url);

        managedSocket.disconnect();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Initial');

        managedSocket.destroy();
      });

      webSocketTest('goes to Reconnecting on error', async ({ serverData: { server, url } }) => {
        const managedSocket = await setupManagedSocket(url);

        server.error();

        expect(managedSocket.machine.getSnapshot().value).toEqual('Reconnecting');

        managedSocket.destroy();
      });

      webSocketTest.concurrent.for([1000, 1001, 1011, 1012, 1013, WebSocketCloseCode.InvalidToken])(
        'goes to Reconnecting on certain close codes',
        async (code, { serverData: { server, url } }) => {
          const managedSocket = await setupManagedSocket(url);

          server.close({
            code,
            reason: 'Test reason',
            wasClean: true
          });

          expect(managedSocket.machine.getSnapshot().value).toEqual('Reconnecting');

          managedSocket.destroy();
        }
      );

      webSocketTest.concurrent.for([
        1015,
        WebSocketCloseCode.InvalidSignerKey,
        WebSocketCloseCode.TokenNotFound
      ])('goes to Failed on certain close codes', async (code, { serverData: { server, url } }) => {
        const managedSocket = await setupManagedSocket(url);

        server.close({
          code,
          reason: 'Test reason',
          wasClean: true
        });

        expect(managedSocket.machine.getSnapshot().value).toEqual('Failed');

        managedSocket.destroy();
      });

      webSocketTest(
        'sends message with message event to websocket server',
        async ({ serverData: { server, url } }) => {
          const managedSocket = await setupManagedSocket(url);

          managedSocket.message({
            type: 'presence',
            data: {},
            clock: 0
          });

          await vi.runOnlyPendingTimersAsync();

          await expect(server).toReceiveMessage(
            JSON.stringify({
              type: 'presence',
              data: {},
              clock: 0
            })
          );

          managedSocket.destroy();
        }
      );
    });
    describe('OpenWaitingPong', () => {
      const setupManagedSocket = async (url: string) => {
        const authenticate = vi.fn<() => Promise<AuthValue>>().mockResolvedValue({
          token: {
            parsed: {
              payload: {
                exp: Date.now(),
                iat: Date.now(),
                uid: 'test'
              },
              protectedHeader: {
                alg: 'RS256'
              }
            },
            raw: 'testtoken',
            type: 'AuthToken'
          },
          type: 'private'
        });

        const createWebSocket = vi
          .fn<(authValue: AuthValue) => WebSocket>()
          .mockImplementation(() => new WebSocket(url));

        const managedSocket = new ManagedSocket({
          authenticate,
          createWebSocket
        });

        managedSocket.connect();

        await vi.waitFor(() => {
          expect(managedSocket.machine.getSnapshot().value).toEqual('Open');
        });

        await vi.advanceTimersByTime(4000);

        await vi.waitFor(() => {
          expect(managedSocket.machine.getSnapshot().value).toEqual('OpenWaitingPong');
        });

        return managedSocket;
      };

      webSocketTest(
        'goes to Open after recieving pong form server',
        async ({ serverData: { server, url } }) => {
          const managedSocket = await setupManagedSocket(url);

          server.send('pong');

          await vi.waitFor(() => {
            expect(managedSocket.machine.getSnapshot().value).toEqual('Open');
          });

          managedSocket.destroy();
        }
      );
    });
  });
});
