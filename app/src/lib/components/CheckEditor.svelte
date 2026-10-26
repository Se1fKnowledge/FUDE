<script lang="ts">
	import type { Check, CheckKind, Range } from '#lib/api.ts';
	import { CHECK_KINDS, TOOLS, defaultCheck } from '#lib/checks.ts';
	import Icon from './Icon.svelte';
	import RangeInput from './RangeInput.svelte';
	import RegionPicker from './RegionPicker.svelte';

	// One check of a step, with the fields its kind needs.
	let {
		check = $bindable(),
		image = null,
		aspect = 1.5,
		onremove
	}: { check: Check; image?: string | null; aspect?: number; onremove: () => void } = $props();

	function setKind(kind: CheckKind['kind']) {
		check = { ...defaultCheck(kind), ...(check.message ? { message: check.message } : {}) };
	}

	// Typed views of the current check for the bindings below.
	const c = $derived(check as Record<string, unknown>);
	const range = (key: string): Range => (c[key] as Range | undefined) ?? {};
	function setRange(key: string, r: Range) {
		check = { ...check, [key]: r } as Check;
	}
	function set(key: string, v: unknown) {
		const next = { ...check } as Record<string, unknown>;
		if (v === undefined || v === '') delete next[key];
		else next[key] = v;
		check = next as Check;
	}
</script>

<div class="check">
	<div class="top">
		<select value={check.kind} onchange={(e) => setKind(e.currentTarget.value as CheckKind['kind'])} aria-label="Вид проверки">
			<optgroup label="Состояние редактора">
				{#each CHECK_KINDS.filter((k) => !k.canvas) as k (k.kind)}<option value={k.kind}>{k.label}</option>{/each}
			</optgroup>
			<optgroup label="Снимок холста">
				{#each CHECK_KINDS.filter((k) => k.canvas) as k (k.kind)}<option value={k.kind}>{k.label}</option>{/each}
			</optgroup>
		</select>
		<button type="button" class="remove" onclick={onremove} aria-label="Удалить проверку"><Icon name="trash" size={16} /></button>
	</div>

	<div class="fields">
		{#if check.kind === 'tool'}
			<select value={check.tool} onchange={(e) => set('tool', e.currentTarget.value)} aria-label="Инструмент">
				{#each TOOLS as [id, name] (id)}<option value={id}>{name}</option>{/each}
			</select>
		{:else if check.kind === 'brush'}
			<RangeInput label="Размер" unit="px" bind:value={() => range('size'), (r) => setRange('size', r)} />
			<RangeInput label="Жёсткость" unit="%" scale={100} bind:value={() => range('hardness'), (r) => setRange('hardness', r)} />
			<RangeInput label="Непрозрачность" unit="%" scale={100} bind:value={() => range('opacity'), (r) => setRange('opacity', r)} />
		{:else if check.kind === 'zoom'}
			<RangeInput label="Масштаб" unit="%" scale={100} bind:value={() => range('zoom'), (r) => setRange('zoom', r)} />
		{:else if check.kind === 'layers'}
			<RangeInput label="Слоёв" bind:value={() => range('count'), (r) => setRange('count', r)} />
		{:else if check.kind === 'layer'}
			<label class="field">Имя содержит <input type="text" value={check.name ?? ''} onchange={(e) => set('name', e.currentTarget.value.trim())} /></label>
			<label class="field">
				Видимость
				<select value={check.visible === undefined ? '' : String(check.visible)} onchange={(e) => set('visible', e.currentTarget.value === '' ? undefined : e.currentTarget.value === 'true')}>
					<option value="">любая</option><option value="true">видим</option><option value="false">скрыт</option>
				</select>
			</label>
			<RangeInput label="Непрозрачность" unit="%" scale={100} bind:value={() => range('opacity'), (r) => setRange('opacity', r)} />
		{:else if check.kind === 'selection'}
			<label class="inline"><input type="checkbox" checked={check.present !== false} onchange={(e) => set('present', e.currentTarget.checked)} /> Выделение есть</label>
			<RangeInput label="Ширина" unit="% холста" scale={100} bind:value={() => range('width'), (r) => setRange('width', r)} />
			<RangeInput label="Высота" unit="% холста" scale={100} bind:value={() => range('height'), (r) => setRange('height', r)} />
		{:else if check.kind === 'history'}
			<label class="inline"><input type="checkbox" checked={check.can_redo === true} onchange={(e) => set('can_redo', e.currentTarget.checked || undefined)} /> Последнее действие отменено</label>
			<RangeInput label="Шагов истории" bind:value={() => range('entries'), (r) => setRange('entries', r)} />
		{:else if check.kind === 'foreground'}
			<label class="field">Цвет <input type="color" value={check.color} onchange={(e) => set('color', e.currentTarget.value.toUpperCase())} /></label>
		{:else if check.kind === 'coverage'}
			<RegionPicker bind:value={() => check.kind === 'coverage' ? check.region : [0, 0, 1, 1], (r) => set('region', r)} {image} {aspect} />
			<RangeInput label="Закрашено" unit="% области" scale={100} bind:value={() => range('changed'), (r) => setRange('changed', r)} />
		{:else if check.kind === 'luminance'}
			<RegionPicker bind:value={() => check.kind === 'luminance' ? check.region : [0, 0, 1, 1], (r) => set('region', r)} {image} {aspect} />
			<RangeInput label="Яркость" unit="%" scale={100} bind:value={() => range('value'), (r) => setRange('value', r)} />
		{:else if check.kind === 'darker'}
			<div class="two">
				<RegionPicker label="Тёмная" bind:value={() => check.kind === 'darker' ? check.region : [0, 0, 1, 1], (r) => set('region', r)} {image} {aspect} />
				<RegionPicker label="Светлее неё" color="var(--ai-500)" bind:value={() => check.kind === 'darker' ? check.than : [0, 0, 1, 1], (r) => set('than', r)} {image} {aspect} />
			</div>
			<label class="field">Темнее хотя бы на, % <input type="number" min="0" max="100" value={Math.round(((check.by ?? 0.05) * 100))} onchange={(e) => set('by', Number(e.currentTarget.value) / 100)} /></label>
		{:else if check.kind === 'color'}
			<RegionPicker bind:value={() => check.kind === 'color' ? check.region : [0, 0, 1, 1], (r) => set('region', r)} {image} {aspect} />
			<label class="field">Цвет <input type="color" value={check.color} onchange={(e) => set('color', e.currentTarget.value.toUpperCase())} /></label>
		{:else if check.kind === 'similar'}
			<RangeInput label="Похожесть" unit="%" scale={100} bind:value={() => range('score'), (r) => setRange('score', r)} />
			<p class="body-s muted">Сравнивается с эталоном задания.</p>
		{/if}
		<label class="field">Сообщение ученику, если не прошла <input type="text" value={check.message ?? ''} onchange={(e) => set('message', e.currentTarget.value.trim())} placeholder="Сообщение составится само" /></label>
	</div>
</div>

<style>
	.check {
		display: grid;
		gap: 12px;
		padding: 14px;
		border-radius: 12px;
		background: var(--paper-base);
		box-shadow: inset 0 0 0 1px var(--paper-line);
	}

	.top {
		display: flex;
		gap: 8px;
	}

	.remove {
		display: grid;
		place-items: center;
		width: 40px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--ink-500);
		cursor: pointer;
	}

	.remove:hover {
		background: var(--shu-100);
		color: var(--shu-700);
	}

	.fields {
		display: grid;
		gap: 10px;
	}

	.two {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.inline {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
	}
</style>
