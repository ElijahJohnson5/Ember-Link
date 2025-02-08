import * as jose from 'jose';

export interface AccessToken {
  iat: number;
  exp: number;
  uid: string;
}

export type AuthValue = { type: 'public' } | { type: 'private'; token: AuthToken };

export interface NoAuthToken {
  type: 'NoAuth';
}

export interface AuthToken {
  type: 'AuthToken';
  readonly raw: string;
  readonly parsed: jose.JWTVerifyResult<AccessToken>;
  readonly tenantId?: string;
}

interface AuthOptions {
  authEndpoint?: AuthEndpoint;
  jwtSignerPublicKey?: string;
  multiTenant?: {
    tenantId: string;
  };
  onAuthenticated?: (token: AuthValue) => void;
}

export type AuthCallback = (
  channelName?: string
) => Promise<{ token: string } | { error: string; reason: string }>;

export type AuthEndpoint = string | AuthCallback;

export class AuthFailedError extends Error {
  constructor(reason: string) {
    super(reason);
  }
}

type AuthType =
  | { type: 'private'; url: string; jwtSignerPublicKey: string }
  | { type: 'callback'; callback: AuthCallback; jwtSignerPublicKey: string }
  | { type: 'none' };

export function createAuth(options: AuthOptions) {
  const { multiTenant } = options;

  let auth: AuthType;

  if (!options.authEndpoint) {
    auth = {
      type: 'none'
    };
  } else if (typeof options.authEndpoint === 'string') {
    if (!options.jwtSignerPublicKey) {
      throw new Error(
        'Could not create auth, if authEndpoint is specified jwtSignerPublicKey is required'
      );
    }

    auth = {
      type: 'private',
      url: options.authEndpoint,
      jwtSignerPublicKey: options.jwtSignerPublicKey
    };
  } else {
    if (!options.jwtSignerPublicKey) {
      throw new Error(
        'Could not create auth, if authEndpoint is specified jwtSignerPublicKey is required'
      );
    }

    auth = {
      type: 'callback',
      callback: options.authEndpoint,
      jwtSignerPublicKey: options.jwtSignerPublicKey
    };
  }

  const requestPromises = new Map<string, Promise<AuthToken | NoAuthToken>>();
  const cachedTokens: { token: AuthToken; expiresAt: number }[] = [];

  function getCachedToken(_channelName: string): AuthToken | undefined {
    const now = Math.ceil(Date.now() / 1000);

    let index = 0;
    for (const { token, expiresAt } of cachedTokens) {
      if (expiresAt <= now) {
        cachedTokens.splice(index, 1);
        index--;
        continue;
      }

      return token;
    }
  }

  // Returns NoAuthToken when no auth is required. Else returns the AuthToken or throws Errors
  async function requestAuth(channelName: string): Promise<AuthToken | NoAuthToken> {
    if (auth.type === 'private') {
      const url = new URL(auth.url);

      if (multiTenant) {
        url.searchParams.append('tenant_id', multiTenant.tenantId);
      }

      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ channelName })
      });

      if (!response.ok) {
        const reason = `${
          (await response.text()).trim() || 'reason not provided in auth response'
        } (${response.status} returned by POST ${auth.url})`;

        if (response.status === 401 || response.status === 403) {
          throw new AuthFailedError(`Unauthorized: ${reason}`);
        } else {
          throw new Error(`Failed to authenticate: ${reason}`);
        }
      }

      let data: unknown;

      try {
        data = await response.json();
      } catch (e) {
        throw new Error(
          `Expected a json response when trying to authenticate to ${auth.url}. ${String(e)}`
        );
      }

      if (!isPOJO(data) || typeof data.token !== 'string') {
        throw new Error('Expected json to be a plain object with a string token');
      }

      // TODO: refactor to not unsecured jwts
      const publicKey = await jose.importSPKI(auth.jwtSignerPublicKey, 'RS256');
      const parsed = await jose.jwtVerify<AccessToken>(data.token, publicKey);
      const authToken: AuthToken = {
        raw: data.token,
        parsed,
        type: 'AuthToken',
        tenantId: multiTenant?.tenantId
      };
      options.onAuthenticated?.({ type: 'private', token: authToken });
      return authToken;
    } else if (auth.type === 'callback') {
      let response: Awaited<ReturnType<AuthCallback>>;

      try {
        response = await auth.callback(channelName);
      } catch (e) {
        throw new AuthFailedError(`Auth failed: ${String(e)}`);
      }

      if (!isPOJO(response)) {
        throw new Error(
          'Expected response to be a plain object with a string token or string error'
        );
      }

      if (!('token' in response) && !('error' in response)) {
        throw new Error('Expected response to contain one of the keys "token" or "error"');
      }

      if ('error' in response) {
        const reason = `Auth failed: ${'reason' in response && typeof response.reason === 'string' ? response.reason : 'Forbidden'}`;

        if (response.error === 'forbidden') {
          throw new AuthFailedError(reason);
        } else {
          throw new Error(reason);
        }
      }

      const publicKey = await jose.importSPKI(auth.jwtSignerPublicKey, 'RS256');
      const parsed = await jose.jwtVerify<AccessToken>(response.token, publicKey);
      const authToken: AuthToken = {
        raw: response.token,
        parsed,
        type: 'AuthToken',
        tenantId: multiTenant?.tenantId
      };
      options.onAuthenticated?.({ type: 'private', token: authToken });
      return authToken;
    } else {
      options.onAuthenticated?.({ type: 'public' });
      return { type: 'NoAuth' };
    }
  }

  async function getAuthValue(channelName: string): Promise<AuthValue> {
    if (auth.type === 'none') {
      // Call on authenticated here since when auth type is none we short circuit and just do the public value here
      options.onAuthenticated?.({ type: 'public' });
      return { type: 'public' };
    }

    const cachedToken = getCachedToken(channelName);
    if (cachedToken !== undefined) {
      return { type: 'private', token: cachedToken };
    }

    let promise = requestPromises.get(channelName);

    if (!promise) {
      promise = requestAuth(channelName);
      requestPromises.set(channelName, promise);
    }

    try {
      const authValue = await promise;

      if (authValue.type === 'NoAuth') {
        return { type: 'public' };
      }

      const buffer = 30;
      const expiresAt =
        Math.floor(Date.now() / 1000) +
        (authValue.parsed.payload.exp - authValue.parsed.payload.iat) -
        buffer;

      cachedTokens.push({ token: authValue, expiresAt });

      return { type: 'private', token: authValue };
    } finally {
      requestPromises.delete(channelName);
    }
  }

  return {
    getAuthValue
  };
}

export function isPOJO(data: unknown): data is { [key: string]: unknown } {
  return (
    data !== null &&
    typeof data === 'object' &&
    Object.prototype.toString.call(data) === '[object Object]'
  );
}
