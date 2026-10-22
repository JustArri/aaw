<script lang="ts">
    import Icon from '@iconify/svelte';
    import { notify } from '$lib/push-notifications.svelte';
    import type { OutputStreamLatency, OutputStreamSampleRate } from '$lib/generated/models';
    import {
        audioHostOutputs,
        audioOutputStream,
        fetchAudioHostOutputs,
        setAudioOutputStream
    } from '$lib/state/audio.svelte';

    type LatencyMode = 'Automatic' | 'Low' | 'Stable' | 'CustomBufferSize';
    type SampleRateMode = 'Automatic' | 'Fixed';

    const latencyModes: { value: LatencyMode; label: string }[] = [
        { value: 'Automatic', label: 'Automatic' },
        { value: 'Low', label: 'Low latency' },
        { value: 'Stable', label: 'Stable' },
        { value: 'CustomBufferSize', label: 'Custom' }
    ];

    const sampleRateModes: { value: SampleRateMode; label: string }[] = [
        { value: 'Automatic', label: 'Automatic' },
        { value: 'Fixed', label: 'Fixed' }
    ];

    let selectedHostId = $state('');
    let selectedOutputId = $state('');
    let latencyMode = $state<LatencyMode>('Automatic');
    let customBufferSize = $state(256);
    let sampleRateMode = $state<SampleRateMode>('Automatic');
    let sampleRate = $state(48_000);
    let applying = $state(false);
    let refreshingOutputs = $state(false);
    let outputsRefreshed = $state(false);

    let hosts = $derived(audioHostOutputs());
    let stream = $derived(audioOutputStream());
    let selectedHost = $derived(hosts.find((entry) => entry.host.id === selectedHostId));
    let selectedOutput = $derived(
        selectedHost?.outputs.find((output) => output.id === selectedOutputId)
    );
    let activeHost = $derived(hosts.find((entry) => entry.host.id === stream?.host_id));
    let activeOutput = $derived(
        activeHost?.outputs.find((output) => output.id === stream?.output_id)
    );

    $effect(() => {
        if (hosts.length === 0) {
            selectedHostId = '';
            selectedOutputId = '';
            return;
        }

        if (!hosts.some((entry) => entry.host.id === selectedHostId)) {
            const streamHost = hosts.find((entry) => entry.host.id === stream?.host_id);
            selectedHostId = streamHost?.host.id ?? hosts[0].host.id;
        }
    });

    $effect(() => {
        const outputs = selectedHost?.outputs ?? [];

        if (outputs.length === 0) {
            selectedOutputId = '';
            return;
        }

        if (!outputs.some((output) => output.id === selectedOutputId)) {
            const streamOutput = outputs.find((output) => output.id === stream?.output_id);
            const defaultOutput = outputs.find((output) => output.is_default);
            selectedOutputId = streamOutput?.id ?? defaultOutput?.id ?? outputs[0].id;
        }
    });

    function selectedLatency(): OutputStreamLatency {
        if (latencyMode === 'CustomBufferSize') {
            return {
                type: 'CustomBufferSize',
                data: { value: Math.max(1, Math.round(customBufferSize)) }
            };
        }

        return { type: latencyMode };
    }

    function selectedSampleRate(): OutputStreamSampleRate {
        if (sampleRateMode === 'Automatic') {
            return { type: 'Automatic' };
        }

        return {
            type: 'Fixed',
            data: { value: Math.max(1, Math.round(sampleRate)) }
        };
    }

    async function applyOutput() {
        if (!selectedHostId || !selectedOutputId || applying) {
            return;
        }

        applying = true;
        const notification = notify.info('Starting the selected output…', {
            duration: null,
            loading: true,
            title: 'Apply audio output'
        });

        try {
            await setAudioOutputStream(
                selectedHostId,
                selectedLatency(),
                selectedOutputId,
                selectedSampleRate()
            );
            notification.success('Audio output applied.');
        } catch (error) {
            notification.error(
                error instanceof Error ? error.message : 'Could not apply the audio output.'
            );
        } finally {
            applying = false;
        }
    }

    async function refreshOutputs() {
        if (refreshingOutputs) {
            return;
        }
        refreshingOutputs = true;
        outputsRefreshed = false;
        try {
            await fetchAudioHostOutputs();
            outputsRefreshed = true;
            setTimeout(() => {
                outputsRefreshed = false;
            }, 1200);
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not refresh audio outputs.',
                { title: 'Refresh audio outputs' }
            );
        } finally {
            refreshingOutputs = false;
        }
    }
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
            <button
                class="border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border"
                type="button"
                aria-label="Refresh audio outputs"
                disabled={refreshingOutputs}
                onclick={() => void refreshOutputs()}>
                <Icon
                    icon={refreshingOutputs
                        ? 'lucide:loader-circle'
                        : outputsRefreshed
                          ? 'lucide:check'
                          : 'lucide:refresh-cw'}
                    width="1rem"
                    class={refreshingOutputs
                        ? 'animate-spin'
                        : outputsRefreshed
                          ? 'text-success'
                          : ''} />
            </button>
        </header>

        <form
            class="grid gap-4 p-3.75 min-[37.5rem]:grid-cols-2 min-[56.25rem]:p-5"
            onsubmit={(event) => {
                event.preventDefault();
                void applyOutput();
            }}>
            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Audio host</span>
                <select
                    class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
                    bind:value={selectedHostId}
                    disabled={hosts.length === 0}>
                    {#each hosts as entry}
                        <option value={entry.host.id}>{entry.host.name}</option>
                    {/each}
                </select>
            </label>

            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Output device</span>
                <select
                    class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
                    bind:value={selectedOutputId}
                    disabled={!selectedHost || selectedHost.outputs.length === 0}>
                    {#each selectedHost?.outputs ?? [] as output}
                        <option value={output.id}>
                            {output.name}{output.is_default ? ' · Default' : ''}
                        </option>
                    {/each}
                </select>
            </label>

            <fieldset class="m-0 grid min-w-0 gap-3 border-0 p-0 min-[37.5rem]:col-span-2">
                <legend class="text-foreground mb-1 text-sm font-semibold">Latency</legend>
                <div class="grid grid-cols-2 gap-2 min-[37.5rem]:grid-cols-4">
                    {#each latencyModes as mode}
                        <label class="cursor-pointer">
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-latency"
                                value={mode.value}
                                bind:group={latencyMode} />
                            <span
                                class={`grid min-h-11 place-items-center rounded-lg border px-2 text-center text-sm font-semibold transition-colors ${latencyMode === mode.value ? 'border-accent/50 bg-accent/10 text-accent' : 'border-border-strong bg-surface-inset text-foreground-strong hover:bg-surface-hover'}`}>
                                {mode.label}
                            </span>
                        </label>
                    {/each}
                </div>
                <label class="grid gap-1.5">
                    <span class="text-sm font-semibold">Custom buffer size</span>
                    <span class="flex items-center gap-2">
                        <input
                            class="border-border-strong bg-surface-inset text-foreground h-10 w-28 rounded-lg border px-3 text-right font-mono text-sm disabled:cursor-not-allowed disabled:opacity-50"
                            type="number"
                            min="1"
                            step="1"
                            disabled={latencyMode !== 'CustomBufferSize'}
                            bind:value={customBufferSize} />
                        <small class="text-foreground-muted text-sm">frames</small>
                    </span>
                </label>
            </fieldset>

            <fieldset class="m-0 grid min-w-0 gap-3 border-0 p-0 min-[37.5rem]:col-span-2">
                <legend class="text-foreground mb-1 text-sm font-semibold">Sample rate</legend>
                <div class="grid grid-cols-2 gap-2">
                    {#each sampleRateModes as mode}
                        <label class="cursor-pointer">
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-sample-rate"
                                value={mode.value}
                                bind:group={sampleRateMode} />
                            <span
                                class={`grid min-h-11 place-items-center rounded-lg border px-2 text-center text-sm font-semibold transition-colors ${sampleRateMode === mode.value ? 'border-accent/50 bg-accent/10 text-accent' : 'border-border-strong bg-surface-inset text-foreground-strong hover:bg-surface-hover'}`}>
                                {mode.label}
                            </span>
                        </label>
                    {/each}
                </div>
                <label class="grid gap-1.5">
                    <span class="text-sm font-semibold">Fixed sample rate</span>
                    <span class="flex items-center gap-2">
                        <input
                            class="border-border-strong bg-surface-inset text-foreground h-10 w-32 rounded-lg border px-3 text-right font-mono text-sm disabled:cursor-not-allowed disabled:opacity-50"
                            type="number"
                            min="1"
                            step="1"
                            disabled={sampleRateMode !== 'Fixed'}
                            bind:value={sampleRate} />
                        <small class="text-foreground-muted text-sm">Hz</small>
                    </span>
                </label>
            </fieldset>

            <button
                class="bg-accent text-accent-foreground min-h-11 cursor-pointer rounded-lg border-0 px-4 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50 min-[37.5rem]:col-span-2"
                type="submit"
                disabled={!selectedOutput || applying}>
                {applying ? 'Applying…' : stream ? 'Apply and restart output' : 'Start output'}
            </button>
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
                    class={`size-1.75 rounded-full ${stream ? 'bg-success' : 'bg-indicator-off'}`}>
                </span>
            </div>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                {stream ? 'Audio rendering is active.' : 'No output stream is running.'}
            </p>
        </header>

        {#if stream}
            <dl class="divide-border m-0 divide-y px-3.75 min-[56.25rem]:px-5">
                <div class="flex items-start justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Device</dt>
                    <dd class="m-0 text-right text-sm font-semibold">
                        {activeOutput?.name ?? stream.output_id}
                        <small class="text-foreground-muted mt-0.5 block font-normal">
                            {activeHost?.host.name ?? stream.host_id}
                        </small>
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Sample rate</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {stream.sample_rate.toLocaleString()} Hz
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Buffer</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {stream.buffer_size === null
                            ? 'Backend default'
                            : `${stream.buffer_size} frames`}
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Channels</dt>
                    <dd class="m-0 font-mono text-sm tabular-nums">
                        {stream.channel_count}
                    </dd>
                </div>
                <div class="flex items-center justify-between gap-4 py-3">
                    <dt class="text-foreground-muted text-sm">Sample format</dt>
                    <dd class="m-0 font-mono text-sm">{stream.sample_format}</dd>
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
