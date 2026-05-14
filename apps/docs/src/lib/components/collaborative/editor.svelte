<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Collaboration from '@tiptap/extension-collaboration';
	import CollaborationCaret from '@tiptap/extension-collaboration-caret';
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
				// In Tiptap v3 the History extension was renamed to UndoRedo.
				StarterKit.configure({ undoRedo: false }),
				Collaboration.configure({
					document: provider.getYDoc()
				}),
				CollaborationCaret.configure({
					provider,
					user: getInitialUser()
				})
			],
			// Do not pass `content` to the Editor at construction. The Yjs
			// document syncs from the server through the Collaboration
			// extension, and any local content set before sync ends up as
			// a duplicate write that broadcasts to every peer. The default
			// content is set below in `onCreate`, but only after sync
			// completes and only if the document is genuinely empty.
			onCreate: ({ editor: currentEditor }) => {
				provider.on('synced', () => {
					if (currentEditor.isEmpty) {
						currentEditor.commands.setContent(defaultContent);
					}
				});
			},
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
	/*
		Tiptap v3 renamed the CollaborationCursor extension to
		CollaborationCaret and re-prefixed every emitted CSS class.
		The new class names are `.collaboration-carets__caret`,
		`.collaboration-carets__label`, and `.collaboration-carets__selection`
		(note the plural "carets").
	*/

	:global(.collaboration-carets__caret) {
		border-left: 1px solid #0d0d0d;
		border-right: 1px solid #0d0d0d;
		margin-left: -1px;
		margin-right: -1px;
		pointer-events: none;
		position: relative;
		word-break: normal;
	}

	/* Render the username above the caret */
	:global(.collaboration-carets__label) {
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

	/* Tint the remote user's selected range with their assigned color. */
	:global(.collaboration-carets__selection) {
		pointer-events: none;
		word-break: normal;
	}
</style>
