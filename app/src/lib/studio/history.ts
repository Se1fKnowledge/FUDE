// Undo and redo for the studio: snapshots of the whole draft as JSON text. Quick edits in a
// row (typing, dragging) are one step, because the page records a snapshot only after the
// edits pause.

export class History {
	private past: string[] = [];
	private future: string[] = [];
	private current: string | null = null;

	constructor(private limit = 100) {}

	/** Records `state` as the latest version; the redo trail is dropped. */
	record(state: string): void {
		if (state === this.current) return;
		if (this.current !== null) {
			this.past.push(this.current);
			if (this.past.length > this.limit) this.past.shift();
		}
		this.current = state;
		this.future = [];
	}

	/** The version before the current one, which becomes current; `null` when there is none. */
	undo(): string | null {
		const prev = this.past.pop();
		if (prev === undefined || this.current === null) return null;
		this.future.push(this.current);
		this.current = prev;
		return prev;
	}

	redo(): string | null {
		const next = this.future.pop();
		if (next === undefined || this.current === null) return null;
		this.past.push(this.current);
		this.current = next;
		return next;
	}

	get canUndo(): boolean {
		return this.past.length > 0;
	}

	get canRedo(): boolean {
		return this.future.length > 0;
	}

	/** Starts over from `state` (after loading). */
	reset(state: string): void {
		this.past = [];
		this.future = [];
		this.current = state;
	}
}
