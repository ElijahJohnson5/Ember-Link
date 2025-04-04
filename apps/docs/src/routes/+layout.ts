import { getHighlighter } from '$lib/highlighter';
import type { LayoutLoad } from './$types';

export const prerender = true;

export const load = (async () => {
	return { highlighter: await getHighlighter() };
}) satisfies LayoutLoad;
