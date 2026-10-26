<script lang="ts">
	import Icon from './Icon.svelte';

	// The top of the lesson path (screens 06–07): back, title, the three stages, close.
	let {
		title,
		subtitle,
		stage,
		back,
		onclose,
		dark = false,
		xp = null
	}: { title: string; subtitle: string; stage: 1 | 2 | 3; back: string; onclose?: () => void; dark?: boolean; xp?: number | null } = $props();

	const stages = ['Гайд и теория', 'Практика', 'Итог'];
</script>

<header class:dark>
	<a class="round" href={back} aria-label="Назад"><Icon name="back" /></a>
	<div class="title">
		<h1>{title}</h1>
		<p>{subtitle}</p>
	</div>
	<ol class="stages" aria-label="Этапы урока">
		{#each stages as s, i (s)}
			<li class:active={stage === i + 1} class:done={stage > i + 1} aria-current={stage === i + 1 ? 'step' : undefined}>
				<span class="n">{#if stage > i + 1}<Icon name="check" size={14} stroke={2.4} />{:else}{i + 1}{/if}</span>
				{s}
			</li>
		{/each}
	</ol>
	{#if xp}<span class="xp"><Icon name="sparkle" size={15} /> +{xp} XP за урок</span>{/if}
	<a class="round" href="/" aria-label="Закрыть урок" onclick={onclose}><Icon name="close" /></a>
</header>

<style>
	header {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto auto auto;
		align-items: center;
		gap: 18px;
		padding: 18px 40px;
	}

	.round {
		display: grid;
		place-items: center;
		width: 42px;
		height: 42px;
		border-radius: 50%;
		background: var(--paper-raised);
		box-shadow: var(--shadow-card);
		color: var(--ink-900);
	}

	h1 {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 30px;
		line-height: 1.1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	p {
		font-size: 13px;
		color: var(--ink-500);
	}

	.stages {
		display: flex;
		align-items: center;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 14px;
		color: var(--ink-500);
	}

	.stages li {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.stages li + li::before {
		content: '';
		width: 34px;
		height: 2px;
		margin-right: 4px;
		border-radius: 2px;
		background: var(--ink-300);
	}

	.n {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 1.5px var(--ink-300);
		font-size: 13px;
		font-weight: 600;
	}

	.active {
		color: var(--ink-900);
		font-weight: 600;
	}

	.active .n {
		background: var(--shu-500);
		box-shadow: none;
		color: #fff;
	}

	.done .n {
		background: var(--ai-500);
		box-shadow: none;
		color: #fff;
	}

	.xp {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 99px;
		background: var(--kaki-100);
		color: var(--kaki-700);
		font-size: 13px;
		font-weight: 600;
	}

	.dark {
		background: var(--sumi-bg);
		color: var(--sumi-text);
		padding: 10px 24px;
	}

	.dark .round {
		background: var(--sumi-raised);
		color: var(--sumi-text);
		box-shadow: none;
	}

	.dark h1 {
		font-family: var(--font-ui);
		font-size: 16px;
		font-weight: 600;
	}

	.dark p,
	.dark .stages {
		color: var(--sumi-muted);
	}

	.dark .active {
		color: var(--sumi-text);
	}

	.dark .stages li + li::before {
		background: var(--sumi-line);
	}

	.dark .n {
		box-shadow: inset 0 0 0 1.5px var(--sumi-line);
	}
</style>
