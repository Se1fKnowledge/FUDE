<script lang="ts">
	import DOMPurify from 'dompurify';
	import { marked } from 'marked';

	// Lesson theory. Imported lessons come from other people, so the HTML is sanitized.
	let { text, skipTitle = false }: { text: string; skipTitle?: boolean } = $props();
	const html = $derived.by(() => {
		const source = skipTitle ? text.replace(/^#\s.*\n+/, '') : text;
		return DOMPurify.sanitize(marked.parse(source, { async: false, gfm: true }) as string, { FORBID_TAGS: ['style', 'form', 'iframe'] });
	});
</script>

<div class="md">{@html html}</div>

<style>
	.md :global(h1) {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 32px;
		line-height: 1.15;
		margin: 0 0 14px;
	}

	.md :global(h2) {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 24px;
		margin: 26px 0 10px;
	}

	.md :global(h3) {
		font-size: 16px;
		margin: 20px 0 8px;
	}

	.md :global(p),
	.md :global(ul),
	.md :global(ol) {
		margin: 0 0 12px;
		color: var(--ink-700);
	}

	.md :global(ul),
	.md :global(ol) {
		padding-left: 22px;
	}

	.md :global(li) {
		margin: 4px 0;
	}

	.md :global(li::marker) {
		color: var(--ai-500);
	}

	.md :global(strong) {
		color: var(--ink-900);
	}

	.md :global(table) {
		width: 100%;
		margin: 6px 0 16px;
		border-collapse: collapse;
		font-size: 14px;
	}

	.md :global(th),
	.md :global(td) {
		padding: 8px 10px;
		border-bottom: 1px solid var(--paper-line);
		text-align: left;
		vertical-align: top;
	}

	.md :global(th) {
		font-size: 12px;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--ink-500);
	}

	.md :global(code) {
		padding: 1px 6px;
		border-radius: 5px;
		background: var(--paper-sunken);
		font-family: var(--font-mono);
		font-size: 12.5px;
	}

	.md :global(h2:last-of-type + p) {
		padding: 14px 16px;
		border-radius: 12px;
		background: var(--ink-900);
		color: var(--sumi-text);
	}
</style>
