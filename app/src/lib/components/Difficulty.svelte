<script lang="ts">
	// Five ink dabs, filled up to the level, with the word: colour is never the only cue.
	let { level, label, compact = false }: { level: number; label: string; compact?: boolean } = $props();
</script>

<span class="difficulty" class:compact title="Сложность {level} из 5">
	<svg width="96" height="14" viewBox="0 0 96 14" aria-hidden="true">
		{#each [0, 1, 2, 3, 4] as i (i)}
			<path
				d="M{i * 19 + 2} 9.5c3-4 9-6.5 14-5.5 1 .3 1 1.2.2 1.6-3.8 1.8-8.6 3.6-13 4.9-1.3.4-2-.2-1.2-1z"
				fill={i < level ? 'var(--ink-900)' : 'var(--paper-line)'}
				transform="rotate(-8 {i * 19 + 9} 7)"
			/>
		{/each}
	</svg>
	{#if !compact}<span class="label">{label}</span>{/if}
	<span class="sr-only">Сложность: {label}, {level} из 5</span>
</span>

<style>
	.difficulty {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		white-space: nowrap;
	}

	.label {
		font-size: 13px;
		color: var(--ink-700);
	}
</style>
