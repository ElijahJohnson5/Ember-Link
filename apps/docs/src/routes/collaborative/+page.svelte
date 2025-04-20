<script lang="ts">
	import CollaborativeExample from '$lib/components/collaborative/index.svelte';
	import { Button } from '$lib/components/ui/button';
</script>

<div class="flex h-full w-full flex-col">
	<div>
		<Button
			variant="link"
			class="px-6 pt-2"
			target="_blank"
			href="https://github.com/ElijahJohnson5/Ember-Link/tree/main/apps/docs/src/lib/components/collaborative"
		>
			View the code here
		</Button>
	</div>

	<div class="flex w-full flex-grow justify-center p-6 pt-2">
		<div
			class="flex w-full flex-col divide-y-2 border-2 md:flex-row md:justify-center md:divide-x-2 md:divide-y-0"
		>
			<CollaborativeExample />
			<CollaborativeExample />
		</div>
	</div>
</div>
