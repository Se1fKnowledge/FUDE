<script lang="ts">
	import type { LessonCard } from '#lib/api.ts';
	import Brush from './Brush.svelte';
	import Difficulty from './Difficulty.svelte';
	import Icon from './Icon.svelte';

	// A recommended lesson on the home screen: category kanji on a tinted cover.
	let { card }: { card: LessonCard } = $props();
	const tint = $derived({ ai: 'var(--ai-100)', shu: 'var(--shu-100)', kaki: 'var(--kaki-100)', kin: 'var(--kin-100)' }[card.accent]);
	const ink = $derived({ ai: 'var(--ai-500)', shu: 'var(--shu-500)', kaki: 'var(--kaki-500)', kin: 'var(--kin-500)' }[card.accent]);
</script>

<a class="tile card" href="/lesson/{card.id}">
	<div class="cover" style="background: {tint}">
		<span class="tag">{card.categoryTitle} · урок {card.number}</span>
		<span class="stroke"><Brush width={170} height={34} seed={card.number * 7 + card.title.length} color="var(--ink-900)" /></span>
		<span class="kanji glyph" style="color: {ink}">{card.kanji ?? '筆'}</span>
	</div>
	<div class="body">
		<h3 class="h3">{card.title}</h3>
		<div class="meta">
			<span class="time"><Icon name="clock" size={15} /> {card.minutes} мин</span>
			<Difficulty level={card.difficulty} label={card.difficultyLabel} />
		</div>
	</div>
</a>

<style>
	.tile {
		display: grid;
		grid-template-rows: 150px auto;
		overflow: hidden;
		color: inherit;
		transition:
			transform 200ms var(--ease-out),
			box-shadow 200ms;
	}

	.tile:hover {
		transform: translateY(-3px);
		box-shadow: var(--shadow-float);
	}

	.cover {
		position: relative;
		overflow: hidden;
	}

	.tag {
		position: absolute;
		left: 14px;
		top: 14px;
		padding: 3px 10px;
		border-radius: 99px;
		background: var(--paper-raised);
		font-size: 12px;
		font-weight: 600;
	}

	.stroke {
		position: absolute;
		left: 14px;
		bottom: 30px;
		transform: rotate(-12deg);
	}

	.glyph {
		position: absolute;
		right: 22px;
		top: 22px;
		font-size: 96px;
		line-height: 1;
	}

	.body {
		display: grid;
		gap: 14px;
		padding: 18px 18px 20px;
	}

	.meta {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		font-size: 13px;
		color: var(--ink-500);
	}

	.time {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
</style>
