<script lang="ts">
    import type { Accent, SettingsSection } from '$lib/sections/settings.svelte';

    interface Props {
        section: SettingsSection;
    }

    let { section }: Props = $props();
</script>

<div class="p-3.75 min-[56.25rem]:p-5">
    <div>
        <span class="block text-sm font-semibold min-[56.25rem]:text-sm">Accent color</span>
        <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
            Choose any RGB color
        </small>
    </div>
    <label
        class="border-border-strong bg-surface-inset mt-3 flex w-full max-w-62.5 items-center gap-3 rounded-lg border px-2.5 py-2 font-mono text-sm">
        <input
            class="h-6.75 w-9.5 border-0 bg-transparent p-0"
            type="color"
            value={section.accentHex}
            oninput={(event) => section.setAccentHex(event.currentTarget.value)} />
        <span>{section.accentHex.toUpperCase()}</span>
    </label>
    <div class="mt-3 grid max-w-95 grid-cols-3 gap-2">
        {#each ['r', 'g', 'b'] as channel}
            <label>
                <span class="text-foreground-muted mb-1 ml-0.5 block text-xs">
                    {channel.toUpperCase()}
                </span>
                <input
                    class="border-border-strong bg-surface-inset text-foreground min-h-9.5 w-full rounded-lg border p-2 text-sm"
                    type="number"
                    min="0"
                    max="255"
                    value={section.accent[channel as keyof Accent]}
                    onchange={(event) =>
                        section.selectAccent({
                            ...section.accent,
                            [channel]: Number(event.currentTarget.value)
                        })} />
            </label>
        {/each}
    </div>
    <div class="mt-4 flex max-w-70 justify-between" aria-label="Accent color presets">
        {#each section.accentPresets as preset}
            <button
                class="border-contrast/15 hover:border-contrast/40 size-8 cursor-pointer rounded-full border-2 transition-transform hover:scale-110"
                type="button"
                style:background={preset.color}
                aria-label={`Use ${preset.label} accent`}
                onclick={() => section.selectAccent(preset.value)}>
            </button>
        {/each}
    </div>
</div>
