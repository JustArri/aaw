<script lang="ts">
    import Icon from '@iconify/svelte';
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import NumberInput from '$lib/components/NumberInput.svelte';
    import type { SystemSection } from '$lib/sections/system.svelte';

    interface Props {
        section: SystemSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border bg-surface-card overflow-hidden rounded-xl border"
    onsubmit={(event) => {
        event.preventDefault();
        void section.saveSynthDefaults();
    }}>
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base min-[56.25rem]:text-lg">Synth defaults</h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                Suggested parameters for new synths. They can be changed during creation.
            </p>
        </div>
        {#if section.voiceLimitsLoading}
            <Icon
                class="text-accent shrink-0 animate-spin"
                icon="lucide:loader-circle"
                width="1.125rem"
                aria-label="Loading synth defaults" />
        {:else}
            <ApplyButton
                saving={section.voiceLimitsSaving}
                pending={section.voiceLimitsChanged}
                disabled={!section.voiceLimitsLoaded}
                label="Apply synth defaults" />
        {/if}
    </header>

    <div
        class="grid gap-3 p-3.75 min-[43.75rem]:grid-cols-2 min-[56.25rem]:gap-4 min-[56.25rem]:p-5 min-[75rem]:grid-cols-4">
        <label
            class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
            <strong class="min-w-0 text-sm font-semibold">Dummy</strong>
            <NumberInput
                class="bg-surface-card w-20 shrink-0 px-2.5 text-right"
                min={1}
                max={65535}
                step={1}
                required
                disabled={!section.voiceLimitsLoaded}
                bind:value={section.synthDefaults.dummy_max_voices} />
        </label>

        <label
            class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
            <strong class="min-w-0 text-sm font-semibold">Fluid</strong>
            <NumberInput
                class="bg-surface-card w-20 shrink-0 px-2.5 text-right"
                min={1}
                max={65535}
                step={1}
                required
                disabled={!section.voiceLimitsLoaded}
                bind:value={section.synthDefaults.fluid_max_voices} />
        </label>

        <label
            class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
            <strong class="min-w-0 text-sm font-semibold">Sfizz</strong>
            <NumberInput
                class="bg-surface-card w-20 shrink-0 px-2.5 text-right"
                min={1}
                max={256}
                step={1}
                required
                disabled={!section.voiceLimitsLoaded}
                bind:value={section.synthDefaults.sfizz_max_voices} />
        </label>

        <label
            class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
            <span class="min-w-0">
                <strong class="block text-sm font-semibold">Sfizz preload</strong>
                <span class="text-foreground-muted text-xs">Frames per sample</span>
            </span>
            <NumberInput
                class="bg-surface-card w-25 shrink-0 px-2.5 text-right"
                min={1}
                max={4294967295}
                step={1}
                required
                disabled={!section.voiceLimitsLoaded}
                bind:value={section.synthDefaults.sfizz_preload_size} />
        </label>
    </div>
</form>
