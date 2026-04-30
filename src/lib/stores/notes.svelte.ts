import type { NoteConfig } from '../types';
import { tauri } from '../utils/tauri';

class NotesStore {
  notes = $state<NoteConfig[]>([]);
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

  async create() {
    const note = await tauri.createNote();
    this.notes = [note, ...this.notes];
    return note;
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
}

export const notesStore = new NotesStore();
