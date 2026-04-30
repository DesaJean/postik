import type { ColorId, TextColorId } from '../types';

export interface ColorDef {
  id: ColorId;
  fill: string;
  border: string;
  text: string;
}

export interface TextColorDef {
  id: TextColorId;
  label: string;
  /** Resolved hex; `null` means "use the palette's default text color". */
  value: string | null;
}

export const TEXT_COLORS: readonly TextColorDef[] = [
  { id: 'auto', label: 'Auto', value: null },
  { id: 'dark', label: 'Dark', value: '#1c1c1a' },
  { id: 'medium', label: 'Medium', value: '#5a5a55' },
  { id: 'light', label: 'Light', value: '#fafaf7' },
  { id: 'accent', label: 'Accent', value: '#d85a30' },
] as const;

export function resolveTextColor(textId: TextColorId | null, paletteText: string): string {
  if (!textId || textId === 'auto') return paletteText;
  return TEXT_COLORS.find((t) => t.id === textId)?.value ?? paletteText;
}

export const COLORS: readonly ColorDef[] = [
  { id: 'amber', fill: '#FAEEDA', border: '#BA7517', text: '#412402' },
  { id: 'teal', fill: '#E1F5EE', border: '#1D9E75', text: '#04342C' },
  { id: 'purple', fill: '#EEEDFE', border: '#534AB7', text: '#26215C' },
  { id: 'pink', fill: '#FBEAF0', border: '#D4537E', text: '#4B1528' },
  { id: 'blue', fill: '#E6F1FB', border: '#378ADD', text: '#042C53' },
  { id: 'gray', fill: '#F1EFE8', border: '#888780', text: '#2C2C2A' },
  { id: 'transparent', fill: 'transparent', border: '#888780', text: '#2C2C2A' },
] as const;

export function getColor(id: ColorId): ColorDef {
  return COLORS.find((c) => c.id === id) ?? COLORS[0];
}
