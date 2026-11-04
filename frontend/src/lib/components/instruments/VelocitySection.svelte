<script lang="ts">
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import BypassToggle from '$lib/components/BypassToggle.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import {
        type InstrumentsSection,
        submitControlGroupOnEnter
    } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-5 border-b p-4"
    use:submitControlGroupOnEnter
    onsubmit={(event) => {
        event.preventDefault();
        void section.velocitySection.save();
    }}>
    <div class="flex items-center justify-between gap-3">
        <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">Velocity</h3>
        <BypassToggle
            bind:checked={section.velocitySection.ignored}
            saving={section.savingSections.has('ignore-velocity-transform')}
            onchange={() => void section.velocitySection.saveIgnored()} />
    </div>
    <div class="grid gap-2">
        <div class="grid min-w-0 gap-4">
            <SegmentedControl
                name="velocity-transform"
                ariaLabel="Velocity transform"
                compact
                bind:value={section.velocitySection.type}
                options={[
                    { value: 'Unit', label: 'Unchanged' },
                    { value: 'Linear', label: 'Linear range' }
                ]} />
            {#if section.velocitySection.type === 'Linear'}
                <div class="grid grid-cols-[3rem_minmax(5rem,1fr)_3rem] items-center gap-2">
                    <input
                        class="border-border-strong bg-surface-inset h-9 min-w-0 appearance-none rounded-lg border px-1 text-center font-mono text-sm [&::-webkit-inner-spin-button]:appearance-none"
                        type="number"
                        aria-label="Minimum velocity"
                        min="0"
                        max={section.velocitySection.max}
                        bind:value={section.velocitySection.min} />
                    <div class="relative h-9 min-w-0">
                        <div
                            class="bg-surface-button absolute top-1/2 right-0 left-0 h-1 -translate-y-1/2 rounded-full">
                        </div>
                        <div
                            class="bg-accent absolute top-1/2 h-1 -translate-y-1/2 rounded-full"
                            style:left={`${(section.velocitySection.min / 127) * 100}%`}
                            style:right={`${100 - (section.velocitySection.max / 127) * 100}%`}>
                        </div>
                        <input
                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:-mt-1.5 [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
                            type="range"
                            aria-label="Minimum velocity"
                            min="0"
                            max="127"
                            value={section.velocitySection.min}
                            oninput={(event) => {
                                const value = Math.min(
                                    Number(event.currentTarget.value),
                                    section.velocitySection.max
                                );
                                section.velocitySection.min = value;
                                event.currentTarget.value = String(value);
                            }} />
                        <input
                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:-mt-1.5 [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
                            type="range"
                            aria-label="Maximum velocity"
                            min="0"
                            max="127"
                            value={section.velocitySection.max}
                            oninput={(event) => {
                                const value = Math.max(
                                    Number(event.currentTarget.value),
                                    section.velocitySection.min
                                );
                                section.velocitySection.max = value;
                                event.currentTarget.value = String(value);
                            }} />
                    </div>
                    <input
                        class="border-border-strong bg-surface-inset h-9 min-w-0 appearance-none rounded-lg border px-1 text-center font-mono text-sm [&::-webkit-inner-spin-button]:appearance-none"
                        type="number"
                        aria-label="Maximum velocity"
                        min={section.velocitySection.min}
                        max="127"
                        bind:value={section.velocitySection.max} />
                </div>
            {/if}
        </div>
        <ApplyButton
            saving={section.savingSections.has('velocity')}
            pending={section.velocitySection.changed} />
    </div>
</form>
