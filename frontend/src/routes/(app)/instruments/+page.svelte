<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import BypassToggle from '$lib/components/BypassToggle.svelte';
    import Button from '$lib/components/Button.svelte';
    import DotToggle from '$lib/components/DotToggle.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import {
        MidiFilterKind,
        type MidiFilter,
        type MidiKeyMapping,
        type MidiKeySelection,
        type MidiKeyTransform,
        type MidiVelocityTransform
    } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { subscribeMidiEvents } from '$lib/state/midi-events.svelte';
    import {
        addInstrument,
        fetchInstruments,
        fetchInstrumentSlots,
        instrumentSlots,
        removeInstrument,
        setInstrumentCcFilter,
        setInstrumentChannelFilter,
        setInstrumentEnabled,
        setInstrumentIgnoreChannelAftertouch,
        setInstrumentIgnoreKeyTransform,
        setInstrumentIgnorePitchBend,
        setInstrumentIgnorePolyphonicAftertouch,
        setInstrumentIgnoreProgramChange,
        setInstrumentIgnoreVelocityTransform,
        setInstrumentKeyTransform,
        setInstrumentName,
        setInstrumentSourceFilter,
        setInstrumentSynth,
        setInstrumentUsesGlobalTransposition,
        setInstrumentVelocityTransform
    } from '$lib/state/instruments.svelte';
    import {
        fetchSynths,
        synths,
        SynthKind,
        type ControllerSlot,
        type SynthEntry
    } from '$lib/state/synths.svelte';
    import { errorToUserMessage } from '$lib/error';

    let {
        instrumentId = null,
        embedded = false,
        showAddButton = true
    } = $props<{
        instrumentId?: number | null;
        embedded?: boolean;
        showAddButton?: boolean;
    }>();

    const midiChannels = Array.from({ length: 16 }, (_, channel) => channel);
    const midiSources = Array.from({ length: 16 }, (_, source) => source);
    const midiControllerNames: Record<number, string> = {
        0: 'Bank select',
        1: 'Modulation',
        2: 'Breath',
        4: 'Foot controller',
        5: 'Portamento time',
        6: 'Data entry',
        7: 'Volume',
        8: 'Balance',
        10: 'Pan',
        11: 'Expression',
        12: 'Effect control 1',
        13: 'Effect control 2',
        16: 'General purpose 1',
        17: 'General purpose 2',
        18: 'General purpose 3',
        19: 'General purpose 4',
        32: 'Bank select LSB',
        33: 'Modulation LSB',
        34: 'Breath LSB',
        36: 'Foot controller LSB',
        37: 'Portamento time LSB',
        38: 'Data entry LSB',
        39: 'Volume LSB',
        40: 'Balance LSB',
        42: 'Pan LSB',
        43: 'Expression LSB',
        44: 'Effect control 1 LSB',
        45: 'Effect control 2 LSB',
        48: 'General purpose 1 LSB',
        49: 'General purpose 2 LSB',
        50: 'General purpose 3 LSB',
        51: 'General purpose 4 LSB',
        64: 'Sustain pedal',
        65: 'Portamento',
        66: 'Sostenuto',
        67: 'Soft pedal',
        68: 'Legato',
        69: 'Hold 2',
        70: 'Sound variation',
        71: 'Resonance',
        72: 'Release time',
        73: 'Attack time',
        74: 'Brightness',
        75: 'Decay time',
        76: 'Vibrato rate',
        77: 'Vibrato depth',
        78: 'Vibrato delay',
        79: 'Sound control 10',
        80: 'General purpose 5',
        81: 'General purpose 6',
        82: 'General purpose 7',
        83: 'General purpose 8',
        84: 'Portamento control',
        88: 'High-resolution velocity',
        91: 'Reverb level',
        92: 'Tremolo level',
        93: 'Chorus level',
        94: 'Celeste level',
        95: 'Phaser level',
        96: 'Data increment',
        97: 'Data decrement',
        98: 'NRPN LSB',
        99: 'NRPN MSB',
        100: 'RPN LSB',
        101: 'RPN MSB',
        120: 'All sound off',
        121: 'Reset controllers',
        122: 'Local control',
        123: 'All notes off',
        124: 'Omni mode off',
        125: 'Omni mode on',
        126: 'Mono mode',
        127: 'Poly mode'
    };
    const midiControllers = Array.from({ length: 128 }, (_, id) => ({
        id,
        name: midiControllerNames[id] ?? 'Undefined'
    }));

    let selectedId = $state(null as number | null);
    let inspectorOpen = $state(false);
    let loading = $state(true);
    let busy = $state(false);
    let addingInstrument = $state(false);
    let removingInstrument = $state(false);
    let togglingId = $state(null as number | null);
    let reloading = $state(false);
    let reloadDone = $state(false);

    let editName = $state('');
    let selectedSynth = $state('');
    let useGlobalTransposition = $state(true);
    let keySelectionType = $state<MidiKeySelection['type']>('All');
    let keyRangeMin = $state(0);
    let keyRangeMax = $state(127);
    let selectedKeys = $state('');
    let keyMappingType = $state<MidiKeyMapping['type']>('Identity');
    let keyMapping = $state('');
    let localTransposition = $state(0);
    let ignoreKeyTransform = $state(false);
    let velocityType = $state<MidiVelocityTransform['type']>('Unit');
    let velocityMin = $state(0);
    let velocityMax = $state(127);
    let ignoreVelocityTransform = $state(false);
    let sourceFilterType = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    let sources = $state<number[]>([]);
    let channelFilterType = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    let channels = $state<number[]>([]);
    let ccFilterType = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    let controllers = $state<number[]>([]);
    let ignorePitchBend = $state(false);
    let ignoreProgramChange = $state(false);
    let ignoreChannelAftertouch = $state(false);
    let ignorePolyphonicAftertouch = $state(false);
    let controllerQuery = $state('');
    let learningController = $state(false);
    let recordingKeySelection = $state<'Range' | 'Keys' | null>(null);
    let firstRecordedRangeKey = $state<number | null>(null);
    let recordingKeyMapping = $state(false);
    let firstRecordedMappingKey = $state<number | null>(null);
    let queuedOperations = 0;
    let operationQueue: Promise<void> = Promise.resolve();
    let savingSections = $state(new Set<string>());
    const savingSectionCounts = new Map<string, number>();

    let entries = $derived(
        Object.entries(instrumentSlots())
            .map(([id, slot]) => ({ id: Number(id), slot }))
            .filter(
                (entry) =>
                    !embedded || instrumentId === null || entry.slot.instrument_id === instrumentId
            )
            .sort((left, right) => left.id - right.id)
    );
    let selectedSlot = $derived(
        selectedId === null ? undefined : instrumentSlots()[String(selectedId)]
    );
    let availableSynths = $derived(
        [...synths()].sort(
            (left, right) => left.kind.localeCompare(right.kind) || left.id - right.id
        )
    );
    let nameChanged = $derived(Boolean(selectedSlot && editName.trim() !== selectedSlot.name));
    let synthChanged = $derived(
        Boolean(selectedSlot && selectedSynth !== synthKey(selectedSlot.synth_id))
    );
    let behaviorChanged = $derived(
        Boolean(selectedSlot && useGlobalTransposition !== selectedSlot.use_global_transposition)
    );
    let keyTransformChanged = $derived.by(() => {
        if (!selectedSlot) {
            return false;
        }
        const transform = selectedSlot.key_transform;
        return (
            keySelectionType !== transform.selection.type ||
            (transform.selection.type === 'Range' &&
                (keyRangeMin !== transform.selection.data.min ||
                    keyRangeMax !== transform.selection.data.max)) ||
            (transform.selection.type === 'Keys' &&
                selectedKeys !== formatList(transform.selection.data.keys)) ||
            keyMappingType !== transform.mapping.type ||
            keyMapping !== formatMapping(transform.mapping) ||
            localTransposition !== transform.transposition
        );
    });
    let ignoreKeyTransformChanged = $derived(
        Boolean(selectedSlot && ignoreKeyTransform !== selectedSlot.ignore_key_transform)
    );
    let velocityChanged = $derived.by(() => {
        if (!selectedSlot) {
            return false;
        }
        const transform = selectedSlot.velocity_transform;
        return (
            velocityType !== transform.type ||
            (transform.type === 'Linear' &&
                (velocityMin !== transform.data.min || velocityMax !== transform.data.max))
        );
    });
    let ignoreVelocityTransformChanged = $derived(
        Boolean(selectedSlot && ignoreVelocityTransform !== selectedSlot.ignore_velocity_transform)
    );
    let channelsChanged = $derived(
        Boolean(
            selectedSlot &&
                (channelFilterType !== selectedSlot.channel_filter.type ||
                    JSON.stringify([...channels].sort((left, right) => left - right)) !==
                        JSON.stringify(
                            [...selectedSlot.channel_filter.list].sort(
                                (left, right) => left - right
                            )
                        ))
        )
    );
    let sourcesChanged = $derived(
        Boolean(
            selectedSlot &&
                (sourceFilterType !== selectedSlot.source_filter.type ||
                    JSON.stringify([...sources].sort((left, right) => left - right)) !==
                        JSON.stringify(
                            [...selectedSlot.source_filter.list].sort((left, right) => left - right)
                        ))
        )
    );
    let controllersChanged = $derived(
        Boolean(
            selectedSlot &&
                (ccFilterType !== selectedSlot.cc_filter.type ||
                    JSON.stringify([...controllers].sort((left, right) => left - right)) !==
                        JSON.stringify(
                            [...selectedSlot.cc_filter.list].sort((left, right) => left - right)
                        ))
        )
    );
    let ignoredMessagesChanged = $derived(
        Boolean(
            selectedSlot &&
                (ignorePitchBend !== selectedSlot.ignore_pitch_bend ||
                    ignoreProgramChange !== selectedSlot.ignore_program_change ||
                    ignoreChannelAftertouch !== selectedSlot.ignore_channel_aftertouch ||
                    ignorePolyphonicAftertouch !== selectedSlot.ignore_polyphonic_aftertouch)
        )
    );
    let availableControllers = $derived(
        midiControllers.filter((controller) => !controllers.includes(controller.id))
    );
    let controllerToAdd = $derived.by(() => {
        const query = controllerQuery.trim();
        if (!query) {
            return undefined;
        }
        const numeric = Number(query);
        if (Number.isInteger(numeric) && availableControllers.some(({ id }) => id === numeric)) {
            return numeric;
        }
        return undefined;
    });
    let selectedControllerHint = $derived.by(() => {
        if (controllerToAdd === undefined) {
            return undefined;
        }
        return availableControllers.find((controller) => controller.id === controllerToAdd);
    });
    let addedControllerHint = $derived.by(() => {
        const query = controllerQuery.trim().toLocaleLowerCase();
        if (!query || selectedControllerHint) {
            return undefined;
        }
        const numeric = Number(query);
        if (Number.isInteger(numeric)) {
            return midiControllers.find(
                (controller) => controller.id === numeric && controllers.includes(controller.id)
            );
        }
        return midiControllers.find(
            (controller) =>
                controllers.includes(controller.id) &&
                controller.name.toLocaleLowerCase().includes(query)
        );
    });

    onMount(async () => {
        try {
            await Promise.all([fetchInstruments(), fetchSynths()]);
            const requestedInstrument =
                instrumentId === null
                    ? new URL(window.location.href).searchParams.get('instrument')
                    : String(instrumentId);
            const initial =
                (requestedInstrument === null
                    ? undefined
                    : entries.find((entry) =>
                          embedded
                              ? entry.slot.instrument_id === Number(requestedInstrument)
                              : entry.id === Number(requestedInstrument)
                      )) ?? entries[0];
            if (initial) {
                selectInstrument(initial.id, embedded || requestedInstrument !== null);
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause));
        } finally {
            loading = false;
        }
    });

    onMount(() =>
        subscribeMidiEvents(({ event }) => {
            if (event.type === 'NoteOn' && event.data.velocity > 0) {
                recordKey(event.data.key);
            }
            if (learningController && event.type === 'ControlChange') {
                controllerQuery = String(event.data.controller);
                learningController = false;
                addController();
            }
        })
    );

    function displayName(slot: ControllerSlot) {
        const name = slot.name.trim();
        return name ? `MIDI Input: ${name}` : 'MIDI Input';
    }

    function synthKey(id: number | null) {
        return id === null ? '' : String(id);
    }

    function synthEntry(id: number | null) {
        if (id === null) {
            return undefined;
        }
        return availableSynths.find((entry) => entry.id === id);
    }

    function synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `${entry.kind} instrument #${entry.id + 1}` : 'Instrument')
        );
    }

    function synthColor(id: number | null) {
        switch (synthEntry(id)?.kind) {
            case SynthKind.Fluid:
                return 'var(--color-synth-fluid)';
            case SynthKind.Sfizz:
                return 'var(--color-synth-sfizz)';
            case SynthKind.Dummy:
                return 'var(--color-synth-dummy)';
            default:
                return 'var(--color-synth-unassigned)';
        }
    }

    function formatList(values: number[], offset = 0) {
        return [...values]
            .sort((left, right) => left - right)
            .map((value) => value + offset)
            .join(', ');
    }

    function formatMapping(mapping: MidiKeyMapping) {
        if (mapping.type !== 'Custom') {
            return '';
        }
        return Object.entries(mapping.data.map)
            .sort(([left], [right]) => Number(left) - Number(right))
            .map(([input, output]) => `${input}:${output}`)
            .join(', ');
    }

    function selectInstrument(id: number, open = true) {
        const slot = instrumentSlots()[String(id)];
        if (!slot) {
            return;
        }
        stopKeyRecording();
        selectedId = id;
        editName = slot.name;
        selectedSynth = synthKey(slot.synth_id);
        useGlobalTransposition = slot.use_global_transposition;
        keySelectionType = slot.key_transform.selection.type;
        keyRangeMin =
            slot.key_transform.selection.type === 'Range'
                ? slot.key_transform.selection.data.min
                : 0;
        keyRangeMax =
            slot.key_transform.selection.type === 'Range'
                ? slot.key_transform.selection.data.max
                : 127;
        selectedKeys =
            slot.key_transform.selection.type === 'Keys'
                ? formatList(slot.key_transform.selection.data.keys)
                : '';
        keyMappingType = slot.key_transform.mapping.type;
        keyMapping = formatMapping(slot.key_transform.mapping);
        localTransposition = slot.key_transform.transposition;
        ignoreKeyTransform = slot.ignore_key_transform;
        velocityType = slot.velocity_transform.type;
        velocityMin =
            slot.velocity_transform.type === 'Linear' ? slot.velocity_transform.data.min : 0;
        velocityMax =
            slot.velocity_transform.type === 'Linear' ? slot.velocity_transform.data.max : 127;
        ignoreVelocityTransform = slot.ignore_velocity_transform;
        sourceFilterType = slot.source_filter.type;
        sources = [...slot.source_filter.list];
        channelFilterType = slot.channel_filter.type;
        channels = [...slot.channel_filter.list];
        ccFilterType = slot.cc_filter.type;
        controllers = [...slot.cc_filter.list];
        ignorePitchBend = slot.ignore_pitch_bend;
        ignoreProgramChange = slot.ignore_program_change;
        ignoreChannelAftertouch = slot.ignore_channel_aftertouch;
        ignorePolyphonicAftertouch = slot.ignore_polyphonic_aftertouch;
        controllerQuery = '';
        inspectorOpen = open;
    }

    function parseList(value: string, min: number, max: number, offset = 0) {
        if (!value.trim()) {
            return [];
        }
        const values = value
            .split(/[\s,]+/)
            .filter(Boolean)
            .map(Number);
        if (values.some((item) => !Number.isInteger(item) || item < min || item > max)) {
            throw new Error(`Enter whole numbers from ${min} to ${max}, separated by commas`);
        }
        return [...new Set(values.map((item) => item + offset))];
    }

    function parseMapping(value: string) {
        const map: Record<string, number> = {};
        if (!value.trim()) {
            return map;
        }
        for (const pair of value.split(/[\s,]+/).filter(Boolean)) {
            const [inputText, outputText, extra] = pair.split(':');
            const input = Number(inputText);
            const output = Number(outputText);
            if (
                extra !== undefined ||
                !Number.isInteger(input) ||
                !Number.isInteger(output) ||
                input < 0 ||
                input > 127 ||
                output < 0 ||
                output > 127
            ) {
                throw new Error('Use input:output pairs from 0 to 127, separated by commas');
            }
            map[String(input)] = output;
        }
        return map;
    }

    async function run(title: string, operation: () => Promise<unknown>, section?: string) {
        queuedOperations += 1;
        busy = true;
        if (section) {
            savingSectionCounts.set(section, (savingSectionCounts.get(section) ?? 0) + 1);
            savingSections = new Set(savingSections).add(section);
        }
        const execute = async () => {
            try {
                await operation();
                return true;
            } catch (cause) {
                notify.error(errorToUserMessage(cause), { title });
                return false;
            }
        };
        const result = operationQueue.then(execute, execute);
        operationQueue = result.then(() => undefined);
        try {
            return await result;
        } finally {
            queuedOperations -= 1;
            busy = queuedOperations > 0;
            if (section) {
                const remaining = (savingSectionCounts.get(section) ?? 1) - 1;
                if (remaining > 0) {
                    savingSectionCounts.set(section, remaining);
                } else {
                    savingSectionCounts.delete(section);
                    const next = new Set(savingSections);
                    next.delete(section);
                    savingSections = next;
                }
            }
        }
    }

    async function appendInstrument() {
        addingInstrument = true;
        let id: number | undefined;
        try {
            const succeeded = await run('Add controller', async () => {
                id = await addInstrument(instrumentId ?? undefined);
            });
            if (succeeded && id !== undefined) {
                selectInstrument(id);
            }
        } finally {
            addingInstrument = false;
        }
    }

    async function reloadInstruments() {
        if (reloading) {
            return;
        }
        reloading = true;
        reloadDone = false;
        try {
            await Promise.all([fetchInstrumentSlots(), fetchSynths()]);
            reloadDone = true;
            setTimeout(() => {
                reloadDone = false;
            }, 1200);
            if (selectedId !== null && !instrumentSlots()[String(selectedId)]) {
                const first = entries[0];
                selectedId = null;
                inspectorOpen = false;
                if (first) {
                    selectInstrument(first.id, false);
                }
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Reload controllers' });
        } finally {
            reloading = false;
        }
    }

    async function toggleEnabled(id: number, enabled: boolean) {
        if (togglingId !== null) {
            return;
        }
        togglingId = id;
        try {
            await setInstrumentEnabled(id, enabled);
        } catch (cause) {
            notify.error(errorToUserMessage(cause), {
                title: enabled ? 'Enable controller' : 'Disable controller'
            });
        } finally {
            togglingId = null;
        }
    }

    async function saveAssignment() {
        if (selectedId === null || !synthChanged) {
            return;
        }
        const id = selectedId;
        const synthId: number | null = selectedSynth ? Number(selectedSynth) : null;
        await run('Assign synth', () => setInstrumentSynth(id, synthId), 'synth');
    }

    async function saveName() {
        if (selectedId === null || !editName.trim() || !nameChanged) {
            return;
        }
        const id = selectedId;
        const name = editName.trim();
        const succeeded = await run('Rename controller', () => setInstrumentName(id, name), 'name');
        if (succeeded && selectedId === id) {
            editName = name;
        }
    }

    async function saveTranspositionMode() {
        if (selectedId === null || !behaviorChanged) {
            return;
        }
        const id = selectedId;
        const enabled = useGlobalTransposition;
        await run(
            'Global transposition',
            () => setInstrumentUsesGlobalTransposition(id, enabled),
            'behavior'
        );
    }

    async function saveKeyTransform() {
        if (selectedId === null || !keyTransformChanged) {
            return;
        }
        const id = selectedId;
        try {
            let selection: MidiKeySelection;
            if (keySelectionType === 'Range') {
                if (keyRangeMin > keyRangeMax) {
                    throw new Error('The first key must not be higher than the last key');
                }
                selection = { type: 'Range', data: { min: keyRangeMin, max: keyRangeMax } };
            } else if (keySelectionType === 'Keys') {
                selection = { type: 'Keys', data: { keys: parseList(selectedKeys, 0, 127) } };
            } else {
                selection = { type: 'All' };
            }
            const mapping: MidiKeyMapping =
                keyMappingType === 'Custom'
                    ? { type: 'Custom', data: { map: parseMapping(keyMapping) } }
                    : { type: 'Identity' };
            const transform: MidiKeyTransform = {
                selection,
                mapping,
                transposition: localTransposition
            };
            const succeeded = await run(
                'Update key transform',
                () => setInstrumentKeyTransform(id, transform),
                'keys'
            );
            if (succeeded && selectedId === id) {
                selectedKeys = selection.type === 'Keys' ? formatList(selection.data.keys) : '';
                keyMapping = formatMapping(mapping);
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update key transform' });
        }
    }

    async function saveIgnoreKeyTransform() {
        if (selectedId === null || !ignoreKeyTransformChanged) {
            return;
        }
        const id = selectedId;
        const ignored = ignoreKeyTransform;
        const succeeded = await run(
            'Bypass key transform',
            () => setInstrumentIgnoreKeyTransform(id, ignored),
            'ignore-key-transform'
        );
        if (!succeeded && selectedId === id) {
            ignoreKeyTransform = !ignored;
        }
    }

    async function saveVelocityTransform() {
        if (selectedId === null || !velocityChanged) {
            return;
        }
        const id = selectedId;
        if (velocityType === 'Linear' && velocityMin > velocityMax) {
            const msg = 'Minimum velocity must not exceed maximum velocity';
            notify.error(msg, { title: 'Update velocity transform' });
            return;
        }
        const transform: MidiVelocityTransform =
            velocityType === 'Linear'
                ? { type: 'Linear', data: { min: velocityMin, max: velocityMax } }
                : { type: 'Unit' };
        await run(
            'Update velocity transform',
            () => setInstrumentVelocityTransform(id, transform),
            'velocity'
        );
    }

    async function saveIgnoreVelocityTransform() {
        if (selectedId === null || !ignoreVelocityTransformChanged) {
            return;
        }
        const id = selectedId;
        const ignored = ignoreVelocityTransform;
        const succeeded = await run(
            'Bypass velocity transform',
            () => setInstrumentIgnoreVelocityTransform(id, ignored),
            'ignore-velocity-transform'
        );
        if (!succeeded && selectedId === id) {
            ignoreVelocityTransform = !ignored;
        }
    }

    async function saveChannelFilter() {
        if (selectedId === null || !channelsChanged) {
            return;
        }
        const id = selectedId;
        try {
            const filter: MidiFilter = {
                type: channelFilterType,
                list: [...channels].sort((left, right) => left - right)
            };
            const succeeded = await run(
                'Update channel filter',
                () => setInstrumentChannelFilter(id, filter),
                'channels'
            );
            if (succeeded && selectedId === id) {
                channels = [...filter.list];
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update channel filter' });
        }
    }

    async function saveSourceFilter() {
        if (selectedId === null || !sourcesChanged) {
            return;
        }
        const id = selectedId;
        const filter: MidiFilter = {
            type: sourceFilterType,
            list: [...sources].sort((left, right) => left - right)
        };
        const succeeded = await run(
            'Update MIDI source filter',
            () => setInstrumentSourceFilter(id, filter),
            'sources'
        );
        if (succeeded && selectedId === id) {
            sources = [...filter.list];
        }
    }

    function toggleSource(source: number, checked: boolean) {
        if (checked) {
            sources = [...sources, source].sort((left, right) => left - right);
        } else {
            sources = sources.filter((item) => item !== source);
        }
    }

    function toggleChannel(channel: number, checked: boolean) {
        if (checked) {
            channels = [...channels, channel].sort((left, right) => left - right);
        } else {
            channels = channels.filter((item) => item !== channel);
        }
    }

    async function saveCcFilter() {
        if (selectedId === null || !controllersChanged) {
            return;
        }
        const id = selectedId;
        try {
            const filter: MidiFilter = {
                type: ccFilterType,
                list: [...controllers].sort((left, right) => left - right)
            };
            const succeeded = await run(
                'Update controller filter',
                () => setInstrumentCcFilter(id, filter),
                'controllers'
            );
            if (succeeded && selectedId === id) {
                controllers = [...filter.list];
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update controller filter' });
        }
    }

    async function saveIgnoredMessages() {
        if (selectedId === null || !selectedSlot || !ignoredMessagesChanged) {
            return;
        }
        const id = selectedId;
        const slot = selectedSlot;
        const pitchBend = ignorePitchBend;
        const programChange = ignoreProgramChange;
        const channelAftertouch = ignoreChannelAftertouch;
        const polyphonicAftertouch = ignorePolyphonicAftertouch;
        await run(
            'Update MIDI handling',
            async () => {
                const updates: Promise<void>[] = [];
                if (pitchBend !== slot.ignore_pitch_bend) {
                    updates.push(setInstrumentIgnorePitchBend(id, pitchBend));
                }
                if (programChange !== slot.ignore_program_change) {
                    updates.push(setInstrumentIgnoreProgramChange(id, programChange));
                }
                if (channelAftertouch !== slot.ignore_channel_aftertouch) {
                    updates.push(setInstrumentIgnoreChannelAftertouch(id, channelAftertouch));
                }
                if (polyphonicAftertouch !== slot.ignore_polyphonic_aftertouch) {
                    updates.push(setInstrumentIgnorePolyphonicAftertouch(id, polyphonicAftertouch));
                }
                await Promise.all(updates);
            },
            'ignored'
        );
    }

    function addController() {
        if (
            controllerToAdd === undefined ||
            !Number.isInteger(controllerToAdd) ||
            controllerToAdd < 0 ||
            controllerToAdd > 127 ||
            controllers.includes(controllerToAdd)
        ) {
            return;
        }
        controllers = [...controllers, controllerToAdd].sort((left, right) => left - right);
        controllerQuery = '';
    }

    function removeController(controller: number) {
        controllers = controllers.filter((item) => item !== controller);
    }

    function toggleControllerLearning() {
        if (learningController) {
            learningController = false;
            return;
        }
        learningController = true;
    }

    function stopKeyRecording() {
        recordingKeySelection = null;
        firstRecordedRangeKey = null;
        recordingKeyMapping = false;
        firstRecordedMappingKey = null;
    }

    function submitControlGroupOnEnter(form: HTMLFormElement) {
        const handleKeydown = (event: KeyboardEvent) => {
            const target = event.target;
            if (
                event.key !== 'Enter' ||
                !(target instanceof HTMLButtonElement) ||
                target.type === 'submit'
            ) {
                return;
            }
            event.preventDefault();
            form.requestSubmit();
        };
        form.addEventListener('keydown', handleKeydown);
        return {
            destroy() {
                form.removeEventListener('keydown', handleKeydown);
            }
        };
    }

    function toggleKeyRecording(mode: 'Range' | 'Keys') {
        if (recordingKeySelection === mode) {
            stopKeyRecording();
            return;
        }
        stopKeyRecording();
        recordingKeySelection = mode;
    }

    function toggleKeyMappingRecording() {
        if (recordingKeyMapping) {
            stopKeyRecording();
            return;
        }
        stopKeyRecording();
        recordingKeyMapping = true;
    }

    function recordKey(key: number) {
        if (recordingKeySelection === 'Range') {
            if (firstRecordedRangeKey === null) {
                firstRecordedRangeKey = key;
                keyRangeMin = key;
                keyRangeMax = key;
                return;
            }
            keyRangeMin = Math.min(firstRecordedRangeKey, key);
            keyRangeMax = Math.max(firstRecordedRangeKey, key);
            stopKeyRecording();
            return;
        }
        if (recordingKeySelection === 'Keys') {
            const keys = selectedKeys
                .split(',')
                .map((value) => Number(value.trim()))
                .filter((value) => Number.isInteger(value) && value >= 0 && value <= 127);
            selectedKeys = formatList([...new Set([...keys, key])]);
            return;
        }
        if (recordingKeyMapping) {
            if (firstRecordedMappingKey === null) {
                firstRecordedMappingKey = key;
                return;
            }
            const map: Record<string, number> = {};
            for (const pair of keyMapping.split(/[\s,]+/).filter(Boolean)) {
                const [inputText, outputText, extra] = pair.split(':');
                const input = Number(inputText);
                const output = Number(outputText);
                if (
                    extra === undefined &&
                    Number.isInteger(input) &&
                    Number.isInteger(output) &&
                    input >= 0 &&
                    input <= 127 &&
                    output >= 0 &&
                    output <= 127
                ) {
                    map[String(input)] = output;
                }
            }
            map[String(firstRecordedMappingKey)] = key;
            keyMapping = Object.entries(map)
                .sort(([left], [right]) => Number(left) - Number(right))
                .map(([input, output]) => `${input}:${output}`)
                .join(', ');
            firstRecordedMappingKey = null;
        }
    }

    async function deleteSelected() {
        if (selectedId === null || !selectedSlot) {
            return;
        }
        const id = selectedId;
        removingInstrument = true;
        try {
            const succeeded = await run('Remove controller', () => removeInstrument(id));
            if (succeeded) {
                selectedId = null;
                inspectorOpen = false;
                const first = entries[0];
                if (first) {
                    selectInstrument(first.id, false);
                }
            }
        } finally {
            removingInstrument = false;
        }
    }
</script>

<div
    class={embedded
        ? 'min-w-0'
        : 'grid min-w-0 gap-3 min-[56.25rem]:grid-cols-[minmax(18rem,0.8fr)_minmax(24rem,1.2fr)] min-[56.25rem]:items-start'}>
    {#if !embedded}<section
            class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
            <header
                class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
                <div class="flex items-baseline gap-2">
                    <h2 class="m-0 text-sm font-semibold">Controllers</h2>
                    <span class="text-foreground-muted text-xs">
                        {entries.filter(({ slot }) => slot.enabled).length} enabled · {entries.length}
                        total
                    </span>
                </div>
                <div class="flex items-center gap-1.5">
                    {#if embedded}<Button
                            variant="secondary"
                            size="sm"
                            square
                            class="size-8.5"
                            type="button"
                            aria-label="Add MIDI input controller"
                            busy={addingInstrument}
                            disabled={busy}
                            onclick={() => void appendInstrument()}>
                            <Icon
                                icon={addingInstrument ? 'lucide:loader-circle' : 'lucide:plus'}
                                width="1.125rem"
                                class={addingInstrument ? 'animate-spin' : ''} />
                        </Button>{/if}
                    <Button
                        variant="ghost"
                        size="sm"
                        square
                        class="size-8.5"
                        type="button"
                        aria-label="Reload controllers"
                        busy={reloading}
                        disabled={reloading}
                        onclick={() => void reloadInstruments()}>
                        <Icon
                            icon={loading || reloading
                                ? 'lucide:loader-circle'
                                : reloadDone
                                  ? 'lucide:check'
                                  : 'lucide:refresh-cw'}
                            width="1.0625rem"
                            class={loading || reloading
                                ? 'animate-spin'
                                : reloadDone
                                  ? 'text-success'
                                  : ''} />
                    </Button>
                </div>
            </header>

            {#if loading}
                <div class="text-foreground-muted grid min-h-52 place-items-center text-sm">
                    Loading controllers…
                </div>
            {:else if entries.length === 0}
                <div class="text-foreground-muted grid min-h-52 place-items-center p-6 text-center">
                    <div>
                        <Icon
                            icon="lucide:sliders-horizontal"
                            width="1.5rem"
                            class="mx-auto mb-2" />
                        <p class="m-0 text-sm">No controllers are configured.</p>
                    </div>
                </div>
            {:else}
                <div class="grid gap-1.5 p-1.5">
                    {#each entries as entry (entry.id)}
                        {@const assignedSynth = synthEntry(entry.slot.synth_id)}
                        <article
                            class={`bg-surface-item grid overflow-hidden rounded-lg border transition-colors ${entry.slot.synth_id === null ? 'grid-cols-[minmax(0,1fr)_auto]' : 'grid-cols-[minmax(0,1fr)_auto_auto] min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(7.5rem,11rem)_auto]'} ${entry.id === selectedId ? 'border-accent/45' : 'border-transparent'}`}>
                            <button
                                class="hover:bg-surface-hover grid min-w-0 cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-3 border-0 bg-transparent px-3 py-3 text-left transition-colors"
                                type="button"
                                onclick={() => selectInstrument(entry.id)}>
                                <span
                                    class="text-foreground-muted grid size-8 place-items-center rounded-lg font-mono text-sm"
                                    style:background={`${synthColor(entry.slot.synth_id)}18`}>
                                    {entry.slot.instrument_id + 1}.{entry.slot.controller_id + 1}
                                </span>
                                <span class="min-w-0">
                                    <strong
                                        class="text-foreground block truncate text-sm font-semibold">
                                        {displayName(entry.slot)}
                                    </strong>
                                    <small
                                        class={`text-foreground-muted mt-0.5 block truncate text-sm ${entry.slot.synth_id === null ? '' : 'min-[37.5rem]:hidden'}`}>
                                        {synthName(assignedSynth)}
                                    </small>
                                </span>
                            </button>
                            {#if entry.slot.synth_id !== null}
                                <a
                                    class="text-foreground-muted hover:border-accent/35 hover:bg-surface-hover hover:text-accent min-[37.5rem]:border-border my-auto grid size-9 place-items-center rounded-lg border border-transparent transition-colors min-[37.5rem]:my-0 min-[37.5rem]:flex min-[37.5rem]:size-auto min-[37.5rem]:min-w-0 min-[37.5rem]:items-center min-[37.5rem]:gap-2 min-[37.5rem]:rounded-none min-[37.5rem]:border-0 min-[37.5rem]:border-l min-[37.5rem]:px-3"
                                    href={`/synths?synth=${entry.slot.synth_id}`}
                                    aria-label={`Open ${synthName(assignedSynth)}`}
                                    title="Open assigned synth">
                                    <span
                                        class="hidden min-w-0 flex-1 truncate text-xs min-[37.5rem]:block">
                                        {synthName(assignedSynth)}
                                    </span>
                                    <Icon
                                        class="shrink-0"
                                        icon="lucide:arrow-up-right"
                                        width="1rem" />
                                </a>
                            {/if}
                            <button
                                class={`mr-2 grid size-9 cursor-pointer place-items-center self-center rounded-lg border transition-colors ${entry.slot.enabled ? 'border-accent/60 bg-accent/20 text-accent' : 'border-border-strong bg-surface-button text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}
                                type="button"
                                aria-label={`${entry.slot.enabled ? 'Disable' : 'Enable'} ${displayName(entry.slot)}`}
                                disabled={togglingId !== null}
                                onclick={() => void toggleEnabled(entry.id, !entry.slot.enabled)}>
                                <Icon
                                    icon={togglingId === entry.id
                                        ? 'lucide:loader-circle'
                                        : 'lucide:power'}
                                    width="1rem"
                                    class={togglingId === entry.id ? 'animate-spin' : ''} />
                            </button>
                        </article>
                    {/each}
                </div>
            {/if}
        </section>{/if}

    {#if !embedded && inspectorOpen}<button
            class="bg-scrim/60 fixed inset-0 z-30 cursor-default border-0 min-[56.25rem]:hidden"
            type="button"
            aria-label="Close controller editor"
            onclick={() => (inspectorOpen = false)}>
        </button>{/if}

    {#if embedded && !loading && entries.length === 0}
        <section
            class="border-border bg-surface-card grid justify-items-center gap-3 rounded-xl border p-6 text-center">
            <div>
                <h3 class="text-foreground m-0 text-sm font-semibold">No controllers</h3>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Add a MIDI input controller to route MIDI events into this instrument.
                </p>
            </div>
            {#if showAddButton}
                <Button
                    variant="primary"
                    size="sm"
                    type="button"
                    busy={addingInstrument}
                    disabled={busy}
                    onclick={() => void appendInstrument()}>
                    <Icon icon="lucide:plus" width="1rem" />
                    Add MIDI input
                </Button>
            {/if}
        </section>
    {/if}

    {#if selectedSlot && selectedId !== null}
        <aside
            class={embedded
                ? 'border-border bg-surface-card overflow-clip rounded-xl border'
                : `border-border bg-surface-card fixed right-0 bottom-0 left-0 z-40 max-h-[calc(100dvh-4rem)] overflow-clip rounded-t-2xl border transition-transform min-[56.25rem]:sticky min-[56.25rem]:top-20.5 min-[56.25rem]:z-0 min-[56.25rem]:max-h-[calc(100dvh-8.5rem)] min-[56.25rem]:translate-y-0 min-[56.25rem]:rounded-xl ${inspectorOpen ? 'translate-y-0' : 'translate-y-full'}`}>
            <header
                class="border-border flex min-h-15 items-center justify-between gap-3 border-b px-4 py-2.5">
                <div class="min-w-0">
                    <strong class="text-foreground block truncate text-base">
                        Controller #{selectedSlot.controller_id + 1}
                    </strong>
                    <small class="text-foreground-muted block truncate text-xs">
                        {displayName(selectedSlot)}
                    </small>
                </div>
                {#if embedded}<div class="flex flex-wrap items-center justify-end gap-1.5">
                        {#if entries.length > 1}<SelectDropdown
                                compact
                                value={String(selectedId)}
                                options={entries.map((entry) => ({
                                    value: String(entry.id),
                                    label: displayName(entry.slot),
                                    description: `Controller #${entry.slot.controller_id + 1}`
                                }))}
                                onchange={(value) => selectInstrument(Number(value), true)}
                                ariaLabel="Selected MIDI input controller" />{/if}
                        <Button
                            variant={selectedSlot.enabled ? 'accent-outline' : 'secondary'}
                            size="sm"
                            type="button"
                            aria-label={`${selectedSlot.enabled ? 'Disable' : 'Enable'} ${displayName(selectedSlot)}`}
                            aria-pressed={selectedSlot.enabled}
                            busy={togglingId === selectedId}
                            disabled={togglingId !== null}
                            onclick={() => void toggleEnabled(selectedId!, !selectedSlot.enabled)}>
                            <Icon
                                icon={togglingId === selectedId
                                    ? 'lucide:loader-circle'
                                    : 'lucide:power'}
                                width="0.875rem"
                                class={togglingId === selectedId ? 'animate-spin' : ''} />
                            {selectedSlot.enabled ? 'Enabled' : 'Disabled'}
                        </Button>
                        {#if showAddButton}
                            <Button
                                variant="secondary"
                                size="sm"
                                square
                                class="size-8.5"
                                type="button"
                                aria-label="Add MIDI input controller"
                                busy={addingInstrument}
                                disabled={busy}
                                onclick={() => void appendInstrument()}>
                                <Icon
                                    icon={addingInstrument ? 'lucide:loader-circle' : 'lucide:plus'}
                                    width="1.125rem"
                                    class={addingInstrument ? 'animate-spin' : ''} />
                            </Button>
                        {/if}
                        <TimedConfirmButton
                            label="Remove"
                            confirmLabel="Confirm remove"
                            busyLabel="Removing"
                            busy={removingInstrument}
                            disabled={busy}
                            onconfirm={deleteSelected} />
                    </div>{:else}<Button
                        variant="ghost"
                        size="md"
                        square
                        class="bg-surface-button min-[56.25rem]:hidden"
                        type="button"
                        aria-label="Close instrument editor"
                        onclick={() => (inspectorOpen = false)}>
                        <Icon icon="lucide:x" width="1.125rem" />
                    </Button>{/if}
            </header>

            <div
                class={embedded
                    ? ''
                    : 'max-h-[calc(100dvh-7.75rem)] overflow-y-auto pb-[max(1rem,env(safe-area-inset-bottom))] min-[56.25rem]:max-h-[calc(100dvh-12.25rem)] min-[56.25rem]:pb-0'}>
                <form
                    class="border-border grid gap-2 border-b p-4"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveName();
                    }}>
                    <label
                        class="text-foreground-strong text-xs font-semibold uppercase"
                        for="instrument-name">
                        Name
                    </label>
                    <AutoSaveTextInput
                        id="instrument-name"
                        bind:value={editName}
                        pending={nameChanged}
                        saving={savingSections.has('name')}
                        showIndicator={false}
                        required
                        oncommit={saveName} />
                </form>

                <section class="border-border grid gap-3 border-b p-4">
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        Behavior
                    </h3>
                    <DotToggle
                        class="min-w-0 justify-between px-3"
                        label="Use global transposition"
                        indicatorEnd
                        bind:checked={useGlobalTransposition}
                        onchange={() => void saveTranspositionMode()} />
                </section>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveSourceFilter();
                    }}>
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        MIDI input sources
                    </h3>
                    <div class="grid gap-2">
                        <div class="grid min-w-0 gap-4">
                            <SegmentedControl
                                name="midi-source-filter-mode"
                                ariaLabel="MIDI input source filter mode"
                                compact
                                bind:value={sourceFilterType}
                                options={[
                                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                    {
                                        value: MidiFilterKind.Denylist,
                                        label: 'Allow all except'
                                    }
                                ]} />
                            <div class="grid gap-1.5">
                                <div class="grid grid-cols-8 gap-1">
                                    {#each midiSources as source}<label
                                            class={`has-focus-visible:outline-accent grid min-h-9 cursor-pointer place-items-center rounded-md border font-mono text-xs font-semibold transition-colors has-focus-visible:outline-2 has-focus-visible:outline-offset-2 ${sources.includes(source) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-inset text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}>
                                            <input
                                                class="sr-only"
                                                type="checkbox"
                                                checked={sources.includes(source)}
                                                onchange={(event) =>
                                                    toggleSource(
                                                        source,
                                                        event.currentTarget.checked
                                                    )} />
                                            <span>{source + 1}</span>
                                        </label>{/each}
                                </div>
                                <div class="flex justify-end gap-1.5">
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={sources.length === midiSources.length}
                                        onclick={() => {
                                            sources = [...midiSources];
                                        }}>
                                        All
                                    </Button>
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={sources.length === 0}
                                        onclick={() => {
                                            sources = [];
                                        }}>
                                        None
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <ApplyButton
                            saving={savingSections.has('sources')}
                            pending={sourcesChanged} />
                    </div>
                    <small class="text-foreground-muted">
                        Checked input slots are {sourceFilterType === MidiFilterKind.Allowlist
                            ? 'allowed'
                            : 'blocked'}.
                    </small>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveKeyTransform();
                    }}>
                    <div>
                        <div class="flex items-center justify-between gap-3">
                            <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                                Keys
                            </h3>
                            <BypassToggle
                                bind:checked={ignoreKeyTransform}
                                saving={savingSections.has('ignore-key-transform')}
                                onchange={() => void saveIgnoreKeyTransform()} />
                        </div>
                    </div>
                    <div class="grid gap-2">
                        <div class="grid min-w-0 gap-4">
                            <div class="grid grid-cols-2 gap-2">
                                <div class="grid gap-1 text-sm">
                                    <span class="text-foreground-muted">Selection</span>
                                    <SegmentedControl
                                        name="key-selection"
                                        ariaLabel="Key selection"
                                        bind:value={keySelectionType}
                                        options={[
                                            { value: 'All', label: 'All' },
                                            { value: 'Range', label: 'Range' },
                                            { value: 'Keys', label: 'Specific' }
                                        ]}
                                        onchange={stopKeyRecording} />
                                </div>
                                <div class="grid gap-1 text-sm">
                                    <span class="text-foreground-muted">Mapping</span>
                                    <SegmentedControl
                                        name="key-mapping"
                                        ariaLabel="Key mapping"
                                        bind:value={keyMappingType}
                                        options={[
                                            { value: 'Identity', label: 'Identity' },
                                            { value: 'Custom', label: 'Custom' }
                                        ]}
                                        onchange={stopKeyRecording} />
                                </div>
                            </div>
                            {#if keySelectionType === 'Range'}
                                <div
                                    class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_2.5rem] items-end gap-2">
                                    <label class="grid gap-1 text-sm">
                                        <span class="text-foreground-muted">First key</span>
                                        <input
                                            class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5 font-mono"
                                            type="number"
                                            min="0"
                                            max="127"
                                            bind:value={keyRangeMin} />
                                    </label>
                                    <label class="grid gap-1 text-sm">
                                        <span class="text-foreground-muted">Last key</span>
                                        <input
                                            class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5 font-mono"
                                            type="number"
                                            min="0"
                                            max="127"
                                            bind:value={keyRangeMax} />
                                    </label>
                                    <button
                                        class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${recordingKeySelection === 'Range' ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
                                        type="button"
                                        aria-label={recordingKeySelection === 'Range'
                                            ? 'Stop recording key range'
                                            : 'Record key range'}
                                        title={recordingKeySelection === 'Range'
                                            ? firstRecordedRangeKey === null
                                                ? 'Waiting for the first key'
                                                : 'Waiting for the second key'
                                            : 'Record two keys to define the range'}
                                        aria-pressed={recordingKeySelection === 'Range'}
                                        onclick={() => toggleKeyRecording('Range')}>
                                        <Icon
                                            icon={recordingKeySelection === 'Range'
                                                ? 'lucide:radio-tower'
                                                : 'lucide:circle-dot'}
                                            width="1rem" />
                                    </button>
                                </div>
                            {:else if keySelectionType === 'Keys'}
                                <div class="grid grid-cols-[minmax(0,1fr)_2.5rem] items-end gap-2">
                                    <label class="grid min-w-0 gap-1 text-sm">
                                        <span class="text-foreground-muted">MIDI key numbers</span>
                                        <input
                                            class="border-border-strong bg-surface-inset h-10 min-w-0 rounded-lg border px-2.5 font-mono"
                                            placeholder="36, 38, 42"
                                            bind:value={selectedKeys} />
                                    </label>
                                    <button
                                        class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${recordingKeySelection === 'Keys' ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
                                        type="button"
                                        aria-label={recordingKeySelection === 'Keys'
                                            ? 'Stop recording specific keys'
                                            : 'Record specific keys'}
                                        title={recordingKeySelection === 'Keys'
                                            ? 'Recording keys; press again to stop'
                                            : 'Record specific keys'}
                                        aria-pressed={recordingKeySelection === 'Keys'}
                                        onclick={() => toggleKeyRecording('Keys')}>
                                        <Icon
                                            icon={recordingKeySelection === 'Keys'
                                                ? 'lucide:radio-tower'
                                                : 'lucide:circle-dot'}
                                            width="1rem" />
                                    </button>
                                </div>
                            {/if}
                            {#if keyMappingType === 'Custom'}<div
                                    class="grid grid-cols-[minmax(0,1fr)_2.5rem] items-end gap-2">
                                    <label class="grid min-w-0 gap-1 text-sm">
                                        <span class="text-foreground-muted">
                                            Input:output pairs
                                        </span>
                                        <input
                                            class="border-border-strong bg-surface-inset h-10 min-w-0 rounded-lg border px-2.5 font-mono"
                                            placeholder="36:48, 38:50"
                                            bind:value={keyMapping} />
                                    </label>
                                    <button
                                        class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${recordingKeyMapping ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
                                        type="button"
                                        aria-label={recordingKeyMapping
                                            ? 'Stop recording key mappings'
                                            : 'Record key mappings'}
                                        title={recordingKeyMapping
                                            ? firstRecordedMappingKey === null
                                                ? 'Waiting for an input key'
                                                : 'Waiting for its output key'
                                            : 'Record input and output key pairs'}
                                        aria-pressed={recordingKeyMapping}
                                        onclick={toggleKeyMappingRecording}>
                                        <Icon
                                            icon={recordingKeyMapping
                                                ? 'lucide:radio-tower'
                                                : 'lucide:circle-dot'}
                                            width="1rem" />
                                    </button>
                                    {#if recordingKeyMapping}<small
                                            class="text-foreground-muted col-span-2 flex items-center gap-2 text-xs"
                                            aria-live="polite">
                                            <span
                                                class="text-accent font-mono font-semibold tabular-nums">
                                                {firstRecordedMappingKey === null
                                                    ? '… → …'
                                                    : `${firstRecordedMappingKey} → …`}
                                            </span>
                                            <span>
                                                {firstRecordedMappingKey === null
                                                    ? 'Play the input key'
                                                    : 'Now play the output key'}
                                            </span>
                                        </small>{/if}
                                </div>{/if}
                            <NumericControl
                                label="Local transposition"
                                value={localTransposition}
                                unit="semitones"
                                smallStep={1}
                                largeStep={12}
                                resetValue={0}
                                min={-128}
                                max={127}
                                wideValue
                                compact
                                onchange={(value) => {
                                    localTransposition = value;
                                }} />
                        </div>
                        <ApplyButton
                            saving={savingSections.has('keys')}
                            pending={keyTransformChanged} />
                    </div>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveVelocityTransform();
                    }}>
                    <div>
                        <div class="flex items-center justify-between gap-3">
                            <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                                Velocity
                            </h3>
                            <BypassToggle
                                bind:checked={ignoreVelocityTransform}
                                saving={savingSections.has('ignore-velocity-transform')}
                                onchange={() => void saveIgnoreVelocityTransform()} />
                        </div>
                    </div>
                    <div class="grid gap-2">
                        <div class="grid min-w-0 gap-4">
                            <SegmentedControl
                                name="velocity-transform"
                                ariaLabel="Velocity transform"
                                compact
                                bind:value={velocityType}
                                options={[
                                    { value: 'Unit', label: 'Unchanged' },
                                    { value: 'Linear', label: 'Linear range' }
                                ]} />
                            {#if velocityType === 'Linear'}<div
                                    class="grid grid-cols-[3rem_minmax(5rem,1fr)_3rem] items-center gap-2">
                                    <input
                                        class="border-border-strong bg-surface-inset h-9 min-w-0 appearance-none rounded-lg border px-1 text-center font-mono text-sm [&::-webkit-inner-spin-button]:appearance-none"
                                        type="number"
                                        aria-label="Minimum velocity"
                                        min="0"
                                        max={velocityMax}
                                        bind:value={velocityMin} />
                                    <div class="relative h-9 min-w-0">
                                        <div
                                            class="bg-surface-button absolute top-1/2 right-0 left-0 h-1 -translate-y-1/2 rounded-full">
                                        </div>
                                        <div
                                            class="bg-accent absolute top-1/2 h-1 -translate-y-1/2 rounded-full"
                                            style:left={`${(velocityMin / 127) * 100}%`}
                                            style:right={`${100 - (velocityMax / 127) * 100}%`}>
                                        </div>
                                        <input
                                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:-mt-1.5 [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
                                            type="range"
                                            aria-label="Minimum velocity"
                                            min="0"
                                            max="127"
                                            value={velocityMin}
                                            oninput={(event) => {
                                                const value = Math.min(
                                                    Number(event.currentTarget.value),
                                                    velocityMax
                                                );
                                                velocityMin = value;
                                                event.currentTarget.value = String(value);
                                            }} />
                                        <input
                                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:-mt-1.5 [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
                                            type="range"
                                            aria-label="Maximum velocity"
                                            min="0"
                                            max="127"
                                            value={velocityMax}
                                            oninput={(event) => {
                                                const value = Math.max(
                                                    Number(event.currentTarget.value),
                                                    velocityMin
                                                );
                                                velocityMax = value;
                                                event.currentTarget.value = String(value);
                                            }} />
                                    </div>
                                    <input
                                        class="border-border-strong bg-surface-inset h-9 min-w-0 appearance-none rounded-lg border px-1 text-center font-mono text-sm [&::-webkit-inner-spin-button]:appearance-none"
                                        type="number"
                                        aria-label="Maximum velocity"
                                        min={velocityMin}
                                        max="127"
                                        bind:value={velocityMax} />
                                </div>{/if}
                        </div>
                        <ApplyButton
                            saving={savingSections.has('velocity')}
                            pending={velocityChanged} />
                    </div>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveChannelFilter();
                    }}>
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        MIDI channels
                    </h3>
                    <div class="grid gap-2">
                        <div class="grid min-w-0 gap-4">
                            <SegmentedControl
                                name="midi-channel-filter-mode"
                                ariaLabel="MIDI channel filter mode"
                                compact
                                bind:value={channelFilterType}
                                options={[
                                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                    {
                                        value: MidiFilterKind.Denylist,
                                        label: 'Allow all except'
                                    }
                                ]} />
                            <div class="grid gap-1.5">
                                <div class="grid grid-cols-8 gap-1">
                                    {#each midiChannels as channel}<label
                                            class={`has-focus-visible:outline-accent grid min-h-9 cursor-pointer place-items-center rounded-md border font-mono text-xs font-semibold transition-colors has-focus-visible:outline-2 has-focus-visible:outline-offset-2 ${channels.includes(channel) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-inset text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}>
                                            <input
                                                class="sr-only"
                                                type="checkbox"
                                                checked={channels.includes(channel)}
                                                onchange={(event) =>
                                                    toggleChannel(
                                                        channel,
                                                        event.currentTarget.checked
                                                    )} />
                                            <span>{channel + 1}</span>
                                        </label>{/each}
                                </div>
                                <div class="flex justify-end gap-1.5">
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={channels.length === midiChannels.length}
                                        onclick={() => {
                                            channels = [...midiChannels];
                                        }}>
                                        All
                                    </Button>
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={channels.length === 0}
                                        onclick={() => {
                                            channels = [];
                                        }}>
                                        None
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <ApplyButton
                            saving={savingSections.has('channels')}
                            pending={channelsChanged} />
                    </div>
                    <small class="text-foreground-muted">
                        Checked channels are {channelFilterType === MidiFilterKind.Allowlist
                            ? 'allowed'
                            : 'blocked'}.
                    </small>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveCcFilter();
                    }}>
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        MIDI controllers
                    </h3>
                    <div class="grid gap-2">
                        <div class="grid min-w-0 gap-4">
                            <SegmentedControl
                                name="midi-controller-filter-mode"
                                ariaLabel="MIDI controller filter mode"
                                compact
                                bind:value={ccFilterType}
                                options={[
                                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                                    {
                                        value: MidiFilterKind.Denylist,
                                        label: 'Allow all except'
                                    }
                                ]} />
                            <div class="grid min-w-0 gap-1">
                                <div class="flex min-w-0 gap-1.5">
                                    <div class="relative min-w-0 flex-1">
                                        <SelectDropdown
                                            id="midi-controller-hints"
                                            bind:value={controllerQuery}
                                            options={midiControllers.map((controller) => ({
                                                value: String(controller.id),
                                                label: `CC ${controller.id}`,
                                                selectionLabel: `CC ${controller.id} · ${controller.name}`,
                                                description: controllers.includes(controller.id)
                                                    ? `${controller.name} · Already added`
                                                    : controller.name,
                                                disabled: controllers.includes(controller.id)
                                            }))}
                                            placeholder="Select controller"
                                            disabled={availableControllers.length === 0}
                                            compact
                                            clearable
                                            searchable
                                            searchPlaceholder="Search controllers"
                                            noMatchesText="No matching controllers"
                                            ariaLabel="MIDI controller to add"
                                            onchange={addController} />
                                    </div>
                                    <button
                                        class={`grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border-0 transition-colors ${learningController ? 'bg-danger/15 text-danger animate-pulse' : 'bg-surface-button text-foreground enabled:hover:bg-surface-hover'}`}
                                        type="button"
                                        aria-label={learningController
                                            ? 'Stop learning MIDI controller'
                                            : 'Learn MIDI controller'}
                                        title={learningController
                                            ? 'Waiting for a Control Change event'
                                            : 'Learn from the next Control Change event'}
                                        aria-pressed={learningController}
                                        onclick={toggleControllerLearning}>
                                        <Icon
                                            icon={learningController
                                                ? 'lucide:radio-tower'
                                                : 'lucide:circle-dot'}
                                            width="1rem" />
                                    </button>
                                </div>
                                {#if selectedControllerHint}<small
                                        class="text-foreground-muted truncate px-1 text-xs">
                                        CC {selectedControllerHint.id} · {selectedControllerHint.name}
                                    </small>{:else if addedControllerHint}<small
                                        class="text-foreground-muted truncate px-1 text-xs">
                                        Already added: CC {addedControllerHint.id} · {addedControllerHint.name}
                                    </small>{:else}<small
                                        class="text-foreground-subtle truncate px-1 text-xs">
                                        No matching controller
                                    </small>{/if}
                            </div>
                            <div class="grid gap-1.5">
                                <div
                                    class="border-border-strong bg-surface-inset flex max-h-32 min-h-10 flex-wrap content-start gap-1 overflow-y-auto rounded-lg border p-1.5">
                                    {#each controllers as controller (controller)}<span
                                            class="bg-surface-button text-foreground flex h-7 max-w-full items-center gap-1 rounded-md pl-2 text-xs">
                                            <span class="truncate">
                                                CC {controller} · {midiControllerNames[
                                                    controller
                                                ] ?? 'Undefined'}
                                            </span>
                                            <Button
                                                variant="danger-ghost"
                                                size="xs"
                                                square
                                                type="button"
                                                aria-label={`Remove CC ${controller}`}
                                                title={`Remove CC ${controller}`}
                                                onclick={() => removeController(controller)}>
                                                <Icon icon="lucide:x" width="0.875rem" />
                                            </Button>
                                        </span>{:else}<span
                                            class="text-foreground-muted self-center px-1 text-xs">
                                            No controllers selected
                                        </span>{/each}
                                </div>
                                <div class="flex justify-end gap-1.5">
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={controllers.length === midiControllers.length}
                                        onclick={() => {
                                            controllers = midiControllers.map(({ id }) => id);
                                        }}>
                                        All
                                    </Button>
                                    <Button
                                        variant="secondary"
                                        size="xs"
                                        class="text-foreground-muted"
                                        type="button"
                                        disabled={controllers.length === 0}
                                        onclick={() => {
                                            controllers = [];
                                        }}>
                                        None
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <ApplyButton
                            saving={savingSections.has('controllers')}
                            pending={controllersChanged} />
                    </div>
                    <small class="text-foreground-muted">
                        Selected controllers are {ccFilterType === MidiFilterKind.Allowlist
                            ? 'allowed'
                            : 'blocked'}.
                    </small>
                </form>

                <section class="border-border grid gap-5 border-b p-4">
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        Ignored MIDI messages
                    </h3>
                    <div class="grid min-w-0 grid-cols-2 gap-1.5">
                        <DotToggle
                            label="Pitch bend"
                            bind:checked={ignorePitchBend}
                            onchange={() => void saveIgnoredMessages()} />
                        <DotToggle
                            label="Program change"
                            bind:checked={ignoreProgramChange}
                            onchange={() => void saveIgnoredMessages()} />
                        <DotToggle
                            label="Channel aftertouch"
                            bind:checked={ignoreChannelAftertouch}
                            onchange={() => void saveIgnoredMessages()} />
                        <DotToggle
                            label="Poly aftertouch"
                            bind:checked={ignorePolyphonicAftertouch}
                            onchange={() => void saveIgnoredMessages()} />
                    </div>
                </section>

                {#if !embedded}
                    <section class="p-4">
                        <TimedConfirmButton
                            class="w-full"
                            label="Remove instrument"
                            confirmLabel="Confirm remove instrument"
                            busyLabel="Removing…"
                            busy={removingInstrument}
                            disabled={busy}
                            onconfirm={deleteSelected} />
                    </section>
                {/if}
            </div>
        </aside>
    {/if}
</div>
