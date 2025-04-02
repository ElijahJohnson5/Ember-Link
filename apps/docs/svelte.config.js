import { mdsvex, escapeSvelte } from 'mdsvex';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { createHighlighter } from 'shiki';

const highlighter = await createHighlighter({
	themes: ['github-dark', 'github-light'],
	langs: ['javascript', 'typescript', 'sh']
});

const config = {
	preprocess: [
		vitePreprocess(),
		mdsvex({
			highlight: {
				highlighter: async (code, lang = 'text') => {
					const html = escapeSvelte(
						highlighter.codeToHtml(code, {
							lang,
							themes: {
								light: 'github-light',
								dark: 'github-dark'
							}
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
