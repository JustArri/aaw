<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { notify } from '$lib/push-notifications.svelte';
    import type {
        OutputStreamLatency,
        OutputStreamSampleRate,
        TimingMetricSnapshot,
        DeadlineMissMidiEventSnapshot
    } from '$lib/generated/models';
    import {
        audioHostOutputs,
        audioMetrics,
        audioOutputStream,
        fetchAudioMetrics,
        fetchAudioHostOutputs,
        resetAudioMetrics,
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
    let resettingMetrics = $state(false);
    let metricsErrorShown = false;

    let hosts = $derived(audioHostOutputs());
    let stream = $derived(audioOutputStream());
    let metrics = $derived(audioMetrics());
    let selectedHost = $derived(hosts.find((entry) => entry.host.id === selectedHostId));
    let selectedOutput = $derived(
        selectedHost?.outputs.find((output) => output.id === selectedOutputId)
    );
    let activeHost = $derived(hosts.find((entry) => entry.host.id === stream?.host_id));
    let activeOutput = $derived(
        activeHost?.outputs.find((output) => output.id === stream?.output_id)
    );

    function phaseRows(): { name: string; timing: TimingMetricSnapshot }[] {
        if (!metrics) {
            return [];
        }

        return [
            { name: 'Engine lock wait', timing: metrics.phases.lock_wait },
            { name: 'Commands', timing: metrics.phases.commands },
            { name: 'MIDI events', timing: metrics.phases.midi },
            { name: 'Instrument rendering', timing: metrics.phases.instruments },
            { name: 'Global effects and mixing', timing: metrics.phases.global_processing },
            { name: 'Output conversion', timing: metrics.phases.output_conversion }
        ];
    }

    function deadlineMisses() {
        return metrics ? [...metrics.deadline_misses].reverse() : [];
    }

    function describeMidiEvent(entry: DeadlineMissMidiEventSnapshot | null) {
        if (!entry) {
            return 'No MIDI event';
        }

        const event = entry.event;
        let detail: string;
        switch (event.type) {
            case 'NoteOn':
                detail = `Note on ${event.data.key} · velocity ${event.data.velocity}`;
                break;
            case 'NoteOff':
                detail = `Note off ${event.data.key} · velocity ${event.data.velocity}`;
                break;
            case 'ControlChange':
                detail = `CC ${event.data.controller} · value ${event.data.value}`;
                break;
            case 'PitchBend':
                detail = `Pitch bend ${event.data.value}`;
                break;
            case 'ProgramChange':
                detail = `Program ${event.data.program}`;
                break;
            case 'ChannelAftertouch':
                detail = `Channel pressure ${event.data.value}`;
                break;
            case 'PolyphonicAftertouch':
                detail = `Key pressure ${event.data.key} · value ${event.data.value}`;
                break;
        }
        return `Instrument #${entry.instrument_id + 1} · ch ${event.channel + 1} · ${detail}`;
    }

    onMount(() => {
        async function refreshMetrics() {
            try {
                await fetchAudioMetrics();
                metricsErrorShown = false;
            } catch (error) {
                if (!metricsErrorShown) {
                    metricsErrorShown = true;
                    notify.error(
                        error instanceof Error
                            ? error.message
                            : 'Could not load audio performance metrics.',
                        { title: 'Audio monitoring' }
                    );
                }
            }
        }

        void refreshMetrics();
        const timer = window.setInterval(() => void refreshMetrics(), 1000);
        return () => window.clearInterval(timer);
    });

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

    async function resetMetrics() {
        if (resettingMetrics) {
            return;
        }

        resettingMetrics = true;
        try {
            await resetAudioMetrics();
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not reset audio metrics.',
                { title: 'Reset audio monitoring' }
            );
        } finally {
            resettingMetrics = false;
        }
    }

    function percent(value: number | undefined) {
        return `${(value ?? 0).toFixed(1)}%`;
    }

    function duration(microseconds: number | undefined) {
        const value = microseconds ?? 0;
        return value >= 1000 ? `${(value / 1000).toFixed(2)} ms` : `${value.toFixed(0)} µs`;
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

    <section
        class="border-border bg-surface-card overflow-hidden rounded-xl border min-[56.25rem]:col-span-2">
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <div class="flex items-center gap-2">
                    <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                        Real-time performance
                    </h2>
                    <span
                        class={`size-1.75 rounded-full ${metrics?.active ? 'bg-success' : 'bg-indicator-off'}`}>
                    </span>
                </div>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Audio callback load and missed processing deadlines.
                </p>
            </div>
            <Button
                variant="secondary"
                size="sm"
                type="button"
                busy={resettingMetrics}
                disabled={resettingMetrics}
                onclick={() => void resetMetrics()}>
                <Icon
                    icon={resettingMetrics ? 'lucide:loader-circle' : 'lucide:rotate-ccw'}
                    width="0.9375rem"
                    class={resettingMetrics ? 'animate-spin' : ''} />
                Reset
            </Button>
        </header>

        <div
            class="bg-border grid grid-cols-2 gap-px min-[37.5rem]:grid-cols-4 min-[75rem]:grid-cols-6">
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Current load</div>
                <strong class="mt-1 block font-mono text-xl tabular-nums">
                    {percent(metrics?.last_load_percent)}
                </strong>
            </div>
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Average load</div>
                <strong class="mt-1 block font-mono text-xl tabular-nums">
                    {percent(metrics?.average_load_percent)}
                </strong>
            </div>
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Peak load</div>
                <strong
                    class={`mt-1 block font-mono text-xl tabular-nums ${(metrics?.peak_load_percent ?? 0) >= 100 ? 'text-danger' : ''}`}>
                    {percent(metrics?.peak_load_percent)}
                </strong>
            </div>
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Deadline misses</div>
                <strong
                    class={`mt-1 block font-mono text-xl tabular-nums ${(metrics?.deadline_miss_count ?? 0) > 0 ? 'text-danger' : ''}`}>
                    {metrics?.deadline_miss_count.toLocaleString() ?? '0'}
                </strong>
            </div>
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Last render</div>
                <strong class="mt-1 block font-mono text-xl tabular-nums">
                    {duration(metrics?.last_render_time_us)}
                </strong>
            </div>
            <div class="bg-surface-card p-3.75 min-[56.25rem]:p-5">
                <div class="text-foreground-muted text-xs uppercase">Longest render</div>
                <strong class="mt-1 block font-mono text-xl tabular-nums">
                    {duration(metrics?.max_render_time_us)}
                </strong>
            </div>
        </div>

        <dl
            class="divide-border m-0 grid divide-y px-3.75 min-[37.5rem]:grid-cols-2 min-[37.5rem]:divide-y-0 min-[56.25rem]:px-5">
            <div class="flex items-center justify-between gap-4 py-3 min-[37.5rem]:pr-5">
                <dt class="text-foreground-muted text-sm">Audio scheduling</dt>
                <dd
                    class={`m-0 text-sm ${metrics?.realtime_scheduling === 'Enabled' ? 'text-success' : metrics?.realtime_scheduling === 'Failed' ? 'text-danger' : 'text-foreground-muted'}`}>
                    {metrics?.realtime_scheduling === 'Enabled'
                        ? 'Real-time · RR 70'
                        : metrics?.realtime_scheduling === 'Failed'
                          ? 'Real-time request failed'
                          : metrics?.realtime_scheduling === 'Unsupported'
                            ? 'Unsupported'
                            : 'Not attempted'}
                </dd>
            </div>
            <div
                class="min-[37.5rem]:border-border flex items-center justify-between gap-4 py-3 min-[37.5rem]:border-l min-[37.5rem]:pl-5">
                <dt class="text-foreground-muted text-sm">Callbacks</dt>
                <dd class="m-0 font-mono text-sm tabular-nums">
                    {metrics?.callback_count.toLocaleString() ?? '0'}
                </dd>
            </div>
            <div
                class="min-[37.5rem]:border-border flex items-center justify-between gap-4 py-3 min-[37.5rem]:border-t min-[37.5rem]:pr-5">
                <dt class="text-foreground-muted text-sm">Callback frames</dt>
                <dd class="m-0 font-mono text-sm tabular-nums">
                    {metrics
                        ? `${metrics.last_callback_frames} current · ${metrics.max_callback_frames} max`
                        : '0'}
                </dd>
            </div>
            <div
                class="min-[37.5rem]:border-border flex items-center justify-between gap-4 py-3 min-[37.5rem]:border-t min-[37.5rem]:border-l min-[37.5rem]:pl-5">
                <dt class="text-foreground-muted text-sm">Rendered frames</dt>
                <dd class="m-0 font-mono text-sm tabular-nums">
                    {metrics?.rendered_frames.toLocaleString() ?? '0'}
                </dd>
            </div>
            <div
                class="min-[37.5rem]:border-border flex items-center justify-between gap-4 py-3 min-[37.5rem]:border-t min-[37.5rem]:pr-5">
                <dt class="text-foreground-muted text-sm">Stream errors</dt>
                <dd
                    class={`m-0 font-mono text-sm tabular-nums ${(metrics?.stream_error_count ?? 0) > 0 ? 'text-danger' : ''}`}>
                    {metrics?.stream_error_count.toLocaleString() ?? '0'}
                </dd>
            </div>
            <div
                class="min-[37.5rem]:border-border flex items-center justify-between gap-4 py-3 min-[37.5rem]:border-t min-[37.5rem]:border-l min-[37.5rem]:pl-5">
                <dt class="text-foreground-muted text-sm">Oversized callbacks</dt>
                <dd
                    class={`m-0 font-mono text-sm tabular-nums ${(metrics?.oversized_callback_count ?? 0) > 0 ? 'text-danger' : ''}`}>
                    {metrics?.oversized_callback_count.toLocaleString() ?? '0'}
                </dd>
            </div>
        </dl>

        <div class="border-border grid border-t min-[56.25rem]:grid-cols-2">
            <div class="min-[56.25rem]:border-r-border min-w-0 min-[56.25rem]:border-r">
                <h3
                    class="text-foreground m-0 px-3.75 pt-4 pb-2 text-sm font-semibold min-[56.25rem]:px-5">
                    Callback phases
                </h3>
                <div class="overflow-x-auto">
                    <table class="w-full min-w-96 border-collapse text-sm">
                        <thead class="text-foreground-muted">
                            <tr>
                                <th class="px-3.75 py-2 text-left font-normal min-[56.25rem]:pl-5">
                                    Phase
                                </th>
                                <th class="px-2 py-2 text-right font-normal">Last</th>
                                <th class="px-2 py-2 text-right font-normal">Average</th>
                                <th class="px-3.75 py-2 text-right font-normal min-[56.25rem]:pr-5">
                                    Peak
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-border divide-y">
                            {#each phaseRows() as phase (phase.name)}
                                <tr>
                                    <td class="px-3.75 py-2 min-[56.25rem]:pl-5">{phase.name}</td>
                                    <td class="px-2 py-2 text-right font-mono tabular-nums">
                                        {duration(phase.timing.last_time_us)}
                                    </td>
                                    <td class="px-2 py-2 text-right font-mono tabular-nums">
                                        {duration(phase.timing.average_time_us)}
                                    </td>
                                    <td
                                        class="px-3.75 py-2 text-right font-mono tabular-nums min-[56.25rem]:pr-5">
                                        {duration(phase.timing.max_time_us)}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="border-border min-w-0 border-t min-[56.25rem]:border-t-0">
                <h3
                    class="text-foreground m-0 px-3.75 pt-4 pb-2 text-sm font-semibold min-[56.25rem]:px-5">
                    Instruments
                </h3>
                {#if metrics?.instruments.length}
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-96 border-collapse text-sm">
                            <thead class="text-foreground-muted">
                                <tr>
                                    <th
                                        class="px-3.75 py-2 text-left font-normal min-[56.25rem]:pl-5">
                                        Instrument
                                    </th>
                                    <th class="px-2 py-2 text-right font-normal">Last</th>
                                    <th class="px-2 py-2 text-right font-normal">Average</th>
                                    <th
                                        class="px-3.75 py-2 text-right font-normal min-[56.25rem]:pr-5">
                                        Peak
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-border divide-y">
                                {#each metrics.instruments as instrument (instrument.instrument_id)}
                                    <tr>
                                        <td class="px-3.75 py-2 min-[56.25rem]:pl-5">
                                            Instrument #{instrument.instrument_id + 1}
                                        </td>
                                        <td class="px-2 py-2 text-right font-mono tabular-nums">
                                            {duration(instrument.last_time_us)}
                                        </td>
                                        <td class="px-2 py-2 text-right font-mono tabular-nums">
                                            {duration(instrument.average_time_us)}
                                        </td>
                                        <td
                                            class="px-3.75 py-2 text-right font-mono tabular-nums min-[56.25rem]:pr-5">
                                            {duration(instrument.max_time_us)}
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {:else}
                    <p class="text-foreground-muted m-0 px-3.75 pb-4 text-sm min-[56.25rem]:px-5">
                        No instrument render data yet.
                    </p>
                {/if}
            </div>
        </div>

        <div class="border-border border-t">
            <div class="px-3.75 pt-4 pb-2 min-[56.25rem]:px-5">
                <h3 class="text-foreground m-0 text-sm font-semibold">Deadline miss history</h3>
                <p class="text-foreground-muted mt-1 mb-0 text-xs">
                    Most recent first · up to 32 callbacks retained
                </p>
            </div>
            {#if deadlineMisses().length}
                <div class="overflow-x-auto">
                    <table class="w-full min-w-256 border-collapse text-sm">
                        <thead class="text-foreground-muted">
                            <tr>
                                <th class="px-3.75 py-2 text-left font-normal min-[56.25rem]:pl-5">
                                    Callback
                                </th>
                                <th class="px-2 py-2 text-right font-normal">Total</th>
                                <th class="px-2 py-2 text-right font-normal">Load</th>
                                <th class="px-2 py-2 text-left font-normal">Last MIDI event</th>
                                <th class="px-2 py-2 text-right font-normal">MIDI</th>
                                <th class="px-2 py-2 text-left font-normal">Slowest instrument</th>
                                <th class="px-2 py-2 text-right font-normal">Render</th>
                                <th class="px-3.75 py-2 text-right font-normal min-[56.25rem]:pr-5">
                                    Other phases
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-border divide-y">
                            {#each deadlineMisses() as miss (miss.callback_index)}
                                <tr>
                                    <td
                                        class="px-3.75 py-2 font-mono tabular-nums min-[56.25rem]:pl-5">
                                        #{miss.callback_index.toLocaleString()}
                                    </td>
                                    <td class="px-2 py-2 text-right font-mono tabular-nums">
                                        {duration(miss.total_time_us)}
                                    </td>
                                    <td
                                        class="text-danger px-2 py-2 text-right font-mono tabular-nums">
                                        {percent(miss.load_percent)}
                                    </td>
                                    <td class="px-2 py-2">
                                        <div>{describeMidiEvent(miss.last_midi_event)}</div>
                                        <div class="text-foreground-muted text-xs">
                                            {miss.midi_event_count} event{miss.midi_event_count ===
                                            1
                                                ? ''
                                                : 's'} in callback
                                        </div>
                                    </td>
                                    <td class="px-2 py-2 text-right font-mono tabular-nums">
                                        {duration(miss.midi_us)}
                                    </td>
                                    <td class="px-2 py-2">
                                        {miss.slowest_instrument_id === null
                                            ? 'None'
                                            : `Instrument #${miss.slowest_instrument_id + 1}`}
                                    </td>
                                    <td class="px-2 py-2 text-right font-mono tabular-nums">
                                        {duration(miss.slowest_instrument_time_us)}
                                    </td>
                                    <td
                                        class="px-3.75 py-2 text-right font-mono text-xs tabular-nums min-[56.25rem]:pr-5">
                                        Lock {duration(miss.lock_wait_us)} · commands {duration(
                                            miss.commands_us
                                        )} · global {duration(miss.global_processing_us)} · output {duration(
                                            miss.output_conversion_us
                                        )}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {:else}
                <p class="text-foreground-muted m-0 px-3.75 pb-4 text-sm min-[56.25rem]:px-5">
                    No missed deadlines since the last reset.
                </p>
            {/if}
        </div>
    </section>
</div>
