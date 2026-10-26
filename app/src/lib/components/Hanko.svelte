<script lang="ts">
	// A hanko seal: a square with a kanji. 筆 is the logo, 済 «passed», 完 «done», 再 «again».
	let {
		kanji,
		size = 40,
		tone = 'shu',
		tilt = -3,
		stamp = false
	}: { kanji: string; size?: number; tone?: 'shu' | 'ink' | 'ai' | 'faded'; tilt?: number; stamp?: boolean } = $props();
</script>

<span class="hanko {tone}" class:stamp style="--s: {size}px; --tilt: {tilt}deg" aria-hidden="true">
	<span class="kanji">{kanji}</span>
</span>

<style>
	.hanko {
		position: relative;
		display: inline-grid;
		place-items: center;
		flex: none;
		width: var(--s);
		height: var(--s);
		border-radius: calc(var(--s) * 0.14);
		background: var(--shu-500);
		color: #fbf8f2;
		transform: rotate(var(--tilt));
		box-shadow: 0 1px 0 rgb(0 0 0 / 8%);
	}

	.hanko::after {
		content: '';
		position: absolute;
		inset: calc(var(--s) * 0.09);
		border: max(1.5px, calc(var(--s) * 0.045)) solid currentColor;
		border-radius: calc(var(--s) * 0.08);
		opacity: 0.92;
	}

	.kanji {
		font-family: var(--font-kanji);
		font-size: calc(var(--s) * 0.56);
		line-height: 1;
		transform: translateY(-2%);
	}

	.ink {
		background: var(--ink-900);
	}

	.ai {
		background: var(--ai-500);
	}

	.faded {
		background: var(--shu-100);
		color: var(--shu-500);
	}

	.stamp {
		animation: stamp 520ms cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}

	@keyframes stamp {
		0% {
			opacity: 0;
			transform: rotate(calc(var(--tilt) - 18deg)) scale(1.9);
		}
		60% {
			opacity: 1;
			transform: rotate(calc(var(--tilt) + 2deg)) scale(0.94);
		}
	}
</style>
