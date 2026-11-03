<script module lang="ts">
    export type TimeSeriesChartSeries = {
        label: string;
        color: string;
        values: number[];
    };
</script>

<script lang="ts">
    let {
        series,
        minimumMaximum = 1,
        formatValue,
        emptyMessage = 'Waiting for monitoring history.'
    }: {
        series: TimeSeriesChartSeries[];
        minimumMaximum?: number;
        formatValue: (value: number) => string;
        emptyMessage?: string;
    } = $props();

    const width = 600;
    const height = 180;
    const left = 48;
    const right = 10;
    const top = 10;
    const bottom = 24;
    const plotWidth = width - left - right;
    const plotHeight = height - top - bottom;

    let pointCount = $derived(Math.max(0, ...series.map((entry) => entry.values.length)));
    let maximum = $derived.by(() => {
        const largest = Math.max(
            minimumMaximum,
            ...series.flatMap((entry) => entry.values.filter(Number.isFinite))
        );
        return niceMaximum(largest);
    });
    let ticks = $derived([maximum, maximum / 2, 0]);

    function niceMaximum(value: number) {
        if (value <= 0) {
            return 1;
        }

        const magnitude = 10 ** Math.floor(Math.log10(value));
        const normalized = value / magnitude;
        const rounded = normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10;
        return rounded * magnitude;
    }

    function x(index: number, count: number) {
        return left + (count <= 1 ? plotWidth : (index / (count - 1)) * plotWidth);
    }

    function y(value: number) {
        return top + plotHeight - (Math.max(0, value) / maximum) * plotHeight;
    }

    function points(values: number[]) {
        return values.map((value, index) => `${x(index, values.length)},${y(value)}`).join(' ');
    }
</script>

<div class="min-w-0">
    <div class="mb-2 flex flex-wrap gap-x-4 gap-y-1">
        {#each series as entry (entry.label)}
            <div class="text-foreground-muted flex items-center gap-1.5 text-xs">
                <span
                    class="h-0.5 w-3 rounded-full"
                    style:background-color={entry.color}
                    aria-hidden="true">
                </span>
                <span>{entry.label}</span>
                <strong class="text-foreground font-mono font-normal tabular-nums">
                    {formatValue(entry.values.at(-1) ?? 0)}
                </strong>
            </div>
        {/each}
    </div>

    {#if pointCount > 0}
        <svg
            class="h-44 w-full overflow-visible"
            viewBox={`0 0 ${width} ${height}`}
            role="img"
            aria-label={`Time-series chart containing ${pointCount} readings`}>
            {#each ticks as tick}
                <line
                    x1={left}
                    x2={width - right}
                    y1={y(tick)}
                    y2={y(tick)}
                    stroke="var(--border)"
                    stroke-width="1" />
                <text
                    x={left - 7}
                    y={y(tick) + 4}
                    fill="var(--foreground-muted)"
                    font-size="11"
                    text-anchor="end">
                    {formatValue(tick)}
                </text>
            {/each}

            {#each series as entry (entry.label)}
                <polyline
                    points={points(entry.values)}
                    fill="none"
                    stroke={entry.color}
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    vector-effect="non-scaling-stroke" />
            {/each}

            <text x={left} y={height - 4} fill="var(--foreground-muted)" font-size="11">
                {pointCount === 1 ? '1 second ago' : `${pointCount} seconds ago`}
            </text>
            <text
                x={width - right}
                y={height - 4}
                fill="var(--foreground-muted)"
                font-size="11"
                text-anchor="end">
                now
            </text>
        </svg>
    {:else}
        <div
            class="text-foreground-muted border-border grid h-44 place-items-center rounded-lg border border-dashed px-4 text-center text-sm">
            {emptyMessage}
        </div>
    {/if}
</div>
