<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import NumberInput from '$lib/components/NumberInput.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { latencyModes, sampleRateModes } from '$lib/content/audio';
    import { AudioSection } from '$lib/sections/audio.svelte';

    const section = new AudioSection();
</script>

<div class="grid gap-3 min-[56.25rem]:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]">
    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Output configuration
                </h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                    Select the audio device and stream parameters.
                </p>
            </div>
            <Button
                variant="secondary"
                size="md"
                square
                class="bg-surface-card text-foreground-muted"
                type="button"
                aria-label="Refresh audio outputs"
                busy={section.refreshingOutputs}
                disabled={section.refreshingOutputs}
                onclick={() => void section.refreshOutputs()}>
                <Icon
                    icon={section.refreshingOutputs
                        ? 'lucide:loader-circle'
                        : section.outputsRefreshed
                          ? 'lucide:check'
                          : 'lucide:refresh-cw'}
                    width="1rem"
                    class={section.refreshingOutputs
                        ? 'animate-spin'
                        : section.outputsRefreshed
                          ? 'text-success'
                          : ''} />
            </Button>
        </header>

        <form
            class="grid gap-4 p-3.75 min-[37.5rem]:grid-cols-2 min-[56.25rem]:p-5"
            onsubmit={(event) => {
                event.preventDefault();
                void section.applyOutput();
            }}>
            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Audio host</span>
                <SelectDropdown
                    id="audio-host"
                    bind:value={section.selectedHostId}
                    options={section.hosts.map((entry) => ({
                        value: entry.host.id,
                        label: entry.host.name
                    }))}
                    disabled={section.hosts.length === 0}
                    ariaLabel="Audio host" />
            </label>

            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Output device</span>
                <SelectDropdown
                    id="audio-output-device"
                    bind:value={section.selectedOutputId}
                    options={(section.selectedHost?.outputs ?? []).map((output) => ({
                        value: output.id,
                        label: output.name,
                        description: output.is_default ? 'Default output' : undefined
                    }))}
                    disabled={!section.selectedHost || section.selectedHost.outputs.length === 0}
                    ariaLabel="Output device" />
            </label>

            <fieldset class="m-0 grid min-w-0 gap-3 border-0 p-0 min-[37.5rem]:col-span-2">
                <legend class="text-foreground mb-1 text-sm font-semibold">Latency</legend>
                <div
                    class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border min-[37.5rem]:grid-cols-4"
                    role="radiogroup"
                    aria-label="Latency">
                    {#each latencyModes as mode, index}
                        <label
                            class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:-outline-offset-2 ${index % 2 === 1 ? 'border-border-strong border-l' : ''} ${index >= 2 ? 'border-border-strong border-t min-[37.5rem]:border-t-0' : ''} ${index === 2 ? 'min-[37.5rem]:border-l' : ''} ${section.latencyMode === mode.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-latency"
                                value={mode.value}
                                bind:group={section.latencyMode} />
                            <span>{mode.label}</span>
                        </label>
                    {/each}
                </div>
                <label class="grid gap-1.5">
                    <span class="text-sm font-semibold">Custom buffer size</span>
                    <span class="flex items-center gap-2">
                        <NumberInput
                            class="w-28 text-right"
                            min={1}
                            step={1}
                            disabled={section.latencyMode !== 'CustomBufferSize'}
                            bind:value={section.customBufferSize} />
                        <small class="text-foreground-muted text-sm">frames</small>
                    </span>
                </label>
            </fieldset>

            <fieldset class="m-0 grid min-w-0 gap-3 border-0 p-0 min-[37.5rem]:col-span-2">
                <legend class="text-foreground mb-1 text-sm font-semibold">Sample rate</legend>
                <div
                    class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border"
                    role="radiogroup"
                    aria-label="Sample rate">
                    {#each sampleRateModes as mode, index}
                        <label
                            class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:-outline-offset-2 ${index > 0 ? 'border-border-strong border-l' : ''} ${section.sampleRateMode === mode.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-sample-rate"
                                value={mode.value}
                                bind:group={section.sampleRateMode} />
                            <span>{mode.label}</span>
                        </label>
                    {/each}
                </div>
                <label class="grid gap-1.5">
                    <span class="text-sm font-semibold">Fixed sample rate</span>
                    <span class="flex items-center gap-2">
                        <NumberInput
                            class="w-32 text-right"
                            min={1}
                            step={1}
                            disabled={section.sampleRateMode !== 'Fixed'}
                            bind:value={section.sampleRate} />
                        <small class="text-foreground-muted text-sm">Hz</small>
                    </span>
                </label>
            </fieldset>

            <Button
                variant="primary"
                size="lg"
                class="min-[37.5rem]:col-span-2"
                type="submit"
                busy={section.applying}
                disabled={!section.selectedOutput || section.applying}>
                {#if section.applying}<Icon
                        icon="lucide:loader-circle"
                        width="0.875rem"
                        class="animate-spin" />{/if}
                {section.applying
                    ? 'Applying…'
                    : section.stream
                      ? 'Apply and restart output'
                      : 'Start output'}
            </Button>
        </form>
    </section>

    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border min-h-16.75 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div class="flex items-center gap-2">
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Active stream
                </h2>
                <span
                    class={`size-1.75 rounded-full ${section.stream ? 'bg-success' : 'bg-indicator-off'}`}>
                </span>
            </div>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                {section.stream ? 'Audio rendering is active.' : 'No output stream is running.'}
            </p>
        </header>

        {#if section.stream}
            <dl class="divide-border m-0 divide-y px-3.75 min-[56.25rem]:px-5">
                <div class="flex items-start justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Device</dt>
                    <dd class="m-0 text-right text-sm font-semibold">
                        {section.activeOutput?.name ?? section.stream.output_id}
                        <small class="text-foreground-muted mt-0.5 block font-normal">
                            {section.activeHost?.host.name ?? section.stream.host_id}
                        </small>
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Sample rate</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {section.stream.sample_rate.toLocaleString()} Hz
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Buffer</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {section.stream.buffer_size === 0
                            ? 'Default size unavailable'
                            : `${section.stream.buffer_size} frames`}
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Channels</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {section.stream.channel_count}
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Sample format</dt>
                    <dd class="m-0 font-mono text-sm">{section.stream.sample_format}</dd>
                </div>
            </dl>
        {:else}
            <div
                class="text-foreground-muted grid min-h-52 place-items-center px-6 text-center text-sm">
                Select an output configuration to start audio.
            </div>
        {/if}
    </section>
</div>
