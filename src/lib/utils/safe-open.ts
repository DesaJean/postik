import { confirm } from '@tauri-apps/plugin-dialog';
import { tauri } from './tauri';

/**
 * Opens `url` via the launcher, but if the distraction blocker is
 * active and the URL matches a blocked host, asks the user before
 * proceeding. The blocker only kicks in during a pomodoro work
 * session — outside of that this is a thin pass-through.
 */
export async function safeOpenUrl(url: string): Promise<void> {
  try {
    await tauri.openUrl(url);
  } catch (e) {
    const msg = String(e);
    if (msg.startsWith('blocked_during_focus:')) {
      const host = msg.slice('blocked_during_focus:'.length);
      const ok = await confirm(`${host} is blocked during your focus session. Open anyway?`, {
        title: 'Stay focused?',
        kind: 'warning',
      });
      if (ok) {
        await tauri.openUrlForce(url);
      }
      return;
    }
    throw e;
  }
}
