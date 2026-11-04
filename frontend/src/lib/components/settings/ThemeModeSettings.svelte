<script lang="ts">
    import type { SettingsSection } from '$lib/sections/settings.svelte';

    interface Props {
        section: SettingsSection;
    }

    let { section }: Props = $props();
</script>

<div class="border-border border-b p-3.75 min-[56.25rem]:p-5">
    <div>
        <span class="block text-sm font-semibold min-[56.25rem]:text-sm">Color mode</span>
        <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
            Use a light theme, a dark theme, or follow this device
        </small>
    </div>
    <div class="mt-3 grid max-w-95 grid-cols-3 gap-2" aria-label="Color mode">
        {#each section.themeModes as mode}
            <button
                class={`min-h-11 cursor-pointer rounded-lg border px-3 text-sm font-semibold transition-colors ${section.themeMode === mode.value ? 'border-accent/50 bg-accent/10 text-accent' : 'border-border-strong bg-surface-inset text-foreground-strong hover:bg-surface-hover'}`}
                type="button"
                aria-pressed={section.themeMode === mode.value}
                onclick={() => section.selectTheme(mode.value)}>
                {mode.label}
            </button>
        {/each}
    </div>
</div>
