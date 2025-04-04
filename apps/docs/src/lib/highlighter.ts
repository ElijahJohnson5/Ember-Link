import { createHighlighter } from 'shiki';

let highlighterPromise: ReturnType<typeof createHighlighter> | null;

const getHighlighter = () => {
	if (highlighterPromise) {
		return highlighterPromise;
	}

	highlighterPromise = createHighlighter({
		themes: ['github-dark', 'github-light'],
		langs: ['javascript', 'typescript', 'sh', 'tsx', 'jsx', 'svelte']
	});

	return highlighterPromise;
};

export { getHighlighter };
