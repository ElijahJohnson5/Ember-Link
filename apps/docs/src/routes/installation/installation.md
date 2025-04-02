<script lang="ts">
  import { selectedLibrary } from '$lib/library.svelte';
</script>

# **Installation & Basic Usage**

Ember Link makes adding real-time collaboration to your app effortless. This guide will help you set up Ember Link and start using its core features.

## **Installation**

You can install Ember Link via **npm** or **yarn** or any other package manager you use:

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

```sh
npm install @ember-link/core
# or
yarn add @ember-link/core
```

{/if}

{#if selectedLibrary.current.value === 'react'}

```sh
npm install @ember-link/react
# or
yarn add @ember-link/react
```

{/if}

{#if selectedLibrary.current.value === 'svelte'}

```sh
npm install @ember-link/svelte
# or
yarn add @ember-link/svelte
```

{/if}

## **Connecting to Ember Link**

After installing, import and create a client instance:

```typescript
import { createClient } from 'ember-link';

const client = createClient({
	serverUrl: 'https://your-server.com',
	roomId: 'my-collaborative-room'
});

await client.connect();
console.log('Connected to Ember Link!');
```

## **Listening to User Presence**

Track when users join or leave the collaboration session:

```typescript
client.subscribeToPresence((users) => {
	console.log('Active users:', users);
});

// Update your own presence
client.updatePresence({ status: 'online' });
```

## **Sending Custom Messages**

Send real-time messages to all connected clients:

```typescript
client.sendMessage('chat', { text: 'Hello, team!' });

client.onMessage('chat', (message) => {
	console.log('Received message:', message.text);
});
```

## **Using Shared Storage (CRDTs)**

Ember Link provides **conflict-free replicated data types (CRDTs)** to sync state across users:

```typescript
const doc = client.getDocument('shared-data');

doc.update((state) => {
	state.text = 'Hello, collaborative world!';
});

doc.subscribe((state) => {
	console.log('Updated state:', state.text);
});
```

## **Next Steps**

- Check out the [full API documentation](#)
- Explore advanced features like permissions & custom events
- Join the Ember Link community for support & updates
