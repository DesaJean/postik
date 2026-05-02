import type { StackRecord } from '../types';
import { tauri } from '../utils/tauri';

class StacksStore {
  stacks = $state<StackRecord[]>([]);
  loading = $state(false);

  async load() {
    this.loading = true;
    try {
      this.stacks = await tauri.listStacks();
    } finally {
      this.loading = false;
    }
  }

  async create(name: string, color: string | null = null) {
    const stack = await tauri.createStack(name, color);
    this.stacks = [...this.stacks, stack];
    return stack;
  }

  async update(id: string, name: string, color: string | null = null) {
    await tauri.updateStack(id, name, color);
    this.stacks = this.stacks.map((s) => (s.id === id ? { ...s, name, color } : s));
  }

  async remove(id: string) {
    await tauri.deleteStack(id);
    this.stacks = this.stacks.filter((s) => s.id !== id);
  }

  byId(id: string | null | undefined): StackRecord | undefined {
    if (!id) return undefined;
    return this.stacks.find((s) => s.id === id);
  }
}

export const stacksStore = new StacksStore();
