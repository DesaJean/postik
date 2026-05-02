import type { ColorId, NoteConfig } from '../types';
import { tauri } from '../utils/tauri';

export interface NoteTemplate {
  id: 'blank' | 'daily' | 'meeting' | 'todo';
  label: string;
  emoji: string;
  /** Resolved at create time so date/timestamp tokens like {date} expand. */
  build: () => { content: string; color: ColorId };
}

function todayStr(): string {
  const d = new Date();
  return d.toISOString().slice(0, 10);
}

export const NOTE_TEMPLATES: ReadonlyArray<NoteTemplate> = [
  {
    id: 'blank',
    label: 'Blank',
    emoji: '+',
    build: () => ({ content: '', color: 'amber' }),
  },
  {
    id: 'daily',
    label: 'Daily',
    emoji: '📅',
    build: () => ({
      content: `${todayStr()}\n\n- [ ] `,
      color: 'blue',
    }),
  },
  {
    id: 'meeting',
    label: 'Meeting',
    emoji: '👥',
    build: () => ({
      content: `Meeting — ${todayStr()}\n\nAttendees:\n- \n\nNotes:\n- `,
      color: 'purple',
    }),
  },
  {
    id: 'todo',
    label: 'Todo',
    emoji: '✓',
    build: () => ({
      content: `- [ ] \n- [ ] \n- [ ] `,
      color: 'amber',
    }),
  },
] as const;

class NotesStore {
  notes = $state<NoteConfig[]>([]);
  archived = $state<NoteConfig[]>([]);
  loading = $state(true);

  async load() {
    this.loading = true;
    try {
      const list = await tauri.listNotes();
      this.notes = list;
    } finally {
      this.loading = false;
    }
  }

  async loadArchived() {
    this.archived = await tauri.listArchivedNotes();
  }

  async archive(id: string) {
    await tauri.archiveNote(id);
    const moved = this.notes.find((n) => n.id === id);
    this.notes = this.notes.filter((n) => n.id !== id);
    if (moved) this.archived = [moved, ...this.archived];
  }

  async unarchive(id: string) {
    await tauri.unarchiveNote(id);
    const moved = this.archived.find((n) => n.id === id);
    this.archived = this.archived.filter((n) => n.id !== id);
    if (moved) this.notes = [moved, ...this.notes];
  }

  async deletePermanently(id: string) {
    await tauri.deleteNote(id);
    this.archived = this.archived.filter((n) => n.id !== id);
    this.notes = this.notes.filter((n) => n.id !== id);
  }

  async create() {
    const note = await tauri.createNote();
    this.notes = [note, ...this.notes];
    return note;
  }

  /** Create a note pre-filled from a template. The note is first created
   * via the standard `create_note` command (so position/window logic is
   * unchanged), then immediately patched with the template's content and
   * color. The returned note carries the post-template state. */
  async createFromTemplate(templateId: NoteTemplate['id']) {
    const tpl = NOTE_TEMPLATES.find((t) => t.id === templateId);
    if (!tpl || tpl.id === 'blank') return this.create();
    const { content, color } = tpl.build();
    const note = await tauri.createNote();
    if (content) await tauri.updateNoteContent(note.id, content);
    if (color !== note.color_id) await tauri.updateNoteColor(note.id, color);
    const patched: NoteConfig = { ...note, content, color_id: color };
    this.notes = [patched, ...this.notes];
    return patched;
  }

  async remove(id: string) {
    await tauri.deleteNote(id);
    this.notes = this.notes.filter((n) => n.id !== id);
  }

  upsert(note: NoteConfig) {
    const idx = this.notes.findIndex((n) => n.id === note.id);
    if (idx === -1) this.notes = [note, ...this.notes];
    else {
      this.notes[idx] = note;
      this.notes = [...this.notes];
    }
  }

  /** Move the note at fromIdx to beforeIdx, persist the new order, and
   * optimistically update the local list. The persistence call is
   * fire-and-log: a failure rolls the optimistic state back. */
  async reorder(fromIdx: number, toIdx: number) {
    if (fromIdx === toIdx) return;
    const next = [...this.notes];
    const [moved] = next.splice(fromIdx, 1);
    next.splice(toIdx, 0, moved);
    const previous = this.notes;
    this.notes = next;
    try {
      await tauri.reorderNotes(next.map((n) => n.id));
    } catch (e) {
      console.error('reorder_notes failed, rolling back:', e);
      this.notes = previous;
    }
  }
}

export const notesStore = new NotesStore();
