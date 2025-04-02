import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});

const { channel } = client.joinChannel('test');

/**
 * Subscribe to every others presence updates.
 * The callback will be called if you or someone else enters or leaves the channel
 * or when someone presence is updated
 */
channel.events.others.subscribe('join', (user) => {
	console.log('User join: ', user);
});

channel.events.others.subscribe('update', (user) => {
	console.log('User update: ', user);
});

channel.events.others.subscribe('leave', (user) => {
	console.log('User leave: ', user);
});
