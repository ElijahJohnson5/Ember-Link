import { afterAll, afterEach, beforeAll, describe, expect, test, vi } from 'vitest';
import { AccessToken, AuthEndpoint, AuthFailedError, AuthValue, createAuth } from '~/auth';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { type JWTVerifyResult } from 'jose';

const mocks = vi.hoisted(() => {
  return { jwtVerify: vi.fn(), importSPKI: vi.fn() };
});

vi.mock(import('jose'), async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    importSPKI: mocks.importSPKI,
    jwtVerify: mocks.jwtVerify
  };
});

describe('createAuth', () => {
  describe('No authEndpoint', () => {
    test('returns public auth type whenever getAuthValue is called', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated
      });

      const authValue = await auth.getAuthValue('test');
      expect(authValue).toEqual({ type: 'public' });
      expect(onAuthenticated).toHaveBeenCalledWith({ type: 'public' });
    });
  });

  describe('URL authEndpoint', () => {
    const handlers = [
      http.post('https://test.com/auth', () => {
        return HttpResponse.json({ token: 'test_token' });
      })
    ];

    const server = setupServer(...handlers);

    beforeAll(() => {
      server.listen({ onUnhandledRequest: 'error' });
    });

    afterEach(() => {
      server.resetHandlers(...handlers);
    });

    afterAll(() => {
      server.close();
    });

    test('throws error if jwtSignerPublicKey is not specified with auth endpoint', () => {
      const onAuthenticated = vi.fn();

      expect(() =>
        createAuth({
          onAuthenticated,
          authEndpoint: 'https://test.com/auth'
        })
      ).toThrowError('Could not create auth');
    });

    test('parsers and returns token', async () => {
      mocks.jwtVerify.mockResolvedValue({
        payload: {
          exp: Date.now() + 3000,
          iat: Date.now(),
          uid: 'test'
        },
        protectedHeader: { alg: 'RS256' }
      } satisfies JWTVerifyResult<AccessToken>);

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: 'https://test.com/auth',
        jwtSignerPublicKey: 'test'
      });

      const authValue = await auth.getAuthValue('test');

      expect(authValue.type).toEqual('private');

      const authValueTyped = authValue as Extract<AuthValue, { type: 'private' }>;

      expect(authValueTyped.token.raw).toEqual('test_token');
    });

    test('throws error for non json response', async () => {
      server.resetHandlers(
        ...[
          http.post('https://test.com/auth', () => {
            return HttpResponse.text('test_token');
          })
        ]
      );

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: 'https://test.com/auth',
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError('Expected a json');
    });

    test('throws error no token response', async () => {
      server.resetHandlers(
        ...[
          http.post('https://test.com/auth', () => {
            return HttpResponse.json({ notToken: 'test_token' });
          })
        ]
      );

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: 'https://test.com/auth',
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(
        'Expected json to be a plain object with a string token'
      );
    });

    test('throws AuthFailedError when response status is 401 or 403', async () => {
      server.resetHandlers(
        ...[
          http.post('https://test.com/auth', () => {
            return HttpResponse.text('test', {
              status: 401
            });
          })
        ]
      );

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: 'https://test.com/auth',
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(AuthFailedError);
    });

    test('throws generic error when response status is not ok', async () => {
      server.resetHandlers(
        ...[
          http.post('https://test.com/auth', () => {
            return HttpResponse.text('test', {
              status: 500
            });
          })
        ]
      );

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: 'https://test.com/auth',
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError('Failed to authenticate');
    });
  });

  describe('Callback authEndpoint', () => {
    test('throws error if jwtSignerPublicKey is not specified with auth endpoint', () => {
      const onAuthenticated = vi.fn();

      expect(() =>
        createAuth({
          onAuthenticated,
          authEndpoint: async () => {
            return { token: 'test_token' };
          }
        })
      ).toThrowError('Could not create auth');
    });

    test('parsers and returns token', async () => {
      mocks.jwtVerify.mockResolvedValue({
        payload: {
          exp: Date.now(),
          iat: Date.now(),
          uid: 'test'
        },
        protectedHeader: { alg: 'RS256' }
      } satisfies JWTVerifyResult<AccessToken>);

      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: async () => {
          return { token: 'test_token' };
        },
        jwtSignerPublicKey: 'test'
      });

      const authValue = await auth.getAuthValue('test');

      expect(authValue.type).toEqual('private');

      const authValueTyped = authValue as Extract<AuthValue, { type: 'private' }>;

      expect(authValueTyped.token.raw).toEqual('test_token');
    });

    test("returns cached token if it isn't expired", async () => {
      mocks.jwtVerify.mockResolvedValue({
        payload: {
          exp: Date.now() + 3000,
          iat: Date.now(),
          uid: 'test'
        },
        protectedHeader: { alg: 'RS256' }
      } satisfies JWTVerifyResult<AccessToken>);

      const onAuthenticated = vi.fn();
      const authEndpoint = vi.fn().mockImplementation(async () => {
        return { token: 'test_token' };
      });

      const auth = createAuth({
        onAuthenticated,
        authEndpoint,
        jwtSignerPublicKey: 'test'
      });

      await auth.getAuthValue('test');

      expect(authEndpoint).toHaveBeenCalled();

      await auth.getAuthValue('test');

      expect(authEndpoint).toHaveBeenCalledOnce();
    });

    test('does not return expired cache token', async () => {
      mocks.jwtVerify.mockResolvedValue({
        payload: {
          exp: Date.now() - 1000,
          iat: Date.now(),
          uid: 'test'
        },
        protectedHeader: { alg: 'RS256' }
      } satisfies JWTVerifyResult<AccessToken>);

      const onAuthenticated = vi.fn();
      const authEndpoint = vi.fn().mockImplementation(async () => {
        return { token: 'test_token' };
      });

      const auth = createAuth({
        onAuthenticated,
        authEndpoint,
        jwtSignerPublicKey: 'test'
      });

      await auth.getAuthValue('test');

      expect(authEndpoint).toHaveBeenCalledTimes(1);

      await auth.getAuthValue('test');

      expect(authEndpoint).toHaveBeenCalledTimes(2);
    });

    test('throws AuthFailedError when callback throws', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: () => {
          return Promise.reject('test');
        },
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(AuthFailedError);
    });

    test('throws Error when callback returns non POJO', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: (async () => {
          return null;
        }) as unknown as AuthEndpoint,
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(
        'Expected response to be a plain object'
      );
    });

    test('throws Error when callback returns incorrect data', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: (async () => {
          return { test: 'test_token' };
        }) as unknown as AuthEndpoint,
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(
        'Expected response to contain one of the keys "token" or "error"'
      );
    });

    test('throws Error when callback returns error response', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: async () => {
          return { error: 'Error', reason: 'unknown' };
        },
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError('Auth failed: unknown');
    });

    test('throws AuthFailedError when callback returns forbidden response', async () => {
      const onAuthenticated = vi.fn();

      const auth = createAuth({
        onAuthenticated,
        authEndpoint: async () => {
          return { error: 'forbidden', reason: 'unknown' };
        },
        jwtSignerPublicKey: 'test'
      });

      await expect(auth.getAuthValue('test')).rejects.toThrowError(AuthFailedError);
    });
  });
});
