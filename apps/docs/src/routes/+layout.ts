import type { LayoutLoad } from './$types';
import { createHighlighter } from 'shiki';

export const prerender = true;

const highlighter = await createHighlighter({
	themes: ['github-dark', 'github-light'],
	langs: ['javascript', 'typescript']
});

export const load = (async () => {
	return { highlighter };
}) satisfies LayoutLoad;
