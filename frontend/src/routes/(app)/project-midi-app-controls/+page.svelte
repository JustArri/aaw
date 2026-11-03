<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import Button from '$lib/components/Button.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import { errorToUserMessage } from '$lib/error';
    import {
        MidiFilterKind,
        type CtrMidiProjectControl,
        type CtrMidiProjectControls,
        type MidiFilter
    } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import {
        fetchProjectMidiAppControls,
        saveProjectMidiAppControls
    } from '$lib/state/midi-app-controls.svelte';
    import { subscribeMidiEvents, type MidiEventEntry } from '$lib/state/midi-events.svelte';
    import { controllerEntries, fetchSynths, synths } from '$lib/state/synths.svelte';
    import { fetchUserPresets, userPresets } from '$lib/state/user-presets.svelte';

    type ControlType =
        | 'note-controller-toggle'
        | 'note-controller-enable'
        | 'note-controller-disable'
        | 'cc-controller-switch'
        | 'note-preset'
        | 'cc-preset';

    const midiSources = Array.from({ length: 16 }, (_, value) => value);
    const midiChannels = Array.from({ length: 16 }, (_, value) => value);

    let drafts = $state<CtrMidiProjectControl[]>([]);
    let dirty = $state(new Set<number>());
    let loading = $state(true);
    let saving = $state(false);
    let learningIndex = $state<number | null>(null);
    let initialized = $state(false);
    let unsubscribeMidiEvents: (() => void) | undefined;

    let controllerOptions = $derived.by(() => {
        const instrumentNames = new Map(
            synths().map((instrument) => [instrument.id, instrument.synth.name])
        );
        return controllerEntries().map(({ instrument_id, controller_id, controller }) => ({
            value: `${instrument_id}:${controller_id}`,
            label: `${instrumentNames.get(instrument_id) || `Untitled instrument #${instrument_id + 1}`} · ${controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine'}: ${controller.data.name || `Controller #${controller_id + 1}`}`
        }));
    });
    let presetOptions = $derived.by(() =>
        userPresets().entries.map((preset, index) => ({
            value: String(index),
            label: preset.name || `User preset #${index + 1}`
        }))
    );
    let controlTypes = $derived([
        {
            value: 'note-controller-toggle',
            label: 'Note toggles controller',
            description: 'Toggle an instrument controller on each matching Note On.',
            disabled: controllerOptions.length === 0
        },
        {
            value: 'note-controller-enable',
            label: 'Note enables controller',
            description: 'Enable an instrument controller on each matching Note On.',
            disabled: controllerOptions.length === 0
        },
        {
            value: 'note-controller-disable',
            label: 'Note disables controller',
            description: 'Disable an instrument controller on each matching Note On.',
            disabled: controllerOptions.length === 0
        },
        {
            value: 'cc-controller-switch',
            label: 'CC switches controller',
            description: 'Enable or disable a controller according to a CC threshold.',
            disabled: controllerOptions.length === 0
        },
        {
            value: 'note-preset',
            label: 'Note selects user preset',
            description: 'Activate a user preset on each matching Note On.',
            disabled: presetOptions.length === 0
        },
        {
            value: 'cc-preset',
            label: 'CC selects user preset',
            description: 'Activate a user preset when the matching CC is nonzero.',
            disabled: presetOptions.length === 0
        }
    ]);

    $effect(() => {
        const controls = activeProject()?.midi_app_controls.controls;
        if (initialized && controls && dirty.size === 0 && !saving) {
            drafts = snapshotControls(controls);
        }
    });

    onMount(async () => {
        unsubscribeMidiEvents = subscribeMidiEvents(handleLearnEvent);
        try {
            const [controls] = await Promise.all([
                fetchProjectMidiAppControls(),
                fetchSynths(),
                fetchUserPresets()
            ]);
            drafts = snapshotControls(controls?.controls ?? []);
            initialized = true;
        } catch (error) {
            notify.error(errorToUserMessage(error), { title: 'Load project MIDI app controls' });
        } finally {
            loading = false;
        }
    });

    onDestroy(() => unsubscribeMidiEvents?.());

    function snapshotControls(controls: CtrMidiProjectControl[]) {
        return $state.snapshot(controls);
    }

    function defaultFilter(): MidiFilter {
        return { type: MidiFilterKind.Denylist, list: [] };
    }

    function firstControllerAddress() {
        const [instrument = '0', controller = '0'] = (controllerOptions[0]?.value ?? '0:0').split(
            ':'
        );
        return { instrument: Number(instrument), controller: Number(controller) };
    }

    function defaultControl(): CtrMidiProjectControl {
        const filters = { source_filter: defaultFilter(), channel_filter: defaultFilter() };
        if (controllerOptions.length > 0) {
            return {
                name: '',
                ...filters,
                kind: { NoteInstrumentControllerToggle: { key: 60, ...firstControllerAddress() } }
            };
        }
        return {
            name: '',
            ...filters,
            kind: { NoteUserPresetSelector: { key: 60, preset_index: 0 } }
        };
    }

    function controlType(control: CtrMidiProjectControl): ControlType {
        if ('NoteInstrumentControllerToggle' in control.kind) return 'note-controller-toggle';
        if ('NoteInstrumentControllerEnable' in control.kind) return 'note-controller-enable';
        if ('NoteInstrumentControllerDisable' in control.kind) return 'note-controller-disable';
        if ('CCInstrumentControllerSwitch' in control.kind) return 'cc-controller-switch';
        if ('NoteUserPresetSelector' in control.kind) return 'note-preset';
        return 'cc-preset';
    }

    function isControllerControl(control: CtrMidiProjectControl) {
        return !(
            'NoteUserPresetSelector' in control.kind || 'CCUserPresetSelector' in control.kind
        );
    }

    function isNoteControl(control: CtrMidiProjectControl) {
        return !(
            'CCInstrumentControllerSwitch' in control.kind || 'CCUserPresetSelector' in control.kind
        );
    }

    function controllerTarget(control: CtrMidiProjectControl) {
        if ('CCInstrumentControllerSwitch' in control.kind) {
            const target = control.kind.CCInstrumentControllerSwitch;
            return { instrument: target.instrument, controller: target.instrument_controller };
        }
        if ('NoteInstrumentControllerToggle' in control.kind)
            return control.kind.NoteInstrumentControllerToggle;
        if ('NoteInstrumentControllerEnable' in control.kind)
            return control.kind.NoteInstrumentControllerEnable;
        if ('NoteInstrumentControllerDisable' in control.kind)
            return control.kind.NoteInstrumentControllerDisable;
        return firstControllerAddress();
    }

    function controllerTargetValue(control: CtrMidiProjectControl) {
        const target = controllerTarget(control);
        return `${target.instrument}:${target.controller}`;
    }

    function controllerOptionsFor(control: CtrMidiProjectControl) {
        const value = controllerTargetValue(control);
        return controllerOptions.some((option) => option.value === value)
            ? controllerOptions
            : [...controllerOptions, { value, label: `Missing controller (${value})` }];
    }

    function presetIndex(control: CtrMidiProjectControl) {
        if ('NoteUserPresetSelector' in control.kind)
            return control.kind.NoteUserPresetSelector.preset_index;
        if ('CCUserPresetSelector' in control.kind)
            return control.kind.CCUserPresetSelector.preset_index;
        return 0;
    }

    function presetOptionsFor(control: CtrMidiProjectControl) {
        const value = String(presetIndex(control));
        return presetOptions.some((option) => option.value === value)
            ? presetOptions
            : [...presetOptions, { value, label: `Missing user preset #${Number(value) + 1}` }];
    }

    function noteKey(control: CtrMidiProjectControl) {
        if ('NoteInstrumentControllerToggle' in control.kind)
            return control.kind.NoteInstrumentControllerToggle.key;
        if ('NoteInstrumentControllerEnable' in control.kind)
            return control.kind.NoteInstrumentControllerEnable.key;
        if ('NoteInstrumentControllerDisable' in control.kind)
            return control.kind.NoteInstrumentControllerDisable.key;
        if ('NoteUserPresetSelector' in control.kind)
            return control.kind.NoteUserPresetSelector.key;
        return 60;
    }

    function ccNumber(control: CtrMidiProjectControl) {
        if ('CCInstrumentControllerSwitch' in control.kind)
            return control.kind.CCInstrumentControllerSwitch.controller;
        if ('CCUserPresetSelector' in control.kind)
            return control.kind.CCUserPresetSelector.controller;
        return 64;
    }

    function changeType(index: number, type: string) {
        const control = drafts[index];
        if (!control) return;
        const target = controllerTarget(control);
        const preset_index = presetIndex(control);
        switch (type as ControlType) {
            case 'note-controller-toggle':
                control.kind = { NoteInstrumentControllerToggle: { key: 60, ...target } };
                break;
            case 'note-controller-enable':
                control.kind = { NoteInstrumentControllerEnable: { key: 60, ...target } };
                break;
            case 'note-controller-disable':
                control.kind = { NoteInstrumentControllerDisable: { key: 60, ...target } };
                break;
            case 'cc-controller-switch':
                control.kind = {
                    CCInstrumentControllerSwitch: {
                        controller: 64,
                        threshold: 64,
                        instrument: target.instrument,
                        instrument_controller: target.controller
                    }
                };
                break;
            case 'note-preset':
                control.kind = { NoteUserPresetSelector: { key: 60, preset_index } };
                break;
            case 'cc-preset':
                control.kind = { CCUserPresetSelector: { controller: 64, preset_index } };
                break;
        }
        markDirty(index);
    }

    function setControllerTarget(index: number, value: string) {
        const control = drafts[index];
        if (!control) return;
        const [instrument, controller] = value.split(':').map(Number);
        if ('NoteInstrumentControllerToggle' in control.kind)
            control.kind.NoteInstrumentControllerToggle = {
                ...control.kind.NoteInstrumentControllerToggle,
                instrument,
                controller
            };
        else if ('NoteInstrumentControllerEnable' in control.kind)
            control.kind.NoteInstrumentControllerEnable = {
                ...control.kind.NoteInstrumentControllerEnable,
                instrument,
                controller
            };
        else if ('NoteInstrumentControllerDisable' in control.kind)
            control.kind.NoteInstrumentControllerDisable = {
                ...control.kind.NoteInstrumentControllerDisable,
                instrument,
                controller
            };
        else if ('CCInstrumentControllerSwitch' in control.kind)
            control.kind.CCInstrumentControllerSwitch = {
                ...control.kind.CCInstrumentControllerSwitch,
                instrument,
                instrument_controller: controller
            };
        markDirty(index);
    }

    function setPreset(index: number, value: string) {
        const control = drafts[index];
        if (!control) return;
        const preset_index = Number(value);
        if ('NoteUserPresetSelector' in control.kind)
            control.kind.NoteUserPresetSelector.preset_index = preset_index;
        else if ('CCUserPresetSelector' in control.kind)
            control.kind.CCUserPresetSelector.preset_index = preset_index;
        markDirty(index);
    }

    function setTrigger(control: CtrMidiProjectControl, value: number) {
        if ('NoteInstrumentControllerToggle' in control.kind)
            control.kind.NoteInstrumentControllerToggle.key = value;
        else if ('NoteInstrumentControllerEnable' in control.kind)
            control.kind.NoteInstrumentControllerEnable.key = value;
        else if ('NoteInstrumentControllerDisable' in control.kind)
            control.kind.NoteInstrumentControllerDisable.key = value;
        else if ('NoteUserPresetSelector' in control.kind)
            control.kind.NoteUserPresetSelector.key = value;
        else if ('CCInstrumentControllerSwitch' in control.kind)
            control.kind.CCInstrumentControllerSwitch.controller = value;
        else if ('CCUserPresetSelector' in control.kind)
            control.kind.CCUserPresetSelector.controller = value;
    }

    function markDirty(index: number) {
        dirty = new Set(dirty).add(index);
    }

    function setFilterMode(index: number, key: 'source_filter' | 'channel_filter', mode: string) {
        const control = drafts[index];
        if (!control) return;
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
        if (!control) return;
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
        if (!control) return;
        control[key].list = [...values];
        markDirty(index);
    }

    function handleLearnEvent(entry: MidiEventEntry) {
        if (learningIndex === null) return;
        const control = drafts[learningIndex];
        if (!control) return;
        const event = entry.event;
        if (event.type === 'NoteOn') {
            if (!isNoteControl(control)) {
                if (isControllerControl(control)) {
                    const target = controllerTarget(control);
                    control.kind = {
                        NoteInstrumentControllerToggle: { key: event.data.key, ...target }
                    };
                } else {
                    control.kind = {
                        NoteUserPresetSelector: {
                            key: event.data.key,
                            preset_index: presetIndex(control)
                        }
                    };
                }
            } else setTrigger(control, event.data.key);
        } else if (event.type === 'ControlChange') {
            if (isNoteControl(control)) {
                if (isControllerControl(control)) {
                    const target = controllerTarget(control);
                    control.kind = {
                        CCInstrumentControllerSwitch: {
                            controller: event.data.controller,
                            threshold: 64,
                            instrument: target.instrument,
                            instrument_controller: target.controller
                        }
                    };
                } else {
                    control.kind = {
                        CCUserPresetSelector: {
                            controller: event.data.controller,
                            preset_index: presetIndex(control)
                        }
                    };
                }
            } else setTrigger(control, event.data.controller);
        } else return;
        control.source_filter = { type: MidiFilterKind.Allowlist, list: [event.slot] };
        control.channel_filter = { type: MidiFilterKind.Allowlist, list: [event.channel] };
        markDirty(learningIndex);
        learningIndex = null;
    }

    async function persist(next: CtrMidiProjectControl[], title: string) {
        if (saving) return false;
        saving = true;
        try {
            const controls: CtrMidiProjectControls = { controls: snapshotControls(next) };
            const saved = await saveProjectMidiAppControls(controls);
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
        await persist([...drafts, defaultControl()], 'Add project MIDI app control');
    }

    async function removeControl(index: number) {
        if (learningIndex === index) learningIndex = null;
        await persist(
            drafts.filter((_, controlIndex) => controlIndex !== index),
            'Remove project MIDI app control'
        );
    }

    function filterDescription(filter: MidiFilter) {
        return filter.type === MidiFilterKind.Allowlist ? 'allowed' : 'blocked';
    }
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Project MIDI app controls
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                Mappings saved with the current project.
            </p>
        </div>
        <Button
            variant="accent-outline"
            size="sm"
            busy={saving}
            disabled={loading || saving || dirty.size > 0}
            onclick={() => void addControl()}>
            <Icon icon="lucide:plus" width="0.875rem" /> Add
        </Button>
    </header>

    {#if loading}
        <div class="text-foreground-muted grid min-h-40 place-items-center text-sm">Loading…</div>
    {:else if drafts.length === 0}
        <div class="grid min-h-52 place-items-center gap-3 px-6 py-8 text-center">
            <div>
                <Icon
                    icon="lucide:radio-tower"
                    width="2rem"
                    class="text-foreground-inactive mx-auto mb-3" />
                <p class="text-foreground m-0 text-sm font-semibold">No project mappings</p>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Add a mapping to control this project from a MIDI device.
                </p>
            </div>
            <Button variant="primary" size="sm" disabled={saving} onclick={() => void addControl()}>
                <Icon icon="lucide:plus" width="0.875rem" /> Add mapping
            </Button>
        </div>
    {:else}
        <div class="grid gap-3 p-3.75 min-[56.25rem]:p-5">
            {#each drafts as control, index}
                <form
                    class="border-border bg-surface-inset grid gap-4 rounded-xl border p-3.5"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void persist(drafts, 'Update project MIDI app control');
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
                                    icon="lucide:circle-dot"
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
                        <label class="grid min-w-0 gap-1">
                            <span class="text-foreground-muted text-xs">
                                {isControllerControl(control)
                                    ? 'Instrument controller'
                                    : 'User preset'}
                            </span>
                            {#if isControllerControl(control)}
                                <SelectDropdown
                                    value={controllerTargetValue(control)}
                                    options={controllerOptionsFor(control)}
                                    ariaLabel={`Mapping ${index + 1} instrument controller`}
                                    onchange={(value) => setControllerTarget(index, value)} />
                            {:else}
                                <SelectDropdown
                                    value={String(presetIndex(control))}
                                    options={presetOptionsFor(control)}
                                    ariaLabel={`Mapping ${index + 1} user preset`}
                                    onchange={(value) => setPreset(index, value)} />
                            {/if}
                        </label>
                    </div>

                    <div class="grid gap-3 min-[37.5rem]:grid-cols-2">
                        <NumericControl
                            compact
                            label={isNoteControl(control) ? 'MIDI note' : 'MIDI CC'}
                            value={isNoteControl(control) ? noteKey(control) : ccNumber(control)}
                            unit={isNoteControl(control) ? 'note' : 'CC'}
                            smallStep={1}
                            largeStep={isNoteControl(control) ? 12 : 10}
                            resetValue={isNoteControl(control) ? 60 : 64}
                            min={0}
                            max={127}
                            onchange={(value) => {
                                setTrigger(control, value);
                                markDirty(index);
                            }} />
                        {#if 'CCInstrumentControllerSwitch' in control.kind}
                            <NumericControl
                                compact
                                label="On threshold"
                                value={control.kind.CCInstrumentControllerSwitch.threshold}
                                unit="MIDI"
                                smallStep={1}
                                largeStep={10}
                                resetValue={64}
                                min={0}
                                max={127}
                                onchange={(value) => {
                                    if ('CCInstrumentControllerSwitch' in control.kind) {
                                        control.kind.CCInstrumentControllerSwitch.threshold = value;
                                        markDirty(index);
                                    }
                                }} />
                        {/if}
                    </div>

                    <div class="grid gap-4 min-[56.25rem]:grid-cols-2">
                        {#each [{ key: 'source_filter' as const, label: 'MIDI input sources', values: midiSources }, { key: 'channel_filter' as const, label: 'MIDI channels', values: midiChannels }] as filter}
                            <fieldset
                                class="border-border m-0 grid min-w-0 gap-3 rounded-lg border p-3">
                                <legend class="text-foreground px-1 text-xs font-semibold">
                                    {filter.label}
                                </legend>
                                <SegmentedControl
                                    name={`${filter.key}-${index}`}
                                    ariaLabel={`Mapping ${index + 1} ${filter.label} mode`}
                                    compact
                                    value={control[filter.key].type}
                                    options={[
                                        { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                        { value: MidiFilterKind.Denylist, label: 'Allow except' }
                                    ]}
                                    onchange={(value) => setFilterMode(index, filter.key, value)} />
                                <div class="grid grid-cols-8 gap-1">
                                    {#each filter.values as value}
                                        <label
                                            class={`has-focus-visible:outline-accent grid min-h-8 cursor-pointer place-items-center rounded-md border font-mono text-xs transition-colors has-focus-visible:outline-2 ${control[filter.key].list.includes(value) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-card text-foreground-muted hover:border-accent/35'}`}>
                                            <input
                                                class="sr-only"
                                                type="checkbox"
                                                checked={control[filter.key].list.includes(value)}
                                                onchange={(event) =>
                                                    toggleFilterValue(
                                                        index,
                                                        filter.key,
                                                        value,
                                                        event.currentTarget.checked
                                                    )} />
                                            {value + 1}
                                        </label>
                                    {/each}
                                </div>
                                <div class="flex items-center justify-between gap-2">
                                    <small class="text-foreground-muted">
                                        Checked values are {filterDescription(control[filter.key])}.
                                    </small>
                                    <div class="flex gap-1">
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(
                                                    index,
                                                    filter.key,
                                                    filter.values
                                                )}>
                                            All
                                        </Button>
                                        <Button
                                            variant="secondary"
                                            size="xs"
                                            onclick={() =>
                                                setAllFilterValues(index, filter.key, [])}>
                                            None
                                        </Button>
                                    </div>
                                </div>
                            </fieldset>
                        {/each}
                    </div>

                    <div class="flex items-center justify-between gap-3">
                        <small class="text-foreground-muted">
                            {learningIndex === index
                                ? 'Move a CC control or play a note.'
                                : dirty.has(index)
                                  ? 'This mapping has unapplied changes.'
                                  : `Mapping ${index + 1} is synchronized with the project.`}
                        </small>
                        <ApplyButton
                            {saving}
                            pending={dirty.has(index)}
                            label="Apply all project MIDI app control changes" />
                    </div>
                </form>
            {/each}
        </div>
    {/if}
</section>
