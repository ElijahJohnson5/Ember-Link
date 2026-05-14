# svelte-auth

SvelteKit example that demonstrates how to authenticate users against an Ember Link server using JWTs signed with RS256. The app exposes a `/api/auth` endpoint that signs a short-lived token for each connecting client and returns the matching public key when the server requests one.

## Getting Started

1. Make sure you have the Ember Link server running locally. Instructions are in the [root README](../../README.md). Run it **without** `ALLOW_UNAUTHORIZED=true` so it requires JWTs.

2. Install dependencies:

   ```sh
   yarn install
   ```

3. Generate an RSA key pair (used for signing and verifying JWTs):

   ```sh
   openssl genpkey -algorithm RSA -out private.pem -pkeyopt rsa_keygen_bits:2048
   openssl rsa -in private.pem -pubout -out public.pem
   ```

4. Copy `.env.example` to `.env` and paste the contents of `private.pem` into `JWT_SECRET_KEY` and `public.pem` into `PUBLIC_JWT_SIGNER_KEY`. Both should be the full PEM, newlines included. Then make sure the Ember Link server has the same public key set as `JWT_SIGNER_KEY`.

5. Run the dev server:

   ```sh
   yarn dev
   ```

6. Open the URL Vite prints. The page connects to the channel `test`; open it in two windows to see presence sync over an authenticated connection.

## How it works

- `POST /api/auth` — called by the Ember Link client SDK to obtain a JWT for the channel it's about to join. Signs the token with `JWT_SECRET_KEY`.
- `GET /api/auth` — returns the public signer key so the SDK can hand it to the server for verification. Reads `PUBLIC_JWT_SIGNER_KEY` at build time via `$env/static/public`.
- `src/routes/+page.svelte` — minimal client that wraps the page in `ChannelProvider` from `@ember-link/svelte`.
