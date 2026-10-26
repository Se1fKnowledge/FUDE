<script lang="ts">
	import type { Range } from '#lib/api.ts';

	// «от … до …» with either end optional. `scale` shows fractions as percent (100).
	let { value = $bindable(), scale = 1, unit = '', label = '' }: { value: Range; scale?: number; unit?: string; label?: string } = $props();

	function set(end: 'min' | 'max', text: string) {
		const n = Number(text.replace(',', '.'));
		const next = { ...value };
		if (text.trim() === '' || Number.isNaN(n)) delete next[end];
		else next[end] = Math.round((n / scale) * 10000) / 10000;
		value = next;
	}

	const show = (v: number | undefined) => (v === undefined ? '' : String(Math.round(v * scale * 100) / 100));
</script>

<span class="range">
	{#if label}<span class="label">{label}</span>{/if}
	<span>от</span>
	<input type="text" inputmode="decimal" value={show(value.min)} onchange={(e) => set('min', e.currentTarget.value)} aria-label="{label} от" />
	<span>до</span>
	<input type="text" inputmode="decimal" value={show(value.max)} onchange={(e) => set('max', e.currentTarget.value)} aria-label="{label} до" />
	<span class="unit">{unit}</span>
</span>

<style>
	.range {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		color: var(--ink-700);
	}

	input {
		width: 72px;
		padding: 6px 8px;
	}

	.unit {
		color: var(--ink-500);
	}
</style>
