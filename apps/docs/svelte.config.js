import { mdsvex, escapeSvelte } from 'mdsvex';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { createHighlighter } from 'shiki';

export function parseMetaCopyButton(meta) {
	if (!meta) return null;
	const match = meta.match(/copyButton/);
	if (!match) return null;

	return match;
}

const highlighter = await createHighlighter({
	themes: ['github-dark', 'github-light'],
	langs: ['javascript', 'typescript', 'sh', 'tsx', 'jsx', 'svelte']
});

const config = {
	preprocess: [
		vitePreprocess(),
		mdsvex({
			highlight: {
				highlighter: async (code, lang = 'text', meta) => {
					const html = escapeSvelte(
						highlighter.codeToHtml(code, {
							lang,
							themes: {
								light: 'github-light',
								dark: 'github-dark'
							},
							transformers: [
								{
									root(node) {
										if (!meta) {
											return;
										}

										const match = parseMetaCopyButton(meta);

										if (!match) {
											return;
										}

										const newNode = {
											type: 'element',
											tagName: 'div',
											properties: {
												class: 'code-container'
											},
											children: [node]
										};

										node.children.push({
											type: 'element',
											tagName: 'button',
											properties: {
												type: 'button',
												data: this.source,
												title: 'Copy code',
												'aria-label': 'Copy code',
												class: 'copy-button',
												'data-name': 'copy-button',
												onclick: /* javascript */ `
													navigator.clipboard.writeText(this.attributes.data.value);
													this.classList.add('copied');
													window.setTimeout(() => this.classList.remove('copied'), ${3000});
												`
											},
											children: [
												{
													type: 'element',
													tagName: 'span',
													properties: { class: 'ready' },
													children: []
												},
												{
													type: 'element',
													tagName: 'span',
													properties: { class: 'success' },
													children: []
												}
											]
										});

										return newNode;
									}
								}
							]
						})
					);

					return `{@html \`${html}\` }`;
				}
			},
			extensions: ['.md', '.svx']
		})
	],
	kit: { adapter: adapter() },
	extensions: ['.svelte', '.md', '.svx']
};

export default config;
