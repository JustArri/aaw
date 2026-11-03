<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import TimeSeriesChart, {
        type TimeSeriesChartSeries
    } from '$lib/components/TimeSeriesChart.svelte';
    import { notify } from '$lib/push-notifications.svelte';
    import type {
        DeadlineMissMidiEventSnapshot,
        TimingMetricSnapshot
    } from '$lib/generated/models';
    import {
        engineMetrics,
        fetchEngineMetrics,
        resetEngineMetrics
    } from '$lib/state/engine.svelte';
    let resettingMetrics = $state(false);
    let metricsErrorShown = false;
    let metrics = $derived(engineMetrics());
    let loadHistory = $derived.by<TimeSeriesChartSeries[]>(() => {
        const history = metrics?.performance_history ?? [];
        return [
            {
                label: 'Average',
                color: 'var(--accent)',
                values: history.map((sample) => sample.average_load_percent)
            },
            {
                label: 'Peak',
                color: 'var(--danger)',
                values: history.map((sample) => sample.peak_load_percent)
            }
        ];
    });
    let renderHistory = $derived.by<TimeSeriesChartSeries[]>(() => {
        const history = metrics?.performance_history ?? [];
        return [
            {
                label: 'Average',
                color: 'var(--accent)',
                values: history.map((sample) => sample.average_render_time_us)
            },
            {
                label: 'Maximum',
                color: 'var(--danger)',
                values: history.map((sample) => sample.max_render_time_us)
            },
            {
                label: 'Budget',
                color: 'var(--success)',
                values: history.map((sample) => sample.average_budget_time_us)
            }
        ];
    });

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
                await fetchEngineMetrics();
                metricsErrorShown = false;
            } catch (error) {
                if (!metricsErrorShown) {
                    metricsErrorShown = true;
                    notify.error(
                        error instanceof Error
                            ? error.message
                            : 'Could not load engine performance metrics.',
                        { title: 'Engine monitoring' }
                    );
                }
            }
        }

        void refreshMetrics();
        const timer = window.setInterval(() => void refreshMetrics(), 1000);
        return () => window.clearInterval(timer);
    });
    async function resetMetrics() {
        if (resettingMetrics) {
            return;
        }

        resettingMetrics = true;
        try {
            await resetEngineMetrics();
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not reset engine metrics.',
                { title: 'Reset engine monitoring' }
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

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
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
                Engine callback load and missed processing deadlines.
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

    <div class="border-border grid border-b min-[75rem]:grid-cols-2">
        <div class="min-w-0 p-3.75 min-[56.25rem]:p-5 min-[75rem]:border-r">
            <div class="mb-3 flex flex-wrap items-start justify-between gap-3">
                <div>
                    <h3 class="text-foreground m-0 text-sm font-semibold">Callback load</h3>
                    <p class="text-foreground-muted mt-1 mb-0 text-xs">
                        One-second averages and peaks · up to two minutes
                    </p>
                </div>
                <div class="flex gap-4 text-right">
                    <div>
                        <div class="text-foreground-muted text-xs uppercase">Current</div>
                        <strong class="font-mono text-sm tabular-nums">
                            {percent(metrics?.last_load_percent)}
                        </strong>
                    </div>
                    <div>
                        <div class="text-foreground-muted text-xs uppercase">Misses</div>
                        <strong
                            class={`font-mono text-sm tabular-nums ${(metrics?.deadline_miss_count ?? 0) > 0 ? 'text-danger' : ''}`}>
                            {metrics?.deadline_miss_count.toLocaleString() ?? '0'}
                        </strong>
                    </div>
                </div>
            </div>
            <TimeSeriesChart
                series={loadHistory}
                minimumMaximum={100}
                formatValue={(value) => `${Math.round(value)}%`} />
        </div>

        <div
            class="border-border min-w-0 border-t p-3.75 min-[56.25rem]:p-5 min-[75rem]:border-t-0">
            <div class="mb-3 flex flex-wrap items-start justify-between gap-3">
                <div>
                    <h3 class="text-foreground m-0 text-sm font-semibold">Render timing</h3>
                    <p class="text-foreground-muted mt-1 mb-0 text-xs">
                        Callback render time compared with its deadline budget
                    </p>
                </div>
                <div class="flex gap-4 text-right">
                    <div>
                        <div class="text-foreground-muted text-xs uppercase">Current</div>
                        <strong class="font-mono text-sm tabular-nums">
                            {duration(metrics?.last_render_time_us)}
                        </strong>
                    </div>
                    <div>
                        <div class="text-foreground-muted text-xs uppercase">Longest</div>
                        <strong class="font-mono text-sm tabular-nums">
                            {duration(metrics?.max_render_time_us)}
                        </strong>
                    </div>
                </div>
            </div>
            <TimeSeriesChart series={renderHistory} formatValue={(value) => duration(value)} />
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
                                <th class="px-3.75 py-2 text-left font-normal min-[56.25rem]:pl-5">
                                    Instrument
                                </th>
                                <th class="px-2 py-2 text-right font-normal">Last</th>
                                <th class="px-2 py-2 text-right font-normal">Average</th>
                                <th class="px-3.75 py-2 text-right font-normal min-[56.25rem]:pr-5">
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
                                <td class="px-3.75 py-2 font-mono tabular-nums min-[56.25rem]:pl-5">
                                    #{miss.callback_index.toLocaleString()}
                                </td>
                                <td class="px-2 py-2 text-right font-mono tabular-nums">
                                    {duration(miss.total_time_us)}
                                </td>
                                <td class="text-danger px-2 py-2 text-right font-mono tabular-nums">
                                    {percent(miss.load_percent)}
                                </td>
                                <td class="px-2 py-2">
                                    <div>{describeMidiEvent(miss.last_midi_event)}</div>
                                    <div class="text-foreground-muted text-xs">
                                        {miss.midi_event_count} event{miss.midi_event_count === 1
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
