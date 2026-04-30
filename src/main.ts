import './styles/global.css';
import { mount } from 'svelte';
import Controller from './lib/components/Controller.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('#app root not found');

mount(Controller, { target });
