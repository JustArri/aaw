import { onMount } from 'svelte';
import type { TimeSeriesChartSeries } from '$lib/components/TimeSeriesChart.svelte';
import type { DeadlineMissMidiEventSnapshot, TimingMetricSnapshot } from '$lib/generated/models';
import { notify } from '$lib/push-notifications.svelte';
import { engineMetrics, fetchEngineMetrics, resetEngineMetrics } from '$lib/state/engine.svelte';

export class EnginePerformanceSection {
    resettingMetrics = $state(false);
    private metricsErrorShown = false;
    metrics = $derived(engineMetrics());
    loadHistory = $derived.by<TimeSeriesChartSeries[]>(() => {
        const history = this.metrics?.performance_history ?? [];
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
    renderHistory = $derived.by<TimeSeriesChartSeries[]>(() => {
        const history = this.metrics?.performance_history ?? [];
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

    constructor() {
        onMount(() => {
            void this.refreshMetrics();
            const timer = window.setInterval(() => void this.refreshMetrics(), 1000);
            return () => window.clearInterval(timer);
        });
    }

    private async refreshMetrics() {
        try {
            await fetchEngineMetrics();
            this.metricsErrorShown = false;
        } catch (error) {
            if (!this.metricsErrorShown) {
                this.metricsErrorShown = true;
                notify.error(
                    error instanceof Error
                        ? error.message
                        : 'Could not load engine performance metrics.',
                    { title: 'Engine monitoring' }
                );
            }
        }
    }

    phaseRows(): { name: string; timing: TimingMetricSnapshot }[] {
        if (!this.metrics) return [];
        return [
            { name: 'Engine lock wait', timing: this.metrics.phases.lock_wait },
            { name: 'Commands', timing: this.metrics.phases.commands },
            { name: 'MIDI events', timing: this.metrics.phases.midi },
            { name: 'Instrument rendering', timing: this.metrics.phases.instruments },
            { name: 'Global effects and mixing', timing: this.metrics.phases.global_processing },
            { name: 'Output conversion', timing: this.metrics.phases.output_conversion }
        ];
    }

    deadlineMisses() {
        return this.metrics ? [...this.metrics.deadline_misses].reverse() : [];
    }

    async resetMetrics() {
        if (this.resettingMetrics) return;
        this.resettingMetrics = true;
        try {
            await resetEngineMetrics();
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not reset engine metrics.',
                { title: 'Reset engine monitoring' }
            );
        } finally {
            this.resettingMetrics = false;
        }
    }
}

export function describeMidiEvent(entry: DeadlineMissMidiEventSnapshot | null) {
    if (!entry) return 'No MIDI event';
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

export function percent(value: number | undefined) {
    return `${(value ?? 0).toFixed(1)}%`;
}

export function duration(microseconds: number | undefined) {
    const value = microseconds ?? 0;
    return value >= 1000 ? `${(value / 1000).toFixed(2)} ms` : `${value.toFixed(0)} µs`;
}
