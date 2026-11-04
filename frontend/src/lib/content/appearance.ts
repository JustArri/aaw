import type { Accent, ThemeMode } from '$lib/state/theme.svelte';

export const accentPresets: { label: string; value: Accent; color: string }[] = [
    { label: 'Blue', value: { r: 96, g: 199, b: 255 }, color: 'var(--color-synth-fluid)' },
    { label: 'Violet', value: { r: 166, g: 150, b: 255 }, color: 'var(--color-synth-sfizz)' },
    { label: 'Green', value: { r: 80, g: 212, b: 160 }, color: 'var(--color-synth-dummy)' },
    { label: 'Orange', value: { r: 255, g: 173, b: 114 }, color: 'var(--color-synth-warm)' },
    { label: 'Rose', value: { r: 255, g: 112, b: 136 }, color: 'var(--color-preset-rose)' }
];

export const themeModes: { value: ThemeMode; label: string }[] = [
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
    { value: 'auto', label: 'Auto' }
];
