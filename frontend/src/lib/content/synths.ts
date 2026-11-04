import { SynthKind, type SynthKind as SynthKindType } from '$lib/state/synths.svelte';

export const synthColors: Record<SynthKindType, string> = {
    [SynthKind.Fluid]: 'var(--color-synth-fluid)',
    [SynthKind.Sfizz]: 'var(--color-synth-sfizz)',
    [SynthKind.Dummy]: 'var(--color-synth-dummy)',
    [SynthKind.Clap]: 'var(--color-synth-clap)'
};
