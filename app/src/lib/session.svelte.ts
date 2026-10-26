// App-wide state shared by the shell and the pages.
import { api, type Boot } from '#lib/api.ts';

export const session = $state<{ boot: Boot | null; error: string | null }>({ boot: null, error: null });

/** Reloads the profile, rank and streak shown in the sidebar. */
export async function refreshSession(): Promise<void> {
	try {
		session.boot = await api.boot();
		session.error = null;
	} catch (e) {
		session.error = String(e);
	}
}
