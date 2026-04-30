import './styles/global.css';
import { mount } from 'svelte';
import Note from './lib/components/Note.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('#app root not found');

// The note window receives its ID via the URL query string (?id=...).
// The backend builds windows with this URL when creating or restoring notes.
const params = new URLSearchParams(window.location.search);
const noteId = params.get('id');
if (!noteId) {
  document.body.textContent = 'Note window opened without an id query parameter.';
  throw new Error('Missing ?id= on note window URL');
}

mount(Note, { target, props: { noteId } });
