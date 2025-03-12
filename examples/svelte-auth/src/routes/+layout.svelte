<script lang="ts">
	import '../app.css';
	import { ClientProvider } from '@ember-link/svelte';
	import { PUBLIC_JWT_SIGNER_KEY } from '$env/static/public';

	let { children } = $props();
</script>

<ClientProvider
	clientOptions={{
		baseUrl: 'http://localhost:9000',
		authEndpoint: 'http://localhost:5173/api/auth',
		jwtSignerPublicKey: PUBLIC_JWT_SIGNER_KEY,
		multiTenant: {
			tenantId: 'test'
		}
	}}
>
	{@render children()}
</ClientProvider>
