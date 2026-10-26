<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { api, errorText, type Accent, type ImportReport, type PackageInfo, type Problem, type Studio } from '#lib/api.ts';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { count } from '#lib/format.ts';
	import { refreshSession } from '#lib/session.svelte.ts';

	let studio = $state<Studio | null>(null);
	let problems = $state<Problem[]>([]);
	let message = $state('');
	let error = $state('');

	let newTitle = $state('');
	let newKanji = $state('');
	let newAccent = $state<Accent>('ai');
	let newDescription = $state('');

	let pending = $state<{ path: string; info: PackageInfo } | null>(null);
	let replace = $state(false);
	let report = $state<ImportReport | null>(null);
	let copyFrom = $state('');

	async function load() {
		try {
			studio = await api.studio();
			problems = (await api.problems()).filter((p) => studio && p.path.startsWith(studio.root));
		} catch (e) {
			error = errorText(e);
		}
	}

	onMount(load);

	async function run(action: () => Promise<unknown>, done?: string) {
		error = '';
		message = '';
		try {
			await action();
			if (done) message = done;
			await load();
		} catch (e) {
			error = errorText(e);
		}
	}

	async function createCategory(e: SubmitEvent) {
		e.preventDefault();
		await run(async () => {
			await api.categoryCreate(newTitle, newKanji.trim() || null, newAccent, newDescription);
			newTitle = newKanji = newDescription = '';
		}, 'Категория создана.');
	}

	async function createLesson(categoryDir: string) {
		error = '';
		try {
			const dir = await api.lessonCreate(categoryDir, 'Новый урок', 1);
			await refreshSession();
			goto(`/studio/edit?dir=${encodeURIComponent(dir)}`);
		} catch (e) {
			error = errorText(e);
		}
	}

	async function copy(categoryDir: string) {
		if (!copyFrom) return;
		error = '';
		try {
			const dir = await api.duplicate(copyFrom, categoryDir);
			goto(`/studio/edit?dir=${encodeURIComponent(dir)}`);
		} catch (e) {
			error = errorText(e);
		}
	}

	async function pickPackage() {
		error = '';
		report = null;
		const path = await open({ multiple: false, filters: [{ name: 'Пакет уроков FUDE', extensions: ['fude'] }] });
		if (typeof path !== 'string') return;
		try {
			pending = { path, info: await api.packageInspect(path) };
		} catch (e) {
			error = errorText(e);
		}
	}

	async function doImport() {
		if (!pending) return;
		const p = pending;
		await run(async () => {
			report = await api.packageImport(p.path, replace);
			pending = null;
		});
	}

	async function exportCategory(id: string, title: string) {
		const out = await save({ defaultPath: `${title}.fude`, filters: [{ name: 'Пакет уроков FUDE', extensions: ['fude'] }] });
		if (!out) return;
		await run(async () => {
			const n = await api.packageExport(id, [], out);
			message = `Сохранено: ${count(n, ['урок', 'урока', 'уроков'])} в ${out}`;
		});
	}

	async function removeDir(dir: string, what: string) {
		if (!confirm(`Удалить ${what}? Это нельзя отменить.`)) return;
		await run(() => api.remove(dir), 'Удалено.');
	}

	const TINT: Record<Accent, string> = { ai: 'var(--ai-100)', shu: 'var(--shu-100)', kaki: 'var(--kaki-100)', kin: 'var(--kin-100)' };
	const INK: Record<Accent, string> = { ai: 'var(--ai-500)', shu: 'var(--shu-500)', kaki: 'var(--kaki-500)', kin: 'var(--kin-500)' };
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">Мастерская</h1>
			<p>Свои категории и уроки, импорт и экспорт пакетов <span class="mono">.fude</span></p>
		</div>
		<button class="btn btn-primary" onclick={pickPackage}><Icon name="download" size={18} /> Импортировать урок</button>
	</header>

	{#if error}<div class="banner warn" role="alert"><Icon name="alert" size={18} /> <span>{error}</span></div>{/if}
	{#if message}<div class="banner ok" role="status"><Icon name="check" size={18} /> <span>{message}</span></div>{/if}

	{#if pending}
		<section class="card import reveal">
			<h2 class="h2">«{pending.info.manifest.title}»</h2>
			<p class="body-s muted">{pending.info.manifest.created_by ?? 'автор не указан'} · {count(pending.info.lessons.length, ['урок', 'урока', 'уроков'])}</p>
			<ul>
				{#each pending.info.lessons as l (l.id)}<li><span class="mono">{l.id}</span> — {l.title}</li>{/each}
			</ul>
			{#if pending.info.issues.some((i) => i.severity === 'error')}
				<div class="banner warn">В пакете есть ошибки — импорт невозможен.</div>
				<ul class="issues">
					{#each pending.info.issues.filter((i) => i.severity === 'error') as i (i.field + i.message)}<li>{i.field}: {i.message}</li>{/each}
				</ul>
			{/if}
			<label class="check"><input type="checkbox" bind:checked={replace} /> Заменить мои уроки с такими же id</label>
			<div class="row">
				<button class="btn btn-accent" onclick={doImport} disabled={pending.info.issues.some((i) => i.severity === 'error')}>Импортировать</button>
				<button class="btn btn-quiet" onclick={() => (pending = null)}>Отмена</button>
			</div>
			<p class="body-s muted">Гайды импортированных уроков могут только рисовать в редакторе: открывать и сохранять файлы им запрещено.</p>
		</section>
	{/if}

	{#if report}
		<section class="card import reveal">
			<h2 class="h2">Импортирован «{report.title}»</h2>
			<p>Новых уроков: {report.lessons_added.length}, заменено: {report.lessons_replaced.length}, новых категорий: {report.categories_added.length}.</p>
			<a href="/lessons">Открыть в каталоге →</a>
		</section>
	{/if}

	{#if studio}
		<div class="layout">
			<section class="list">
				{#each studio.categories as c (c.dir)}
					<article class="card cat">
						<header>
							<span class="glyph kanji" style="background: {TINT[c.category.accent]}; color: {INK[c.category.accent]}">{c.category.kanji ?? '筆'}</span>
							<div>
								<h2 class="h2">{c.category.title}</h2>
								<p class="body-s muted">{c.category.description || 'Без описания'} · id <span class="mono">{c.category.id}</span></p>
							</div>
							<div class="tools">
								<button class="btn btn-quiet btn-s" onclick={() => exportCategory(c.category.id, c.category.title)} disabled={!c.lessons.length}><Icon name="upload" size={16} /> Экспорт</button>
								<button class="icon-btn" onclick={() => removeDir(c.dir, `категорию «${c.category.title}» со всеми уроками`)} aria-label="Удалить категорию"><Icon name="trash" size={18} /></button>
							</div>
						</header>
						<ol>
							{#each c.lessons as l (l.dir)}
								<li>
									<a href="/studio/edit?dir={encodeURIComponent(l.dir)}">
										<span class="label">{l.title}</span>
										<span class="body-s muted">{l.id || 'не читается'}</span>
									</a>
									{#if l.broken}
										<span class="pill pill-kaki">файл повреждён</span>
									{:else if l.errors}
										<span class="pill pill-kaki">черновик · {count(l.errors, ['ошибка', 'ошибки', 'ошибок'])}</span>
									{:else}
										<span class="pill pill-ai">в каталоге</span>
									{/if}
									<button class="icon-btn" onclick={() => removeDir(l.dir, `урок «${l.title}»`)} aria-label="Удалить урок"><Icon name="trash" size={17} /></button>
								</li>
							{:else}
								<li class="muted body-s">Уроков пока нет.</li>
							{/each}
						</ol>
						<footer>
							<button class="btn btn-primary btn-s" onclick={() => createLesson(c.dir)}><Icon name="plus" size={16} /> Новый урок</button>
							<span class="copy">
								<select bind:value={copyFrom} aria-label="Встроенный урок для копии">
									<option value="">Копия встроенного урока…</option>
									{#each studio.builtin as [id, title] (id)}<option value={id}>{title}</option>{/each}
								</select>
								<button class="btn btn-quiet btn-s" onclick={() => copy(c.dir)} disabled={!copyFrom}><Icon name="copy" size={16} /> Скопировать</button>
							</span>
						</footer>
					</article>
				{:else}
					<article class="card empty">
						<Hanko kanji="創" size={56} />
						<h2 class="h1">Своих уроков пока нет</h2>
						<p class="muted">Создайте категорию справа, добавьте в неё урок или импортируйте готовый пакет <span class="mono">.fude</span>.</p>
					</article>
				{/each}

				{#if problems.length}
					<article class="card problems">
						<h2 class="h2">Не попали в каталог</h2>
						{#each problems as p (p.path)}
							<p class="body-s"><span class="mono">{p.path.replace(studio.root, '…')}</span> — {p.message}</p>
						{/each}
					</article>
				{/if}
			</section>

			<aside>
				<form class="card create" onsubmit={createCategory}>
					<h2 class="h2">Новая категория</h2>
					<label class="field">Название <input type="text" bind:value={newTitle} required maxlength="60" placeholder="Например, Акварель" /></label>
					<label class="field">Иероглиф <input type="text" bind:value={newKanji} maxlength="2" placeholder="水" /></label>
					<fieldset>
						<legend class="label">Цвет</legend>
						<div class="accents">
							{#each ['ai', 'shu', 'kaki', 'kin'] as const as a (a)}
								<button type="button" class="swatch" style="background: {INK[a]}" aria-pressed={newAccent === a} aria-label={{ ai: 'Индиго', shu: 'Киноварь', kaki: 'Хурма', kin: 'Золото' }[a]} onclick={() => (newAccent = a)}></button>
							{/each}
						</div>
					</fieldset>
					<label class="field">Описание <textarea rows="3" bind:value={newDescription}></textarea></label>
					<button class="btn btn-accent" disabled={!newTitle.trim()}>Создать категорию</button>
				</form>
				<p class="body-s muted where">Свои уроки хранятся в <span class="mono">{studio.root}</span></p>
			</aside>
		</div>
	{/if}
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 340px;
		gap: 24px;
		align-items: start;
	}

	.list {
		display: grid;
		gap: 20px;
	}

	.banner {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 18px;
		padding: 12px 16px;
		border-radius: 12px;
		font-size: 14px;
	}

	.warn {
		background: var(--kaki-100);
		color: var(--kaki-700);
	}

	.ok {
		background: var(--ai-100);
		color: var(--ai-700);
	}

	.import {
		display: grid;
		gap: 12px;
		margin-bottom: 20px;
		padding: 24px;
	}

	.import ul {
		margin: 0;
		padding-left: 18px;
	}

	.issues {
		color: var(--kaki-700);
		font-size: 13px;
	}

	.check {
		display: flex;
		gap: 8px;
		align-items: center;
		font-size: 14px;
	}

	.row {
		display: flex;
		gap: 10px;
	}

	.cat {
		padding: 22px;
	}

	.cat header {
		display: grid;
		grid-template-columns: 52px 1fr auto;
		align-items: center;
		gap: 16px;
	}

	.glyph {
		display: grid;
		place-items: center;
		width: 52px;
		height: 52px;
		border-radius: 12px;
		font-size: 30px;
	}

	.tools {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.icon-btn {
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--ink-500);
		cursor: pointer;
	}

	.icon-btn:hover {
		background: var(--paper-sunken);
		color: var(--shu-700);
	}

	.cat ol {
		display: grid;
		gap: 4px;
		margin: 16px 0;
		padding: 0;
		list-style: none;
	}

	.cat li {
		display: grid;
		grid-template-columns: 1fr auto auto;
		align-items: center;
		gap: 12px;
		padding: 8px 10px;
		border-radius: 10px;
	}

	.cat li:hover {
		background: rgb(22 22 26 / 3%);
	}

	.cat li a {
		display: grid;
		color: inherit;
	}

	footer {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 10px;
	}

	.copy {
		display: flex;
		gap: 8px;
	}

	.copy select {
		width: 240px;
		padding: 6px 10px;
	}

	.empty {
		display: grid;
		justify-items: center;
		gap: 12px;
		padding: 48px;
		text-align: center;
	}

	.problems {
		display: grid;
		gap: 6px;
		padding: 22px;
		color: var(--kaki-700);
	}

	.create {
		display: grid;
		gap: 14px;
		padding: 24px;
	}

	fieldset {
		margin: 0;
		padding: 0;
		border: 0;
	}

	.accents {
		display: flex;
		gap: 10px;
		margin-top: 8px;
	}

	.swatch {
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 50%;
		cursor: pointer;
	}

	.swatch[aria-pressed='true'] {
		outline: 3px solid var(--ink-900);
		outline-offset: 2px;
	}

	.where {
		margin-top: 12px;
		word-break: break-all;
	}
</style>
