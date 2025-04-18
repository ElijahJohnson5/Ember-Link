<script lang="ts">
	import '../app.css';
	import { EmberLinkProvider } from '@ember-link/svelte';
	import { PUBLIC_JWT_SIGNER_KEY } from '$env/static/public';

	let { children } = $props();
</script>

<EmberLinkProvider
	baseUrl="http://localhost:8787"
	authEndpoint="http://localhost:5173/api/auth"
	jwtSignerPublicKey={PUBLIC_JWT_SIGNER_KEY}
	multiTenant={{
		tenantId: 'test'
	}}
>
	{@render children()}
</EmberLinkProvider>
