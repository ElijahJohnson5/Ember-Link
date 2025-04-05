<script lang="ts">
	import CollaborativeExample from '$lib/components/collaborative/index.svelte';
</script>

<div class="flex w-full justify-center p-6">
	<div
		class="flex w-full flex-col divide-y-2 border-2 md:flex-row md:justify-center md:divide-x-2 md:divide-y-0"
	>
		<CollaborativeExample />
		<CollaborativeExample />
	</div>
</div>
