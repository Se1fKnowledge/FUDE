<script lang="ts">
	import { DABS, maskUrl } from '#lib/ink.ts';

	// Five dabs of real ink, filled up to the level, with the word: colour is never the only cue.
	let { level, label, compact = false }: { level: number; label: string; compact?: boolean } = $props();
</script>

<span class="difficulty" class:compact title="Сложность {level} из 5">
	<span class="dabs" aria-hidden="true">
		{#each DABS as src, i (src)}
			<span class="dab" class:on={i < level} style="mask-image: {maskUrl(src)}"></span>
		{/each}
	</span>
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

	.dabs {
		display: inline-flex;
		gap: 2px;
		width: 96px;
	}

	.dab {
		width: 17px;
		height: 10px;
		background: var(--paper-line);
		mask-size: 100% 100%;
		mask-repeat: no-repeat;
		transform: rotate(-8deg);
	}

	.dab.on {
		background: var(--ink-900);
	}

	.label {
		font-size: 13px;
		color: var(--ink-700);
	}
</style>
