<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Collaboration from '@tiptap/extension-collaboration';
	import CollaborationCursor from '@tiptap/extension-collaboration-cursor';
	import { EmberLinkYjsProvider, getYjsProviderForChannel } from '@ember-link/yjs-provider';
	import { getChannelContext } from '@ember-link/svelte';
	import { Badge } from '$lib/components/ui/badge';

	let element: HTMLDivElement;
	let editor: Editor;
	let provider: EmberLinkYjsProvider;

	const colors = [
		'#958DF1',
		'#F98181',
		'#FBBC88',
		'#FAF594',
		'#70CFF8',
		'#94FADB',
		'#B9F18D',
		'#C3E2C2',
		'#EAECCC',
		'#AFC8AD',
		'#EEC759',
		'#9BB8CD',
		'#FF90BC',
		'#FFC0D9',
		'#DC8686',
		'#7ED7C1',
		'#F3EEEA',
		'#89B9AD',
		'#D0BFFF',
		'#FFF8C9',
		'#CBFFA9',
		'#9BABB8',
		'#E3F4F4'
	];
	const names = [
		'Lea Thompson',
		'Cyndi Lauper',
		'Tom Cruise',
		'Madonna',
		'Jerry Hall',
		'Joan Collins',
		'Winona Ryder',
		'Christina Applegate',
		'Alyssa Milano',
		'Molly Ringwald',
		'Ally Sheedy',
		'Debbie Harry',
		'Olivia Newton-John',
		'Elton John',
		'Michael J. Fox',
		'Axl Rose',
		'Emilio Estevez',
		'Ralph Macchio',
		'Rob Lowe',
		'Jennifer Grey',
		'Mickey Rourke',
		'John Cusack',
		'Matthew Broderick',
		'Justine Bateman',
		'Lisa Bonet'
	];

	const defaultContent = `
  <p>Hi 👋, this is a collaborative document.</p>
  <p>Feel free to edit and collaborate in real-time!</p>
`;

	const getRandomElement = (list: string[]) => list[Math.floor(Math.random() * list.length)];

	const getRandomColor = () => getRandomElement(colors);
	const getRandomName = () => getRandomElement(names);

	const getInitialUser = () => {
		return {
			name: getRandomName(),
			color: getRandomColor()
		};
	};

	const channel = getChannelContext();

	onMount(() => {
		provider = getYjsProviderForChannel(channel.getRawChannel());

		editor = new Editor({
			element: element,
			editorProps: {
				attributes: {
					class: 'h-full prose mx-auto w-full max-w-4xl p-6 dark:prose-invert'
				}
			},
			extensions: [
				StarterKit.configure({ history: false }),
				Collaboration.extend().configure({
					document: provider.getYDoc()
				}),

				CollaborationCursor.extend().configure({
					provider,
					user: getInitialUser()
				})
			],
			onCreate: ({ editor: currentEditor }) => {
				provider.on('synced', () => {
					if (currentEditor.isEmpty) {
						currentEditor.commands.setContent(defaultContent);
					}
				});
			},
			content: defaultContent,
			onTransaction: () => {
				// force re-render so `editor.isActive` works as expected
				editor = editor;
			}
		});
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
		}
	});
</script>

<div class="relative flex w-full flex-grow flex-col">
	<div class="absolute right-2 top-2">
		<Badge>
			{channel.status.toUpperCase()}
		</Badge>
	</div>
	<div class="w-full flex-grow" bind:this={element}></div>
</div>

<style>
	:global(.collaboration-cursor__caret) {
		border-left: 1px solid #0d0d0d;
		border-right: 1px solid #0d0d0d;
		margin-left: -1px;
		margin-right: -1px;
		pointer-events: none;
		position: relative;
		word-break: normal;
	}

	/* Render the username above the caret */
	:global(.collaboration-cursor__label) {
		border-radius: 3px 3px 3px 0;
		color: #0d0d0d;
		font-size: 12px;
		font-style: normal;
		font-weight: 600;
		left: -1px;
		line-height: normal;
		padding: 0.1rem 0.3rem;
		position: absolute;
		top: -1.4em;
		user-select: none;
		white-space: nowrap;
	}
</style>
