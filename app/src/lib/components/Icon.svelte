<script lang="ts" module>
	// Line icons on a 24 px grid, 1.8 px stroke, round caps.
	export const ICONS = {
		home: 'M3.5 10.5 12 4l8.5 6.5V20a1 1 0 0 1-1 1h-5v-6h-5v6h-5a1 1 0 0 1-1-1z',
		book: 'M2.5 5.5c3-1.4 6.2-1.4 9.5.8 3.3-2.2 6.5-2.2 9.5-.8v13.5c-3-1.4-6.2-1.4-9.5.8-3.3-2.2-6.5-2.2-9.5-.8zM12 6.3v13.5',
		trophy: 'M7 4h10v5a5 5 0 0 1-10 0zM7 6H4v1.5A3.5 3.5 0 0 0 7.5 11M17 6h3v1.5A3.5 3.5 0 0 1 16.5 11M12 14v4M8 21h8M9.5 18h5',
		grid: 'M4 4h6.5v6.5H4zM13.5 4H20v6.5h-6.5zM4 13.5h6.5V20H4zM13.5 13.5H20V20h-6.5z',
		chart: 'M4 4v16h16M7.5 15l4-4.5 3 3L19 8',
		brush: 'M14.5 3.5 20.5 9.5l-7.5 7.5-6-6zM7 11l-1.5 1.5c-1.5 1.5-1 3-2 4.5-.5.8-1 1.5-1 2.5 1 0 1.7-.5 2.5-1 1.5-1 3-.5 4.5-2L11 15',
		search: 'M10.5 4a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zM20 20l-4.5-4.5',
		clock: 'M12 3.5a8.5 8.5 0 1 1 0 17 8.5 8.5 0 0 1 0-17zM12 7.5V12l3 2',
		lock: 'M6 11h12v9.5H6zM8.5 11V8a3.5 3.5 0 0 1 7 0v3',
		check: 'M5 12.5l4.2 4.2L19 7',
		bulb: 'M9.5 18h5M10.5 21h3M12 3a6 6 0 0 0-3.6 10.8c.6.5 1.1 1.3 1.1 2.2h5c0-.9.5-1.7 1.1-2.2A6 6 0 0 0 12 3z',
		close: 'M6 6l12 12M18 6 6 18',
		back: 'M15 5l-7 7 7 7',
		arrow: 'M5 12h14M13 6l6 6-6 6',
		play: 'M7 4.5 19.5 12 7 19.5z',
		stop: 'M6.5 6.5h11v11h-11z',
		upload: 'M12 15.5V4M7 8.5 12 3.5l5 5M4 20.5h16',
		download: 'M12 3.5V15M7 10l5 5 5-5M4 20.5h16',
		plus: 'M12 5v14M5 12h14',
		trash: 'M4 7h16M9.5 7V4.5h5V7M6 7l1 13.5h10L18 7M10 11v6M14 11v6',
		flame: 'M12 3c.8 3.8 5.5 5.6 5.5 10.5a5.5 5.5 0 0 1-11 0c0-2.8 1.6-4.1 2-6.5 1.7 1 2.8 2.8 2.8 4.6 1.6-1.9 1.3-5.6.7-8.6z',
		calendar: 'M4 5.5h16V20H4zM4 10h16M8.5 3v4.5M15.5 3v4.5',
		sparkle: 'M12 3.5l2 6.5 6.5 2-6.5 2-2 6.5-2-6.5-6.5-2 6.5-2z',
		eye: 'M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6z',
		pencil: 'M4 20l1-4.5L16 4.5l3.5 3.5L8.5 19zM14 7l3.5 3.5',
		sliders: 'M4 7h10M18 7h2M4 17h4M12 17h8M16 5v4M10 15v4',
		target: 'M12 3.5a8.5 8.5 0 1 1 0 17 8.5 8.5 0 0 1 0-17zM12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8zM12 11.3a.7.7 0 1 1 0 1.4.7.7 0 0 1 0-1.4z',
		video: 'M3 6.5h12.5v11H3zM15.5 10.5l5.5-3v9l-5.5-3z',
		copy: 'M8 8h11.5v11.5H8zM16 8V4.5H4.5V16H8',
		folder: 'M3 6.5a1 1 0 0 1 1-1h5l2 2.5h9a1 1 0 0 1 1 1V18.5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z',
		alert: 'M12 4 21 19.5H3zM12 10v4.5M12 17v.5',
		layers: 'M12 4 21 9l-9 5-9-5zM3 14l9 5 9-5',
		refresh: 'M19.5 12a7.5 7.5 0 1 1-2.2-5.3M19.5 4v4.5H15'
	} as const;
	export type IconName = keyof typeof ICONS;
</script>

<script lang="ts">
	let { name, size = 20, stroke = 1.8, label = '' }: { name: IconName; size?: number; stroke?: number; label?: string } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width={stroke}
	stroke-linecap="round"
	stroke-linejoin="round"
	role={label ? 'img' : undefined}
	aria-label={label || undefined}
	aria-hidden={label ? undefined : 'true'}
	style="flex: none"
>
	<path d={ICONS[name]} />
</svg>
