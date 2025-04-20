# @ember-link/react

## What is @ember-link/react?

> **@ember-link/react** @ember-link/react is the official React integration for the Ember Link SDK. It provides context providers and React hooks that make it easy to connect to real-time channels, manage presence, and send custom messages using idiomatic React patterns.

## Installation

```bash copyButton
yarn install @ember-link/react
```

## Basic Usage

```tsx copyButton
import {
	EmberLinkProvider,
	ChannelProvider,
	useOthers,
	useMyPresence,
	useCustomMessage
} from '@ember-link/react';

function App() {
	return (
		<EmberLinkProvider baseUrl="http://localhost:9000">
			<ChannelProvider channelName="test">
				<Page />
			</ChannelProvider>
		</EmberLinkProvider>
	);
}

function Page() {
	// Get the base channel if you need to
	const channel = useChannel();
	const others = useOthers();
	// myPresence is this users presence, and setMyPresence updates their presence on the server
	const [myPresence, setMyPresence] = useMyPresence();
	// sendCustomMessage is the function you can call to send a message
	const sendCustomMessage = useCustomMessage((message) => {
		console.log('Got custom message: ', message);
	});

	useEffect(() => {
		setMyPresence({
			online: true
		});
	}, [setMyPresence]);

	return (
		<>
			{others.map(() => {
				return <div>{others.clientId}</div>;
			})}
		</>
	);
}
```

## API

### Hooks

#### **useChannel()**

Returns the current channel instance provided by the closest ChannelProvider.

Useful for advanced use-cases where you want direct access to low-level channel methods.

#### **useOthers()**

Returns an array of presence objects for other users currently connected to the same channel.

```ts
type useOthers = <P extends DefaultPresence, C extends DefaultCustomMessageData>() => User<P>[];
```

#### **useMyPresence()**

Returns a tuple of:

The current user's presence object

A setter function to update presence

Updates are automatically propagated to other users.

#### **useCustomMessage**

Registers a handler for custom messages and returns a function to send messages:

```ts
const sendMessage = useCustomMessage((message) => {
	console.log('Got a message!', message);
});

sendMessage({ type: 'ping' });
```

#### **useStatus**

Returns the current status of the underlying websocket connection

#### **useArrayStorage(name: string)**

Returns a CRDT-backed array that is synced across all users in the channel.

```ts
const items = useArrayStorage('test');
```

This is a wrapper over [@ember-link/storage - ArrayStorage](/packages/storage#array-storage).

To access the current up to date array use `items.current`.

#### **useMapStorage(name: string)**

Returns a CRDT-backed Map that is synced across all users in the channel.

```ts
const items = useMapStorage('test');
```

This is a wrapper over [@ember-link/storage - MapStorage](/packages/storage#map-storage).

To access the current up to date map use `items.current`.

## Related

- [@ember-link/core](/packages/core) – Core WebSocket + channel logic

- [@ember-link/svelte](/packages/svelte) – Svelte integration
