<script lang="ts">
	import Icon from '../components/Icon.svelte';
	import { TOOLS } from '../checks.ts';
	import { GROUP_LABELS, OPTIONS, PANELS, search, type TargetGroup } from './targets.ts';
	import { targetLabel } from './timeline.ts';

	// «Куда нажать»: the author points at a place on a map of the PhotoCraft window, or finds it
	// by name. The student later sees a ring there.
	let { value = null, step = '', onpick, onclose }: { value?: string | null; step?: string; onpick: (target: string | null) => void; onclose: () => void } = $props();

	let query = $state('');
	let picked = $state<string | null>(null);
	$effect.pre(() => {
		picked = value;
	});
	const found = $derived(search(query));
	const groups = $derived(
		(Object.keys(GROUP_LABELS) as TargetGroup[]).map((g) => ({ g, items: found.filter((t) => t.group === g) })).filter((x) => x.items.length)
	);
	const toolbar = TOOLS.slice(0, 14);
	const isOn = (id: string) => picked === id;
	let dialog: HTMLDialogElement | undefined = $state();
	$effect(() => {
		dialog?.showModal();
	});
</script>

<dialog bind:this={dialog} class="map" aria-labelledby="map-title" onclose={onclose}>
	<header>
		<div>
			<h2 id="map-title">Куда нажать{step ? ` — ${step}` : ''}</h2>
			<p>Щёлкните по месту в окне PhotoCraft. Ученик увидит там кольцо «начните здесь».</p>
		</div>
		<button class="x" onclick={() => dialog?.close()} aria-label="Закрыть"><Icon name="close" size={16} /></button>
	</header>
	<div class="body">
		<div class="window" aria-label="Схема окна PhotoCraft">
			<div class="menubar">
				{#each ['Файл', 'Правка', 'Изображение', 'Слой', 'Выделение', 'Фильтр', 'Вид', 'Окно'] as m (m)}<span>{m}</span>{/each}
			</div>
			<div class="options">
				{#each OPTIONS.slice(0, 4) as o (o)}
					<button class:on={isOn(`option:${o}`)} onclick={() => (picked = `option:${o}`)}>{targetLabel(`option:${o}`)}</button>
				{/each}
			</div>
			<div class="work">
				<div class="tools">
					{#each toolbar as [id, label] (id)}
						<button class="tool" class:on={isOn(`tool:${id}`)} onclick={() => (picked = `tool:${id}`)} title={label} aria-label={label}>
							<span>{label.slice(0, 2)}</span>
						</button>
					{/each}
				</div>
				<div class="canvas"><div class="sheet"></div></div>
				<div class="panels">
					{#each PANELS.slice(0, 4) as p (p)}
						<button class="panel" class:on={isOn(`panel:${p}`)} onclick={() => (picked = `panel:${p}`)}>{targetLabel(`panel:${p}`)}</button>
					{/each}
				</div>
			</div>
		</div>
		<aside>
			<label class="search"><Icon name="search" size={14} /><input type="search" placeholder="Найти: кисть, слои, жёсткость" bind:value={query} /></label>
			<div class="list">
				{#each groups as { g, items } (g)}
					<p class="group">{GROUP_LABELS[g]}</p>
					{#each items as t (t.id)}
						<button class="item" class:on={isOn(t.id)} onclick={() => (picked = t.id)}>
							<span>{t.label}</span>
							<span class="mono id">{t.id}</span>
						</button>
					{/each}
				{:else}
					<p class="empty">Ничего не нашлось. Попробуйте «кисть» или «слои».</p>
				{/each}
			</div>
			<footer>
				{#if value}<button class="ghost-btn" onclick={() => onpick(null)}>Убрать</button>{/if}
				<span class="grow"></span>
				<button class="ghost-btn" onclick={() => dialog?.close()}>Отмена</button>
				<button class="primary" disabled={!picked} onclick={() => onpick(picked)}>Выбрать{picked ? ` «${targetLabel(picked)}»` : ''}</button>
			</footer>
		</aside>
	</div>
</dialog>

<style>
	.map {
		width: min(980px, 94vw);
		padding: 0;
		border: 1px solid var(--sumi-line);
		border-radius: 14px;
		background: var(--sumi-panel);
		color: var(--sumi-text);
		box-shadow: 0 24px 60px rgb(0 0 0 / 50%);
		font-family: var(--font-ui);
	}

	.map::backdrop {
		background: rgb(10 10 12 / 70%);
	}

	header {
		display: flex;
		gap: 16px;
		align-items: center;
		padding: 16px 16px 16px 20px;
		border-bottom: 1px solid var(--sumi-line);
	}

	h2 {
		margin: 0;
		font: 700 20px/1.3 var(--font-ui);
	}

	header p {
		margin: 2px 0 0;
		color: var(--sumi-muted);
		font-size: 13px;
	}

	.x {
		margin-left: auto;
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 8px;
		background: var(--sumi-raised);
		color: var(--sumi-text);
		cursor: pointer;
	}

	.body {
		display: grid;
		grid-template-columns: 1fr 300px;
		min-height: 460px;
	}

	.window {
		margin: 20px;
		display: flex;
		flex-direction: column;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: #131418;
		overflow: hidden;
	}

	.menubar,
	.options {
		display: flex;
		gap: 14px;
		padding: 6px 10px;
		background: #1a1b20;
		color: var(--sumi-muted);
		font-size: 12px;
	}

	.options {
		border-top: 1px solid var(--sumi-line);
	}

	.options button,
	.panel,
	.tool,
	.item {
		border: 1px solid transparent;
		background: none;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}

	.options button {
		padding: 2px 4px;
		border-bottom: 1px dashed var(--ai-300);
		border-radius: 0;
	}

	.work {
		flex: 1;
		display: grid;
		grid-template-columns: 44px 1fr 170px;
	}

	.tools {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 6px;
		background: #1a1b20;
	}

	.tool {
		height: 26px;
		border-radius: 6px;
		color: var(--sumi-muted);
		font-size: 10px;
	}

	.canvas {
		display: grid;
		place-items: center;
	}

	.sheet {
		width: 60%;
		aspect-ratio: 3 / 2;
		background: #e8e2d6;
	}

	.panels {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		background: #1a1b20;
	}

	.panel {
		flex: 1;
		padding: 8px;
		border: 1px dashed var(--sumi-line);
		border-radius: 6px;
		color: var(--sumi-muted);
		text-align: left;
		font-size: 12px;
	}

	.on {
		border: 1px solid var(--ai-300) !important;
		background: var(--ai-500) !important;
		color: #fff !important;
		box-shadow: 0 0 0 4px rgb(143 178 214 / 25%);
	}

	aside {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 16px;
		border-left: 1px solid var(--sumi-line);
	}

	.search {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
		color: var(--sumi-muted);
	}

	.search input {
		flex: 1;
		border: 0;
		background: none;
		color: var(--sumi-text);
		font: 13px var(--font-ui);
		outline: none;
	}

	.list {
		flex: 1;
		overflow-y: auto;
		max-height: 340px;
	}

	.group {
		margin: 10px 0 4px;
		color: var(--sumi-muted);
		font: 700 11px var(--font-ui);
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.item {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		width: 100%;
		padding: 7px 8px;
		border-radius: 6px;
		text-align: left;
		font-size: 13px;
	}

	.item:hover {
		background: var(--sumi-raised);
	}

	.id {
		color: var(--sumi-muted);
		font-size: 11px;
	}

	.item.on .id {
		color: var(--ai-100);
	}

	.empty {
		color: var(--sumi-muted);
		font-size: 13px;
	}

	footer {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.grow {
		flex: 1;
	}

	.ghost-btn,
	.primary {
		padding: 8px 14px;
		border-radius: 8px;
		border: 0;
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.ghost-btn {
		background: var(--sumi-raised);
		color: var(--sumi-text);
	}

	.primary {
		background: var(--shu-500);
		color: #fff;
	}

	.primary:disabled {
		opacity: 0.5;
		cursor: default;
	}
</style>
