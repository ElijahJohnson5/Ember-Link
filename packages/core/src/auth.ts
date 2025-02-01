interface AccessToken {
  iat: number;
  exp: number;
  uid: string;
}

interface AuthOptions {
  authEndpoint: AuthEndpoint;
  onAuthenticated: (token: AccessToken) => void;
}

export type AuthCallback = (
  channelName: string
) => Promise<{ token: string } | { error: string; reason: string }>;

export type AuthEndpoint = string | AuthCallback;

export class AuthFailedError extends Error {
  constructor(reason: string) {
    super(reason);
  }
}

type AuthType = { type: 'private'; url: string } | { type: 'callback'; callback: AuthCallback };

export function createAuth(options: AuthOptions) {
  let auth: AuthType;

  if (typeof options.authEndpoint === 'string') {
    auth = {
      type: 'private',
      url: options.authEndpoint
    };
  } else {
    auth = {
      type: 'callback',
      callback: options.authEndpoint
    };
  }

  async function requestAuth(channelName?: string) {
    if (auth.type === 'private') {
      const response = await fetch(auth.url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ channel: channelName })
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

      // TODO: Parse token
      return { token: data.token };
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

      // TODO: Parse token
      return { token: response.token };
    }
  }

  return {
    requestAuth
  };
}

export function isPOJO(data: unknown): data is { [key: string]: unknown } {
  return (
    data !== null &&
    typeof data === 'object' &&
    Object.prototype.toString.call(data) === '[object Object]'
  );
}
