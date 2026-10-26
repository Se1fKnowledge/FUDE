<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Brush from './Brush.svelte';
	import Enso from './Enso.svelte';
	import Hanko from './Hanko.svelte';
	import Icon, { type IconName } from './Icon.svelte';
	import { session } from '#lib/session.svelte.ts';
	import { count } from '#lib/format.ts';

	const items: { href: string; label: string; icon: IconName; seed: number }[] = [
		{ href: '/', label: 'Главная', icon: 'home', seed: 11 },
		{ href: '/lessons', label: 'Уроки', icon: 'book', seed: 12 },
		{ href: '/achievements', label: 'Достижения', icon: 'trophy', seed: 13 },
		{ href: '/activity', label: 'Активность', icon: 'grid', seed: 14 },
		{ href: '/stats', label: 'Статистика', icon: 'chart', seed: 15 },
		{ href: '/studio', label: 'Мастерская', icon: 'pencil', seed: 16 }
	];

	const path = $derived(page.url.pathname);
	const active = (href: string) => (href === '/' ? path === '/' : path.startsWith(href));

	let query = $state('');
	let search: HTMLInputElement | undefined = $state();

	function onkeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
			e.preventDefault();
			search?.focus();
		}
	}

	function find(e: SubmitEvent) {
		e.preventDefault();
		goto(`/lessons?q=${encodeURIComponent(query.trim())}`);
	}

	const boot = $derived(session.boot);
	const initials = $derived(
		(boot?.profile.name ?? '')
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase())
			.join('') || '筆'
	);
	const goal = $derived(boot?.profile.dailyGoal ?? 2);
	const done = $derived(boot?.doneToday ?? 0);
</script>

<svelte:window {onkeydown} />

<aside>
	<div class="enso"><Enso size={420} width={30} color="#23262d" seed={21} /></div>

	<a class="brand" href="/" aria-label="FUDE, на главную">
		<Hanko kanji="筆" size={40} />
		<span>
			<span class="name">FUDE</span>
			<span class="sub">школа цифрового рисунка</span>
		</span>
	</a>

	<form class="search" onsubmit={find} role="search">
		<Icon name="search" size={17} />
		<input bind:this={search} bind:value={query} type="search" placeholder="Поиск уроков" aria-label="Поиск уроков" />
		<kbd>Ctrl K</kbd>
	</form>

	<nav aria-label="Разделы">
		{#each items as item (item.href)}
			<a href={item.href} class:active={active(item.href)} aria-current={active(item.href) ? 'page' : undefined}>
				{#if active(item.href)}
					<span class="stroke"><Brush width={232} height={46} seed={item.seed} draw /></span>
				{/if}
				<Icon name={item.icon} />
				<span>{item.label}</span>
			</a>
		{/each}
	</nav>

	<div class="spacer"></div>

	{#if boot}
		<div class="streak">
			<div class="row">
				<span class="flame"><Icon name="flame" size={18} /></span>
				<span>
					{#if boot.streak.current > 0}
						{count(boot.streak.current, ['день', 'дня', 'дней'])} подряд
					{:else}
						Серия ещё не началась
					{/if}
				</span>
			</div>
			<div class="bar"><i style="width: {Math.min(100, (done / goal) * 100)}%"></i></div>
			<p>Цель дня: {Math.min(done, goal)} из {count(goal, ['задания', 'заданий', 'заданий'])}</p>
		</div>

		<div class="me">
			<span class="avatar">{initials}</span>
			<span class="who">
				<span class="me-name">{boot.profile.name || 'Ученик'}</span>
				<span class="rank">{boot.rank.name} · {boot.rank.title}</span>
			</span>
		</div>
	{/if}
</aside>

<style>
	aside {
		position: sticky;
		top: 0;
		display: flex;
		flex-direction: column;
		width: var(--sidebar);
		height: 100vh;
		padding: 36px 20px 24px;
		background: var(--ink-900);
		color: var(--sumi-text);
		overflow: hidden;
	}

	.enso {
		position: absolute;
		left: -150px;
		top: 380px;
		opacity: 0.9;
		pointer-events: none;
	}

	.brand {
		position: relative;
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 0 10px;
		color: inherit;
	}

	.name {
		display: block;
		font-family: var(--font-display);
		font-weight: 600;
		font-size: 30px;
		letter-spacing: 0.12em;
		line-height: 1;
	}

	.sub {
		display: block;
		margin-top: 6px;
		font-size: 12px;
		color: var(--sumi-muted);
	}

	.search {
		position: relative;
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 30px 0 22px;
		padding: 0 10px 0 14px;
		height: 42px;
		border-radius: 10px;
		background: var(--sumi-panel);
		color: var(--sumi-muted);
	}

	.search input {
		flex: 1;
		min-width: 0;
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--sumi-text);
		font-size: 14px;
	}

	.search input:focus {
		outline: none;
	}

	.search:focus-within {
		outline: 2px solid var(--ai-500);
	}

	kbd {
		padding: 2px 6px;
		border-radius: 5px;
		background: var(--sumi-raised);
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--sumi-muted);
	}

	nav {
		position: relative;
		display: grid;
		gap: 4px;
	}

	nav a {
		position: relative;
		display: flex;
		align-items: center;
		gap: 14px;
		height: 46px;
		padding: 0 16px;
		border-radius: 10px;
		color: var(--sumi-text);
		font-weight: 600;
		font-size: 15px;
		transition: background 160ms;
	}

	nav a:hover:not(.active) {
		background: rgb(255 255 255 / 5%);
	}

	nav a.active {
		color: #fff;
	}

	nav a > :global(svg),
	nav a > span:not(.stroke) {
		position: relative;
	}

	.stroke {
		position: absolute;
		left: -4px;
		top: 0;
	}

	.spacer {
		flex: 1;
	}

	.streak {
		position: relative;
		padding: 18px 16px;
		border-radius: 14px;
		background: var(--sumi-panel);
		font-size: 14px;
	}

	.streak .row {
		display: flex;
		align-items: center;
		gap: 10px;
		font-weight: 600;
	}

	.flame {
		color: var(--kaki-500);
		display: inline-flex;
	}

	.bar {
		height: 5px;
		margin: 14px 0 10px;
		border-radius: 99px;
		background: var(--sumi-line);
		overflow: hidden;
	}

	.bar i {
		display: block;
		height: 100%;
		background: var(--kaki-500);
		border-radius: inherit;
		transition: width 600ms var(--ease-out);
	}

	.streak p {
		font-size: 13px;
		color: var(--sumi-muted);
	}

	.me {
		position: relative;
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 22px;
		padding: 0 4px;
	}

	.avatar {
		display: grid;
		place-items: center;
		width: 42px;
		height: 42px;
		border-radius: 50%;
		background: var(--ai-500);
		font-weight: 700;
		font-size: 14px;
	}

	.who {
		display: grid;
		min-width: 0;
	}

	.me-name {
		font-weight: 600;
		font-size: 14px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.rank {
		font-size: 13px;
		color: var(--kaki-500);
	}
</style>
