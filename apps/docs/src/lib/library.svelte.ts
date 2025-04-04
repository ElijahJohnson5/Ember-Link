import Javascript from '$lib/components/icons/javascript.svelte';
import React from '$lib/components/icons/react.svelte';
import Svelte from '$lib/components/icons/svelte.svelte';
import Typescript from '$lib/components/icons/typescript.svelte';
import type { Component } from 'svelte';

interface LibraryOption {
	display: string;
	value: 'js' | 'ts' | 'react' | 'svelte';
	icon: Component;
}

const libraryOptions: Array<LibraryOption> = [
	{
		display: 'Javascript',
		value: 'js' as const,
		icon: Javascript
	},
	{
		display: 'Typescript',
		value: 'ts' as const,
		icon: Typescript
	},
	{
		display: 'React',
		value: 'react' as const,
		icon: React
	},
	{
		display: 'Svelte',
		value: 'svelte' as const,
		icon: Svelte
	}
];

class SelectedLibrary {
	current = $state(libraryOptions[0]);
}

const selectedLibrary = new SelectedLibrary();

export { selectedLibrary, libraryOptions };
