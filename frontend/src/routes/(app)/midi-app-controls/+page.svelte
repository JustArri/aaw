<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import Button from '$lib/components/Button.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import {
        MidiFilterKind,
        type CtrMidiEngineControl,
        type CtrMidiEngineControls,
        type MidiFilter
    } from '$lib/generated/models';
    import { errorToUserMessage } from '$lib/error';
    import { notify } from '$lib/push-notifications.svelte';
    import { subscribeMidiEvents, type MidiEventEntry } from '$lib/state/midi-events.svelte';
    import {
        engineMidiAppControls,
        fetchEngineMidiAppControls,
        saveEngineMidiAppControls
    } from '$lib/state/midi-app-controls.svelte';

    type ControlType = 'note-toggle' | 'note-enable' | 'note-disable' | 'cc-switch';

    const midiSources = Array.from({ length: 16 }, (_, value) => value);
    const midiChannels = Array.from({ length: 16 }, (_, value) => value);
    const controlTypes = [
        {
            value: 'note-toggle',
            label: 'Note toggles transport',
            description: 'Each matching Note On toggles Play/Stop.'
        },
        {
            value: 'note-enable',
            label: 'Note starts transport',
            description: 'Each matching Note On starts playback.'
        },
        {
            value: 'note-disable',
            label: 'Note stops transport',
            description: 'Each matching Note On stops playback.'
        },
        {
            value: 'cc-switch',
            label: 'CC switches transport',
            description: 'Values at or above the threshold play; lower values stop.'
        }
    ];

    let drafts = $state<CtrMidiEngineControl[]>([]);
    let dirty = $state(new Set<number>());
    let loading = $state(true);
    let saving = $state(false);
    let learningIndex = $state<number | null>(null);
    let initialized = $state(false);
    let unsubscribeMidiEvents: (() => void) | undefined;

    $effect(() => {
        const controls = engineMidiAppControls().controls;
        if (initialized && dirty.size === 0 && !saving) {
            drafts = snapshotControls(controls);
        }
    });

    onMount(async () => {
        unsubscribeMidiEvents = subscribeMidiEvents(handleLearnEvent);
        try {
            const controls = await fetchEngineMidiAppControls();
            drafts = snapshotControls(controls.controls);
            initialized = true;
        } catch (error) {
            notify.error(errorToUserMessage(error), { title: 'Load MIDI app controls' });
        } finally {
            loading = false;
        }
    });

    onDestroy(() => unsubscribeMidiEvents?.());

    function defaultFilter(): MidiFilter {
        return { type: MidiFilterKind.Denylist, list: [] };
    }

    function snapshotControls(controls: CtrMidiEngineControl[]) {
        return $state.snapshot(controls);
    }

    function defaultControl(): CtrMidiEngineControl {
        return {
            name: '',
            source_filter: defaultFilter(),
            channel_filter: defaultFilter(),
            kind: { CCTransportSwitch: { controller: 64, threshold: 64 } }
        };
    }

    function controlType(control: CtrMidiEngineControl): ControlType {
        if ('NoteTransportToggle' in control.kind) {
            return 'note-toggle';
        }
        if ('NoteTransportEnable' in control.kind) {
            return 'note-enable';
        }
        if ('NoteTransportDisable' in control.kind) {
            return 'note-disable';
        }
        return 'cc-switch';
    }

    function isNoteControl(control: CtrMidiEngineControl) {
        return !('CCTransportSwitch' in control.kind);
    }

    function noteKey(control: CtrMidiEngineControl) {
        if ('NoteTransportToggle' in control.kind) {
            return control.kind.NoteTransportToggle.key;
        }
        if ('NoteTransportEnable' in control.kind) {
            return control.kind.NoteTransportEnable.key;
        }
        if ('NoteTransportDisable' in control.kind) {
            return control.kind.NoteTransportDisable.key;
        }
        return 60;
    }

    function setNoteKey(control: CtrMidiEngineControl, key: number) {
        if ('NoteTransportEnable' in control.kind) {
            control.kind = { NoteTransportEnable: { key } };
        } else if ('NoteTransportDisable' in control.kind) {
            control.kind = { NoteTransportDisable: { key } };
        } else {
            control.kind = { NoteTransportToggle: { key } };
        }
    }

    function markDirty(index: number) {
        dirty = new Set(dirty).add(index);
    }

    function changeType(index: number, type: string) {
        const control = drafts[index];
        if (!control) {
            return;
        }
        switch (type as ControlType) {
            case 'note-toggle':
                control.kind = { NoteTransportToggle: { key: 60 } };
                break;
            case 'note-enable':
                control.kind = { NoteTransportEnable: { key: 60 } };
                break;
            case 'note-disable':
                control.kind = { NoteTransportDisable: { key: 60 } };
                break;
            case 'cc-switch':
                control.kind = { CCTransportSwitch: { controller: 64, threshold: 64 } };
                break;
        }
        markDirty(index);
    }

    function setFilterMode(index: number, key: 'source_filter' | 'channel_filter', mode: string) {
        const control = drafts[index];
        if (!control) {
            return;
        }
        control[key].type = mode as MidiFilterKind;
        markDirty(index);
    }

    function toggleFilterValue(
        index: number,
        key: 'source_filter' | 'channel_filter',
        value: number,
        checked: boolean
    ) {
        const control = drafts[index];
        if (!control) {
            return;
        }
        const values = control[key].list;
        control[key].list = checked
            ? [...new Set([...values, value])].sort((left, right) => left - right)
            : values.filter((entry) => entry !== value);
        markDirty(index);
    }

    function setAllFilterValues(
        index: number,
        key: 'source_filter' | 'channel_filter',
        values: number[]
    ) {
        const control = drafts[index];
        if (!control) {
            return;
        }
        control[key].list = [...values];
        markDirty(index);
    }

    function handleLearnEvent(entry: MidiEventEntry) {
        if (learningIndex === null) {
            return;
        }
        const control = drafts[learningIndex];
        if (!control) {
            learningIndex = null;
            return;
        }
        const event = entry.event;
        if (event.type === 'NoteOn') {
            setNoteKey(control, event.data.key);
        } else if (event.type === 'ControlChange') {
            control.kind = {
                CCTransportSwitch: {
                    controller: event.data.controller,
                    threshold: 64
                }
            };
        } else {
            return;
        }
        control.source_filter = { type: MidiFilterKind.Allowlist, list: [event.slot] };
        control.channel_filter = { type: MidiFilterKind.Allowlist, list: [event.channel] };
        markDirty(learningIndex);
        learningIndex = null;
    }

    async function persist(next: CtrMidiEngineControl[], title: string) {
        if (saving) {
            return false;
        }
        saving = true;
        try {
            const controls: CtrMidiEngineControls = { controls: snapshotControls(next) };
            const saved = await saveEngineMidiAppControls(controls);
            if (saved) {
                drafts = snapshotControls(controls.controls);
                dirty = new Set();
            }
            return saved;
        } catch (error) {
            notify.error(errorToUserMessage(error), { title });
            return false;
        } finally {
            saving = false;
        }
    }

    async function addControl() {
        await persist([...drafts, defaultControl()], 'Add MIDI app control');
    }

    async function applyControl() {
        await persist(drafts, 'Update MIDI app control');
    }

    async function removeControl(index: number) {
        learningIndex = learningIndex === index ? null : learningIndex;
        await persist(
            drafts.filter((_, controlIndex) => controlIndex !== index),
            'Remove MIDI app control'
        );
    }

    function filterDescription(filter: MidiFilter) {
        return filter.type === MidiFilterKind.Allowlist ? 'allowed' : 'blocked';
    }
</script>

<div class="grid gap-3">
    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Engine MIDI app controls
                </h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Global mappings loaded at startup and applied to every project.
                </p>
            </div>
            <Button
                variant="accent-outline"
                size="sm"
                busy={saving}
                disabled={loading || saving || dirty.size > 0}
                onclick={() => void addControl()}>
                <Icon icon="lucide:plus" width="0.875rem" />
                Add
            </Button>
        </header>

        {#if loading}
            <div class="text-foreground-muted grid min-h-40 place-items-center text-sm">
                Loading MIDI app controls…
            </div>
        {:else if drafts.length === 0}
            <div class="grid min-h-52 place-items-center gap-3 px-6 py-8 text-center">
                <div>
                    <Icon
                        icon="lucide:radio-tower"
                        width="2rem"
                        class="text-foreground-inactive mx-auto mb-3" />
                    <p class="text-foreground m-0 text-sm font-semibold">No engine mappings</p>
                    <p class="text-foreground-muted mt-1 mb-0 text-sm">
                        Add a mapping to control transport from an external MIDI device.
                    </p>
                </div>
                <Button
                    variant="primary"
                    size="sm"
                    disabled={saving}
                    onclick={() => void addControl()}>
                    <Icon icon="lucide:plus" width="0.875rem" />
                    Add mapping
                </Button>
            </div>
        {:else}
            <div class="grid gap-3 p-3.75 min-[56.25rem]:p-5">
                {#each drafts as control, index}
                    <form
                        class="border-border bg-surface-inset grid gap-4 rounded-xl border p-3.5"
                        onsubmit={(event) => {
                            event.preventDefault();
                            void applyControl();
                        }}>
                        <div class="flex flex-wrap items-end gap-2">
                            <label class="grid min-w-48 flex-1 gap-1">
                                <span class="text-foreground-muted text-xs">Name</span>
                                <input
                                    class="border-border-strong bg-surface-card text-foreground h-10 rounded-lg border px-2.5 text-sm"
                                    value={control.name}
                                    placeholder={`Mapping ${index + 1}`}
                                    oninput={(event) => {
                                        control.name = event.currentTarget.value;
                                        markDirty(index);
                                    }} />
                            </label>
                            <div class="flex items-center gap-1.5">
                                <Button
                                    variant={learningIndex === index ? 'danger' : 'secondary'}
                                    size="sm"
                                    aria-pressed={learningIndex === index}
                                    onclick={() =>
                                        (learningIndex = learningIndex === index ? null : index)}>
                                    <Icon
                                        icon={learningIndex === index
                                            ? 'lucide:radio-tower'
                                            : 'lucide:circle-dot'}
                                        width="0.875rem"
                                        class={learningIndex === index ? 'animate-pulse' : ''} />
                                    {learningIndex === index ? 'Listening…' : 'MIDI Learn'}
                                </Button>
                                <TimedConfirmButton
                                    label="Remove"
                                    confirmLabel="Confirm remove"
                                    busyLabel="Removing"
                                    busy={saving}
                                    disabled={dirty.size > 0}
                                    onconfirm={() => removeControl(index)} />
                            </div>
                        </div>

                        <div class="grid gap-3 min-[37.5rem]:grid-cols-2">
                            <label class="grid min-w-0 gap-1">
                                <span class="text-foreground-muted text-xs">Action</span>
                                <SelectDropdown
                                    value={controlType(control)}
                                    options={controlTypes}
                                    ariaLabel={`Mapping ${index + 1} action`}
                                    onchange={(value) => changeType(index, value)} />
                            </label>
                            {#if isNoteControl(control)}
                                <NumericControl
                                    compact
                                    label="MIDI note"
                                    value={noteKey(control)}
                                    unit="note"
                                    smallStep={1}
                                    largeStep={12}
                                    resetValue={60}
                                    min={0}
                                    max={127}
                                    onchange={(value) => {
                                        setNoteKey(control, value);
                                        markDirty(index);
                                    }} />
                            {:else if 'CCTransportSwitch' in control.kind}
                                <div class="grid gap-3 min-[37.5rem]:grid-cols-2">
                                    <NumericControl
                                        compact
                                        label="MIDI CC"
                                        value={control.kind.CCTransportSwitch.controller}
                                        unit="CC"
                                        smallStep={1}
                                        largeStep={10}
                                        resetValue={64}
                                        min={0}
                                        max={127}
                                        onchange={(value) => {
                                            if ('CCTransportSwitch' in control.kind) {
                                                control.kind.CCTransportSwitch.controller = value;
                                                markDirty(index);
                                            }
                                        }} />
                                    <NumericControl
                                        compact
                                        label="On threshold"
                                        value={control.kind.CCTransportSwitch.threshold}
                                        unit="MIDI"
                                        smallStep={1}
                                        largeStep={10}
                                        resetValue={64}
                                        min={0}
                                        max={127}
                                        onchange={(value) => {
                                            if ('CCTransportSwitch' in control.kind) {
                                                control.kind.CCTransportSwitch.threshold = value;
                                                markDirty(index);
                                            }
                                        }} />
                                </div>
                            {/if}
                        </div>

                        <div class="grid gap-4 min-[56.25rem]:grid-cols-2">
                            <fieldset
                                class="border-border m-0 grid min-w-0 gap-3 rounded-lg border p-3">
                                <legend class="text-foreground px-1 text-xs font-semibold">
                                    MIDI input sources
                                </legend>
                                <SegmentedControl
                                    name={`source-filter-${index}`}
                                    ariaLabel={`Mapping ${index + 1} source filter mode`}
                                    compact
                                    value={control.source_filter.type}
                                    options={[
                                        { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                        { value: MidiFilterKind.Denylist, label: 'Allow except' }
                                    ]}
                                    onchange={(value) =>
                                        setFilterMode(index, 'source_filter', value)} />
                                <div class="grid grid-cols-8 gap-1">
                                    {#each midiSources as source}
                                        <label
                                            class={`has-focus-visible:outline-accent grid min-h-8 cursor-pointer place-items-center rounded-md border font-mono text-xs transition-colors has-focus-visible:outline-2 ${control.source_filter.list.includes(source) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-card text-foreground-muted hover:border-accent/35'}`}>
                                            <input
                                                class="sr-only"
                                                type="checkbox"
                                                checked={control.source_filter.list.includes(
                                                    source
                                                )}
                                                onchange={(event) =>
                                                    toggleFilterValue(
                                                        index,
                                                        'source_filter',
                                                        source,
                                                        event.currentTarget.checked
                                                    )} />
                                            {source + 1}
                                        </label>
                                    {/each}
                                </div>
                                <div class="flex items-center justify-between gap-2">
                                    <small class="text-foreground-muted">
                                        Checked slots are {filterDescription(
                                            control.source_filter
                                        )}.
                                    </small>
                                    <div class="flex gap-1">
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(
                                                    index,
                                                    'source_filter',
                                                    midiSources
                                                )}>
                                            All
                                        </Button>
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(index, 'source_filter', [])}>
                                            None
                                        </Button>
                                    </div>
                                </div>
                            </fieldset>

                            <fieldset
                                class="border-border m-0 grid min-w-0 gap-3 rounded-lg border p-3">
                                <legend class="text-foreground px-1 text-xs font-semibold">
                                    MIDI channels
                                </legend>
                                <SegmentedControl
                                    name={`channel-filter-${index}`}
                                    ariaLabel={`Mapping ${index + 1} channel filter mode`}
                                    compact
                                    value={control.channel_filter.type}
                                    options={[
                                        { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                        { value: MidiFilterKind.Denylist, label: 'Allow except' }
                                    ]}
                                    onchange={(value) =>
                                        setFilterMode(index, 'channel_filter', value)} />
                                <div class="grid grid-cols-8 gap-1">
                                    {#each midiChannels as channel}
                                        <label
                                            class={`has-focus-visible:outline-accent grid min-h-8 cursor-pointer place-items-center rounded-md border font-mono text-xs transition-colors has-focus-visible:outline-2 ${control.channel_filter.list.includes(channel) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-card text-foreground-muted hover:border-accent/35'}`}>
                                            <input
                                                class="sr-only"
                                                type="checkbox"
                                                checked={control.channel_filter.list.includes(
                                                    channel
                                                )}
                                                onchange={(event) =>
                                                    toggleFilterValue(
                                                        index,
                                                        'channel_filter',
                                                        channel,
                                                        event.currentTarget.checked
                                                    )} />
                                            {channel + 1}
                                        </label>
                                    {/each}
                                </div>
                                <div class="flex items-center justify-between gap-2">
                                    <small class="text-foreground-muted">
                                        Checked channels are {filterDescription(
                                            control.channel_filter
                                        )}.
                                    </small>
                                    <div class="flex gap-1">
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(
                                                    index,
                                                    'channel_filter',
                                                    midiChannels
                                                )}>
                                            All
                                        </Button>
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(index, 'channel_filter', [])}>
                                            None
                                        </Button>
                                    </div>
                                </div>
                            </fieldset>
                        </div>

                        <div class="flex items-center justify-between gap-3">
                            <small class="text-foreground-muted">
                                {#if learningIndex === index}
                                    Move a CC control or play a note on the desired MIDI source.
                                {:else if dirty.has(index)}
                                    This mapping has unapplied changes.
                                {:else}
                                    Mapping {index + 1} is synchronized with the engine.
                                {/if}
                            </small>
                            <ApplyButton
                                {saving}
                                pending={dirty.has(index)}
                                label="Apply all MIDI app control changes" />
                        </div>
                    </form>
                {/each}
            </div>
        {/if}
    </section>
</div>
