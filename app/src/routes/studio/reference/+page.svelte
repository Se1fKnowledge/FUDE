<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
	import { onMount } from 'svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { KEYS, SECTIONS, shotSize, shotUrl, type RefSection } from '#lib/studio/reference.ts';

	// The reference to the lesson editor: a table of contents on the left, on the right every
	// part of the workshop and the studio with its screenshot and what each control does.
	const KEYS_ID = 'keys';
	const toc = [...SECTIONS.map((s) => ({ id: s.id, title: s.title, group: s.group })), { id: KEYS_ID, title: t('Горячие клавиши'), group: t('Горячие клавиши') }];

	let active = $state(toc[0].id);
	let zoomed = $state<RefSection | null>(null);

	function go(id: string) {
		active = id;
		document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') zoomed = null;
	}

	// The contents follow the reading: the section nearest the top of the window is the current one.
	onMount(() => {
		const seen = new IntersectionObserver(
			(entries) => {
				for (const e of entries) if (e.isIntersecting) active = e.target.id;
			},
			{ rootMargin: '-15% 0px -75% 0px' }
		);
		for (const s of toc) {
			const el = document.getElementById(s.id);
			if (el) seen.observe(el);
		}
		return () => seen.disconnect();
	});
</script>

<svelte:window {onkeydown} />

{#snippet picture(s: RefSection)}
	{@const [width, height] = shotSize(s.shot ?? '')}
	<img src={shotUrl(s.shot ?? '')} alt={t('Снимок экрана: {title}', { title: s.title })} {width} {height} />
	{#each s.marks ?? [] as [n, x, y] (n)}
		<span class="dot" style="left: {x}%; top: {y}%" aria-hidden="true">{n}</span>
	{/each}
{/snippet}

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">{t('Справочник по редактору уроков')}</h1>
			<p>{t('Все функции мастерской и студии: что делает каждая кнопка и где она находится.')}</p>
		</div>
		<a class="btn btn-quiet" href="/studio"><Icon name="back" size={18} /> {t('В мастерскую')}</a>
	</header>

	<div class="layout">
		<nav class="toc" aria-label={t('Разделы справочника')}>
			{#each toc as item, i (item.id)}
				{#if item.group !== item.title && item.group !== toc[i - 1]?.group}
					<p class="caps group">{item.group}</p>
				{/if}
				<button class:on={active === item.id} class:sub={item.group !== item.title} aria-current={active === item.id ? 'true' : undefined} onclick={() => go(item.id)}>{item.title}</button>
			{/each}
		</nav>

		<div class="sections">
			{#each SECTIONS as s (s.id)}
				<section id={s.id}>
					{#if s.group !== s.title}<p class="caps where">{s.group}</p>{/if}
					<h2 class="h1">{s.title}</h2>
					<p class="intro">{s.intro}</p>
					{#if s.shot}
						<button class="shot" onclick={() => (zoomed = s)} aria-label={t('Увеличить снимок: {title}', { title: s.title })}>
							{@render picture(s)}
						</button>
					{/if}
					<dl>
						{#each s.items as item (item.name)}
							<div class="row">
								<dt>
									<span class="head">
										{#if item.mark}<span class="dot inline" aria-hidden="true">{item.mark}</span>{/if}
										<span class="name">{item.name}</span>
									</span>
									{#if item.keys}
										<span class="keys">{#each item.keys as k (k)}<kbd class="kbd mono">{k}</kbd>{/each}</span>
									{/if}
								</dt>
								<dd>{item.text}</dd>
							</div>
						{/each}
					</dl>
				</section>
			{/each}

			<section id={KEYS_ID}>
				<h2 class="h1">{t('Горячие клавиши')}</h2>
				<p class="intro">{t('Клавиши инструментов работают, пока курсор не стоит в поле ввода.')}</p>
				<div class="keys-grid">
					{#each KEYS as block (block.where)}
						<div class="keys-block card">
							<h3 class="h3">{block.where}</h3>
							<ul>
								{#each block.rows as [keys, what] (what)}
									<li>
										<span class="keys">{#each keys as k (k)}<kbd class="kbd mono">{k}</kbd>{/each}</span>
										<span>{what}</span>
									</li>
								{/each}
							</ul>
						</div>
					{/each}
				</div>
			</section>
		</div>
	</div>
</div>

{#if zoomed}
	<div class="zoom" role="dialog" aria-modal="true" aria-label={zoomed.title}>
		<button class="veil" onclick={() => (zoomed = null)} aria-label={t('Закрыть')}></button>
		<figure>
			<div class="big">{@render picture(zoomed)}</div>
			<figcaption>{zoomed.title} <button class="close" onclick={() => (zoomed = null)}>{t('Закрыть')} <span class="kbd mono">Esc</span></button></figcaption>
		</figure>
	</div>
{/if}

<style>
	.layout {
		display: grid;
		grid-template-columns: 208px minmax(0, 1fr);
		gap: 40px;
		align-items: start;
	}

	.toc {
		position: sticky;
		top: 24px;
		display: grid;
		gap: 2px;
	}

	.toc .group {
		margin: 14px 0 4px 12px;
		color: var(--ink-300);
	}

	.toc button {
		padding: 7px 12px;
		border: 0;
		border-radius: var(--radius-s);
		background: transparent;
		font: inherit;
		font-size: 14px;
		font-weight: 600;
		text-align: left;
		color: var(--ink-700);
		cursor: pointer;
	}

	.toc button.sub {
		font-weight: 500;
	}

	.toc button:hover {
		background: var(--paper-sunken);
	}

	.toc button.on {
		background: var(--night);
		color: var(--night-text);
	}

	.sections {
		display: grid;
		gap: 64px;
		min-width: 0;
		max-width: 920px;
	}

	section {
		scroll-margin-top: 24px;
	}

	.where {
		margin-bottom: 6px;
		color: var(--ai-500);
	}

	.intro {
		max-width: 68ch;
		margin-top: 10px;
		font-size: 16px;
		line-height: 1.55;
		color: var(--ink-700);
	}

	/* The screenshot is a dark studio screen on paper: a thin dark frame holds it together. */
	.shot {
		position: relative;
		display: block;
		width: 100%;
		margin-top: 20px;
		padding: 0;
		border: 0;
		border-radius: var(--radius-m);
		background: var(--sumi-bg);
		box-shadow:
			0 0 0 1px var(--sumi-line),
			var(--shadow-card);
		overflow: hidden;
		cursor: zoom-in;
	}

	.shot img,
	.big img {
		display: block;
		width: 100%;
		height: auto;
	}

	.dot {
		position: absolute;
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: var(--shu-500);
		box-shadow: 0 0 0 2px #fff;
		font-family: var(--font-ui);
		font-size: 12px;
		font-weight: 700;
		line-height: 1;
		color: #fff;
		transform: translate(-50%, -50%);
	}

	.dot.inline {
		position: static;
		flex: none;
		box-shadow: none;
		transform: none;
	}

	dl {
		margin-top: 20px;
		border-top: 1px solid var(--paper-line);
	}

	.row {
		display: grid;
		grid-template-columns: 264px minmax(0, 1fr);
		gap: 24px;
		padding: 14px 0;
		border-bottom: 1px solid var(--paper-line);
	}

	dt {
		display: grid;
		align-content: start;
		gap: 8px;
		font-weight: 700;
		font-size: 15px;
		line-height: 1.45;
	}

	.head {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}

	.name {
		min-width: 0;
	}

	.keys {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	dd {
		margin: 0;
		max-width: 68ch;
		font-size: 15px;
		line-height: 1.55;
		color: var(--ink-700);
	}

	.keys-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 16px;
		margin-top: 20px;
		align-items: start;
	}

	.keys-block {
		padding: 20px 22px;
	}

	.keys-block ul {
		display: grid;
		gap: 10px;
		margin: 14px 0 0;
		padding: 0;
		list-style: none;
	}

	.keys-block li {
		display: grid;
		grid-template-columns: 150px minmax(0, 1fr);
		gap: 12px;
		align-items: center;
		font-size: 14px;
		line-height: 1.4;
		color: var(--ink-700);
	}

	.zoom {
		position: fixed;
		inset: 0;
		z-index: 80;
		display: grid;
		place-items: center;
		padding: 32px;
	}

	.veil {
		position: absolute;
		inset: 0;
		border: 0;
		background: rgb(6 7 11 / 82%);
		cursor: zoom-out;
	}

	.zoom figure {
		position: relative;
		margin: 0;
		width: min(100%, calc((100vh - 120px) * 1.5));
	}

	.big {
		position: relative;
		border-radius: var(--radius-m);
		box-shadow: 0 0 0 1px var(--sumi-line);
		overflow: hidden;
	}

	figcaption {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 12px;
		font-weight: 600;
		color: #fff;
	}

	.close {
		display: flex;
		align-items: center;
		gap: 8px;
		border: 0;
		background: transparent;
		font: inherit;
		font-size: 13px;
		color: #dce7f1;
		cursor: pointer;
	}

	@media (max-width: 1100px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}

		.toc {
			position: static;
			display: flex;
			flex-wrap: wrap;
		}

		.toc .group {
			display: none;
		}

		.row {
			grid-template-columns: minmax(0, 1fr);
			gap: 6px;
		}
	}
</style>
