<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
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

        try {
            await setAudioOutputStream(
                selectedHostId,
                selectedLatency(),
                selectedOutputId,
                selectedSampleRate()
            );
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not apply the audio output.',
                { title: 'Apply audio output' }
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
            <Button
                variant="secondary"
                size="md"
                square
                class="bg-surface-card text-foreground-muted"
                type="button"
                aria-label="Refresh audio outputs"
                busy={refreshingOutputs}
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
            </Button>
        </header>

        <form
            class="grid gap-4 p-3.75 min-[37.5rem]:grid-cols-2 min-[56.25rem]:p-5"
            onsubmit={(event) => {
                event.preventDefault();
                void applyOutput();
            }}>
            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Audio host</span>
                <SelectDropdown
                    id="audio-host"
                    bind:value={selectedHostId}
                    options={hosts.map((entry) => ({
                        value: entry.host.id,
                        label: entry.host.name
                    }))}
                    disabled={hosts.length === 0}
                    ariaLabel="Audio host" />
            </label>

            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Output device</span>
                <SelectDropdown
                    id="audio-output-device"
                    bind:value={selectedOutputId}
                    options={(selectedHost?.outputs ?? []).map((output) => ({
                        value: output.id,
                        label: output.name,
                        description: output.is_default ? 'Default output' : undefined
                    }))}
                    disabled={!selectedHost || selectedHost.outputs.length === 0}
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
                            class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:-outline-offset-2 ${index % 2 === 1 ? 'border-border-strong border-l' : ''} ${index >= 2 ? 'border-border-strong border-t min-[37.5rem]:border-t-0' : ''} ${index === 2 ? 'min-[37.5rem]:border-l' : ''} ${latencyMode === mode.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-latency"
                                value={mode.value}
                                bind:group={latencyMode} />
                            <span>{mode.label}</span>
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
                <div
                    class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border"
                    role="radiogroup"
                    aria-label="Sample rate">
                    {#each sampleRateModes as mode, index}
                        <label
                            class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:-outline-offset-2 ${index > 0 ? 'border-border-strong border-l' : ''} ${sampleRateMode === mode.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                            <input
                                class="sr-only"
                                type="radio"
                                name="audio-sample-rate"
                                value={mode.value}
                                bind:group={sampleRateMode} />
                            <span>{mode.label}</span>
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

            <Button
                variant="primary"
                size="lg"
                class="min-[37.5rem]:col-span-2"
                type="submit"
                busy={applying}
                disabled={!selectedOutput || applying}>
                {#if applying}<Icon
                        icon="lucide:loader-circle"
                        width="0.875rem"
                        class="animate-spin" />{/if}
                {applying ? 'Applying…' : stream ? 'Apply and restart output' : 'Start output'}
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
                        {stream.buffer_size === 0
                            ? 'Default size unavailable'
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
