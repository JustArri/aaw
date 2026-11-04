<script lang="ts">
    import type { SettingsSection } from '$lib/sections/settings.svelte';
    import AccentColorSettings from './AccentColorSettings.svelte';
    import ThemeModeSettings from './ThemeModeSettings.svelte';

    interface Props {
        section: SettingsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base min-[56.25rem]:text-lg">Appearance</h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                Interface preferences are stored on this device.
            </p>
        </div>
        <span
            class="border-contrast/15 size-6.75 shrink-0 rounded-full border-2 shadow-[0_0_1rem_var(--accent-glow-soft)]"
            style:background={section.accentHex}>
        </span>
    </header>
    <ThemeModeSettings {section} />
    <AccentColorSettings {section} />
</section>
