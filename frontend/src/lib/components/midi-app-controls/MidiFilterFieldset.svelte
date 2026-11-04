<script lang="ts">
    import NumberedCheckboxGrid from '$lib/components/NumberedCheckboxGrid.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import { MidiFilterKind, type MidiFilter } from '$lib/generated/models';

    interface Props {
        filter: MidiFilter;
        values: number[];
        name: string;
        ariaLabel: string;
        legend: string;
        checkedLabel: string;
        description: string;
        onmodechange: (value: string) => void;
        ontoggle: (value: number, checked: boolean) => void;
        onsetall: (values: number[]) => void;
    }

    let {
        filter,
        values,
        name,
        ariaLabel,
        legend,
        checkedLabel,
        description,
        onmodechange,
        ontoggle,
        onsetall
    }: Props = $props();
</script>

<fieldset class="border-border m-0 grid min-w-0 gap-3 rounded-lg border p-3">
    <legend class="text-foreground px-1 text-xs font-semibold">{legend}</legend>
    <SegmentedControl
        {name}
        {ariaLabel}
        compact
        value={filter.type}
        options={[
            { value: MidiFilterKind.Allowlist, label: 'Allow only' },
            { value: MidiFilterKind.Denylist, label: 'Allow except' }
        ]}
        onchange={onmodechange} />
    <NumberedCheckboxGrid
        {values}
        selected={filter.list}
        itemLabel={legend}
        {ontoggle}
        {onsetall} />
    <small class="text-foreground-muted">Checked {checkedLabel} are {description}.</small>
</fieldset>
