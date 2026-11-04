<script lang="ts">
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import BypassToggle from '$lib/components/BypassToggle.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import {
        type InstrumentsSection,
        submitControlGroupOnEnter
    } from '$lib/sections/instruments.svelte';
    import KeyMappingField from './KeyMappingField.svelte';
    import KeySelectionFields from './KeySelectionFields.svelte';

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
        void section.keySection.save();
    }}>
    <div class="flex items-center justify-between gap-3">
        <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">Keys</h3>
        <BypassToggle
            bind:checked={section.keySection.ignored}
            saving={section.savingSections.has('ignore-key-transform')}
            onchange={() => void section.keySection.saveIgnored()} />
    </div>
    <div class="grid gap-2">
        <div class="grid min-w-0 gap-4">
            <div class="grid grid-cols-2 gap-2">
                <div class="grid gap-1 text-sm">
                    <span class="text-foreground-muted">Selection</span>
                    <SegmentedControl
                        name="key-selection"
                        ariaLabel="Key selection"
                        bind:value={section.keySection.selectionType}
                        options={[
                            { value: 'All', label: 'All' },
                            { value: 'Range', label: 'Range' },
                            { value: 'Keys', label: 'Specific' }
                        ]}
                        onchange={() => section.keySection.stopRecording()} />
                </div>
                <div class="grid gap-1 text-sm">
                    <span class="text-foreground-muted">Mapping</span>
                    <SegmentedControl
                        name="key-mapping"
                        ariaLabel="Key mapping"
                        bind:value={section.keySection.mappingType}
                        options={[
                            { value: 'Identity', label: 'Identity' },
                            { value: 'Custom', label: 'Custom' }
                        ]}
                        onchange={() => section.keySection.stopRecording()} />
                </div>
            </div>
            <KeySelectionFields {section} />
            <KeyMappingField {section} />
            <NumericControl
                label="Local transposition"
                value={section.keySection.transposition}
                unit="semitones"
                smallStep={1}
                largeStep={12}
                resetValue={0}
                min={-128}
                max={127}
                wideValue
                compact
                onchange={(value) => {
                    section.keySection.transposition = value;
                }} />
        </div>
        <ApplyButton
            saving={section.savingSections.has('keys')}
            pending={section.keySection.changed} />
    </div>
</form>
