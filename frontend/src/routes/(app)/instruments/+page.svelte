<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount, tick } from 'svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import {
        MidiFilterKind,
        type Instrument,
        type MidiFilter,
        type MidiKeyMapping,
        type MidiKeySelection,
        type MidiKeyTransform,
        type MidiVelocityTransform,
        type SynthId
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
        setInstrumentSynth,
        setInstrumentUsesGlobalTransposition,
        setInstrumentVelocityTransform
    } from '$lib/state/instruments.svelte';
    import { fetchSynths, synths, SynthKind, type SynthEntry } from '$lib/state/synths.svelte';

    const midiChannels = Array.from({ length: 16 }, (_, channel) => channel);
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
    let togglingId = $state(null as number | null);
    let reloading = $state(false);
    let reloadDone = $state(false);
    let removeConfirming = $state(false);
    let removeConfirmationElement = $state<HTMLDivElement>();
    let error = $state('');

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
    let channelFilterType = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    let channels = $state<number[]>([]);
    let ccFilterType = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    let controllers = $state<number[]>([]);
    let ignorePitchBend = $state(false);
    let ignoreProgramChange = $state(false);
    let ignoreChannelAftertouch = $state(false);
    let ignorePolyphonicAftertouch = $state(false);
    let controllerQuery = $state('');
    let controllerHintsOpen = $state(false);
    let activeControllerHint = $state(-1);
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
    let controllerHints = $derived(
        availableControllers.filter((controller) => {
            const query = controllerQuery.trim().toLocaleLowerCase();
            return (
                !query ||
                String(controller.id).includes(query) ||
                controller.name.toLocaleLowerCase().includes(query)
            );
        })
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
        return controllerHints[0]?.id;
    });
    let selectedControllerHint = $derived.by(() => {
        if (controllerToAdd === undefined) {
            return undefined;
        }
        return availableControllers.find((controller) => controller.id === controllerToAdd);
    });

    onMount(async () => {
        try {
            await Promise.all([fetchInstruments(), fetchSynths()]);
            const first = entries[0];
            if (first) {
                selectInstrument(first.id, false);
            }
        } catch (cause) {
            showError(cause);
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
                controllerHintsOpen = false;
                activeControllerHint = -1;
                learningController = false;
            }
        })
    );

    function showError(cause: unknown) {
        error = cause instanceof Error ? cause.message : String(cause);
    }

    function displayName(id: number, slot: Instrument) {
        return slot.name.trim() || `Unnamed instrument #${id + 1}`;
    }

    function synthKey(id: SynthId | null) {
        return id === null ? '' : String(id);
    }

    function synthEntry(id: SynthId | null) {
        if (id === null) {
            return undefined;
        }
        return availableSynths.find((entry) => entry.id === id);
    }

    function synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `Unnamed ${entry.kind.toLowerCase()} synth #${entry.id}` : 'Unassigned')
        );
    }

    function synthColor(id: SynthId | null) {
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
        channelFilterType = slot.channel_filter.type;
        channels = [...slot.channel_filter.list];
        ccFilterType = slot.cc_filter.type;
        controllers = [...slot.cc_filter.list];
        ignorePitchBend = slot.ignore_pitch_bend;
        ignoreProgramChange = slot.ignore_program_change;
        ignoreChannelAftertouch = slot.ignore_channel_aftertouch;
        ignorePolyphonicAftertouch = slot.ignore_polyphonic_aftertouch;
        controllerQuery = '';
        removeConfirming = false;
        error = '';
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

    async function run(
        title: string,
        success: string,
        operation: () => Promise<unknown>,
        section?: string
    ) {
        queuedOperations += 1;
        busy = true;
        if (section) {
            savingSectionCounts.set(section, (savingSectionCounts.get(section) ?? 0) + 1);
            savingSections = new Set(savingSections).add(section);
        }
        const execute = async () => {
            error = '';
            const automatic =
                section === 'name' ||
                section === 'synth' ||
                section === 'behavior' ||
                section === 'ignore-key-transform' ||
                section === 'ignore-velocity-transform';
            const notification = automatic
                ? undefined
                : notify.info('In progress…', {
                      duration: null,
                      loading: true,
                      title
                  });
            try {
                await operation();
                notification?.success(success);
                return true;
            } catch (cause) {
                showError(cause);
                if (notification) {
                    notification.error(error);
                } else {
                    notify.error(error, { title });
                }
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
        let id: number | undefined;
        const succeeded = await run('Add instrument', 'Instrument added.', async () => {
            id = await addInstrument();
        });
        if (succeeded && id !== undefined) {
            selectInstrument(id);
        }
    }

    async function reloadInstruments() {
        if (reloading) {
            return;
        }
        reloading = true;
        reloadDone = false;
        error = '';
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
            showError(cause);
            notify.error(error, { title: 'Reload instruments' });
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
            showError(cause);
            notify.error(error, { title: enabled ? 'Enable instrument' : 'Disable instrument' });
        } finally {
            togglingId = null;
        }
    }

    async function saveAssignment() {
        if (selectedId === null || !synthChanged) {
            return;
        }
        const id = selectedId;
        const synthId: SynthId | null = selectedSynth ? Number(selectedSynth) : null;
        await run(
            'Assign synth',
            'Synth assignment updated.',
            () => setInstrumentSynth(id, synthId),
            'synth'
        );
    }

    async function saveName() {
        if (selectedId === null || !editName.trim() || !nameChanged) {
            return;
        }
        const id = selectedId;
        const name = editName.trim();
        const succeeded = await run(
            'Rename instrument',
            'Instrument renamed.',
            () => setInstrumentName(id, name),
            'name'
        );
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
            'Transposition mode updated.',
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
                'Key transform updated.',
                () => setInstrumentKeyTransform(id, transform),
                'keys'
            );
            if (succeeded && selectedId === id) {
                selectedKeys = selection.type === 'Keys' ? formatList(selection.data.keys) : '';
                keyMapping = formatMapping(mapping);
            }
        } catch (cause) {
            showError(cause);
            notify.error(error, { title: 'Update key transform' });
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
            'Key transform bypass updated.',
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
            error = 'Minimum velocity must not exceed maximum velocity';
            notify.error(error, { title: 'Update velocity transform' });
            return;
        }
        const transform: MidiVelocityTransform =
            velocityType === 'Linear'
                ? { type: 'Linear', data: { min: velocityMin, max: velocityMax } }
                : { type: 'Unit' };
        await run(
            'Update velocity transform',
            'Velocity transform updated.',
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
            'Velocity transform bypass updated.',
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
                'Channel filter updated.',
                () => setInstrumentChannelFilter(id, filter),
                'channels'
            );
            if (succeeded && selectedId === id) {
                channels = [...filter.list];
            }
        } catch (cause) {
            showError(cause);
            notify.error(error, { title: 'Update channel filter' });
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
                'Controller filter updated.',
                () => setInstrumentCcFilter(id, filter),
                'controllers'
            );
            if (succeeded && selectedId === id) {
                controllers = [...filter.list];
            }
        } catch (cause) {
            showError(cause);
            notify.error(error, { title: 'Update controller filter' });
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
            'MIDI message handling updated.',
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
        controllerHintsOpen = false;
        activeControllerHint = -1;
    }

    function removeController(controller: number) {
        controllers = controllers.filter((item) => item !== controller);
    }

    function selectControllerHint(controller: number) {
        controllerQuery = String(controller);
        controllerHintsOpen = false;
        activeControllerHint = -1;
    }

    function handleControllerKeydown(event: KeyboardEvent) {
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            controllerHintsOpen = true;
            activeControllerHint = Math.min(activeControllerHint + 1, controllerHints.length - 1);
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            activeControllerHint = Math.max(activeControllerHint - 1, 0);
        } else if (event.key === 'Enter') {
            event.preventDefault();
            if (activeControllerHint >= 0) {
                selectControllerHint(controllerHints[activeControllerHint].id);
            } else {
                addController();
            }
        } else if (event.key === 'Escape') {
            controllerHintsOpen = false;
            activeControllerHint = -1;
        }
    }

    function toggleControllerLearning() {
        if (learningController) {
            learningController = false;
            return;
        }
        controllerHintsOpen = false;
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
        const succeeded = await run('Remove instrument', 'Instrument removed.', () =>
            removeInstrument(id)
        );
        if (succeeded) {
            selectedId = null;
            inspectorOpen = false;
            const first = entries[0];
            if (first) {
                selectInstrument(first.id, false);
            }
        }
    }

    async function showRemoveConfirmation() {
        removeConfirming = true;
        await tick();
        removeConfirmationElement?.scrollIntoView({
            behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches
                ? 'auto'
                : 'smooth',
            block: 'nearest'
        });
    }
</script>

{#snippet saveStatus(section: string, pending: boolean)}
    <span
        class={`grid size-9 shrink-0 place-items-center ${savingSections.has(section) || pending ? 'text-accent' : 'text-foreground-icon'}`}
        title={savingSections.has(section) ? 'Saving…' : pending ? 'Pending change' : 'Saved'}
        aria-label={savingSections.has(section) ? 'Saving' : pending ? 'Pending change' : 'Saved'}>
        <Icon
            icon={savingSections.has(section) ? 'lucide:loader-circle' : 'lucide:check'}
            width="1.125rem"
            class={savingSections.has(section) ? 'animate-spin' : ''} />
    </span>
{/snippet}

{#snippet saveButton(section: string, pending: boolean)}
    <button
        class="text-accent enabled:hover:bg-accent/10 disabled:text-foreground-icon grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border-0 bg-transparent transition-colors disabled:cursor-default disabled:opacity-50"
        type="submit"
        aria-label="Apply changes"
        title="Apply changes"
        disabled={busy || !pending}>
        <Icon
            icon={savingSections.has(section) ? 'lucide:loader-circle' : 'lucide:check'}
            width="1.125rem"
            class={savingSections.has(section) ? 'animate-spin' : ''} />
    </button>
{/snippet}

{#if error}
    <div
        class="border-danger/30 bg-danger/8 text-danger mb-3 rounded-lg border px-3 py-2 text-sm"
        role="alert">
        {error}
    </div>
{/if}

<div
    class="grid min-w-0 gap-3 min-[56.25rem]:grid-cols-[minmax(18rem,0.8fr)_minmax(24rem,1.2fr)] min-[56.25rem]:items-start">
    <section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Instrument slots
                </h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    {entries.filter(({ slot }) => slot.enabled).length} enabled · {entries.length} total
                </p>
            </div>
            <div class="flex items-center gap-1.5">
                <button
                    class="text-foreground-muted hover:bg-surface-hover grid size-9 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-wait disabled:opacity-60"
                    type="button"
                    aria-label="Reload instruments"
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
                </button>
                <button
                    class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover grid size-9 cursor-pointer place-items-center rounded-lg border transition-colors disabled:cursor-wait disabled:opacity-60"
                    type="button"
                    aria-label="Add instrument"
                    disabled={busy}
                    onclick={() => void appendInstrument()}>
                    <Icon icon="lucide:plus" width="1.125rem" />
                </button>
            </div>
        </header>

        {#if loading}
            <div class="text-foreground-muted grid min-h-52 place-items-center text-sm">
                Loading instruments…
            </div>
        {:else if entries.length === 0}
            <div class="text-foreground-muted grid min-h-52 place-items-center p-6 text-center">
                <div>
                    <Icon icon="lucide:sliders-horizontal" width="1.5rem" class="mx-auto mb-2" />
                    <p class="m-0 text-sm">No instrument slots are configured.</p>
                </div>
            </div>
        {:else}
            <div class="grid gap-1.5 p-1.5">
                {#each entries as entry (entry.id)}
                    {@const assignedSynth = synthEntry(entry.slot.synth_id)}
                    <article
                        class={`bg-surface-item relative rounded-lg border transition-colors ${entry.id === selectedId ? 'border-accent/45' : 'border-transparent'}`}>
                        <button
                            class="grid w-full cursor-pointer grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 border-0 bg-transparent px-3 py-3 pr-14 text-left"
                            type="button"
                            onclick={() => selectInstrument(entry.id)}>
                            <span
                                class="text-foreground-muted grid size-8 place-items-center rounded-lg font-mono text-sm"
                                style:background={`${synthColor(entry.slot.synth_id)}18`}>
                                {entry.id + 1}
                            </span>
                            <span class="min-w-0">
                                <strong
                                    class={`block truncate text-sm font-semibold ${entry.slot.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                                    {displayName(entry.id, entry.slot)}
                                </strong>
                                <small class="text-foreground-muted mt-0.5 block truncate text-sm">
                                    {synthName(assignedSynth)}
                                </small>
                            </span>
                            <Icon
                                icon="lucide:chevron-right"
                                width="1rem"
                                class="text-foreground-icon" />
                        </button>
                        <button
                            class={`absolute top-1/2 right-2 grid size-9 -translate-y-1/2 cursor-pointer place-items-center rounded-lg border transition-colors ${entry.slot.enabled ? 'border-accent/60 bg-accent/20 text-accent' : 'border-border-strong bg-surface-button text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}
                            type="button"
                            aria-label={`${entry.slot.enabled ? 'Disable' : 'Enable'} ${displayName(entry.id, entry.slot)}`}
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
    </section>

    {#if inspectorOpen}<button
            class="bg-scrim/60 fixed inset-0 z-30 cursor-default border-0 min-[56.25rem]:hidden"
            type="button"
            aria-label="Close instrument editor"
            onclick={() => (inspectorOpen = false)}>
        </button>{/if}

    {#if selectedSlot && selectedId !== null}
        <aside
            class={`border-border bg-surface-card fixed right-0 bottom-0 left-0 z-40 max-h-[calc(100dvh-4rem)] overflow-clip rounded-t-2xl border transition-transform min-[56.25rem]:sticky min-[56.25rem]:top-20.5 min-[56.25rem]:z-0 min-[56.25rem]:max-h-[calc(100dvh-8.5rem)] min-[56.25rem]:translate-y-0 min-[56.25rem]:rounded-xl ${inspectorOpen ? 'translate-y-0' : 'translate-y-full'}`}>
            <header
                class="border-border flex min-h-15 items-center justify-between gap-3 border-b px-4 py-2.5">
                <div class="min-w-0">
                    <small class="text-foreground-muted block text-xs">Slot {selectedId + 1}</small>
                    <strong
                        class={`block truncate text-base ${selectedSlot.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                        {displayName(selectedId, selectedSlot)}
                    </strong>
                </div>
                <button
                    class="bg-surface-button grid size-9 place-items-center rounded-lg border-0 min-[56.25rem]:hidden"
                    type="button"
                    aria-label="Close instrument editor"
                    onclick={() => (inspectorOpen = false)}>
                    <Icon icon="lucide:x" width="1.125rem" />
                </button>
            </header>

            <div
                class="max-h-[calc(100dvh-7.75rem)] overflow-y-auto pb-[max(1rem,env(safe-area-inset-bottom))] min-[56.25rem]:max-h-[calc(100dvh-12.25rem)] min-[56.25rem]:pb-0">
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
                    <div class="flex gap-2">
                        <input
                            id="instrument-name"
                            class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 flex-1 rounded-lg border px-2.5 text-sm"
                            bind:value={editName}
                            onblur={() => {
                                if (nameChanged) {
                                    void saveName();
                                }
                            }}
                            onkeydown={(event) => {
                                if (event.key === 'Enter') {
                                    event.preventDefault();
                                    event.currentTarget.blur();
                                }
                            }}
                            required />
                        {@render saveStatus('name', nameChanged)}
                    </div>
                </form>

                <form
                    class="border-border grid gap-2 border-b p-4"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveAssignment();
                    }}>
                    <label
                        class="text-foreground-strong text-xs font-semibold uppercase"
                        for="instrument-synth">
                        Synth
                    </label>
                    <div class="flex items-center gap-2">
                        <select
                            id="instrument-synth"
                            class="border-border-strong bg-surface-inset text-foreground h-9 min-w-0 flex-1 rounded-lg border px-2.5 text-sm"
                            bind:value={selectedSynth}
                            onchange={() => void saveAssignment()}>
                            <option value="">Unassigned</option>
                            {#each availableSynths as synth}<option value={String(synth.id)}>
                                    {synth.kind} · {synthName(synth)} · #{synth.id}
                                </option>{/each}
                        </select>
                        {@render saveStatus('synth', synthChanged)}
                    </div>
                </form>

                <section class="border-border grid gap-3 border-b p-4">
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        Behavior
                    </h3>
                    <div class="flex items-center gap-2">
                        <label
                            class={`has-focus-visible:outline-accent flex h-9 min-w-0 flex-1 cursor-pointer items-center justify-between gap-3 rounded-lg border px-3 text-sm transition-colors has-focus-visible:outline-2 has-focus-visible:outline-offset-2 ${useGlobalTransposition ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'}`}>
                            <span class="min-w-0 truncate">Use global transposition</span>
                            <input
                                class="sr-only"
                                type="checkbox"
                                bind:checked={useGlobalTransposition}
                                onchange={() => void saveTranspositionMode()} />
                            <span
                                class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${useGlobalTransposition ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                aria-hidden="true">
                                {#if useGlobalTransposition}<span
                                        class="bg-accent size-2 rounded-full">
                                    </span>{/if}
                            </span>
                        </label>
                        {@render saveStatus('behavior', behaviorChanged)}
                    </div>
                </section>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveKeyTransform();
                    }}>
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] gap-2">
                        <div class="flex items-center justify-between gap-3">
                            <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                                Keys
                            </h3>
                            <label
                                class="text-foreground-muted has-focus-visible:outline-accent flex cursor-pointer items-center gap-2 rounded-md text-xs font-medium has-focus-visible:outline-2 has-focus-visible:outline-offset-2">
                                <span>Bypass</span>
                                <input
                                    class="sr-only"
                                    type="checkbox"
                                    bind:checked={ignoreKeyTransform}
                                    disabled={savingSections.has('ignore-key-transform')}
                                    onchange={() => void saveIgnoreKeyTransform()} />
                                <span
                                    class={`relative h-4 w-7 shrink-0 rounded-full border transition-colors ${ignoreKeyTransform ? 'border-accent/60 bg-accent/25' : 'border-border-strong bg-surface-inset'}`}
                                    aria-hidden="true">
                                    {#if savingSections.has('ignore-key-transform')}
                                        <Icon
                                            icon="lucide:loader-circle"
                                            width="0.75rem"
                                            class="text-accent absolute inset-0 m-auto animate-spin" />
                                    {:else}<span
                                            class={`absolute top-0.5 left-0.5 size-2.5 rounded-full transition-all ${ignoreKeyTransform ? 'bg-accent translate-x-3' : 'bg-foreground-icon'}`}>
                                        </span>{/if}
                                </span>
                            </label>
                        </div>
                    </div>
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] items-end gap-2">
                        <div class="grid min-w-0 gap-4">
                            <div class="grid grid-cols-2 gap-2">
                                <div class="grid gap-1 text-sm">
                                    <span class="text-foreground-muted">Selection</span>
                                    <div
                                        class="border-border-strong bg-surface-inset grid grid-cols-3 overflow-hidden rounded-lg border"
                                        role="radiogroup"
                                        aria-label="Key selection">
                                        {#each [{ value: 'All', label: 'All' }, { value: 'Range', label: 'Range' }, { value: 'Keys', label: 'Specific' }] as option, index}<label
                                                class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-1 text-center text-xs font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:outline-offset-[-0.125rem] ${index > 0 ? 'border-border-strong border-l' : ''} ${keySelectionType === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                                                <input
                                                    class="sr-only"
                                                    type="radio"
                                                    name="key-selection"
                                                    value={option.value}
                                                    bind:group={keySelectionType}
                                                    onchange={stopKeyRecording} />
                                                <span>{option.label}</span>
                                            </label>{/each}
                                    </div>
                                </div>
                                <div class="grid gap-1 text-sm">
                                    <span class="text-foreground-muted">Mapping</span>
                                    <div
                                        class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border"
                                        role="radiogroup"
                                        aria-label="Key mapping">
                                        {#each [{ value: 'Identity', label: 'Identity' }, { value: 'Custom', label: 'Custom' }] as option, index}<label
                                                class={`has-focus-visible:outline-accent grid min-h-10 cursor-pointer place-items-center px-1 text-center text-xs font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:outline-offset-[-0.125rem] ${index > 0 ? 'border-border-strong border-l' : ''} ${keyMappingType === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                                                <input
                                                    class="sr-only"
                                                    type="radio"
                                                    name="key-mapping"
                                                    value={option.value}
                                                    bind:group={keyMappingType}
                                                    onchange={stopKeyRecording} />
                                                <span>{option.label}</span>
                                            </label>{/each}
                                    </div>
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
                                disabled={busy}
                                onchange={(value) => {
                                    localTransposition = value;
                                }} />
                        </div>
                        {@render saveButton('keys', keyTransformChanged)}
                    </div>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveVelocityTransform();
                    }}>
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] gap-2">
                        <div class="flex items-center justify-between gap-3">
                            <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                                Velocity
                            </h3>
                            <label
                                class="text-foreground-muted has-focus-visible:outline-accent flex cursor-pointer items-center gap-2 rounded-md text-xs font-medium has-focus-visible:outline-2 has-focus-visible:outline-offset-2">
                                <span>Bypass</span>
                                <input
                                    class="sr-only"
                                    type="checkbox"
                                    bind:checked={ignoreVelocityTransform}
                                    disabled={savingSections.has('ignore-velocity-transform')}
                                    onchange={() => void saveIgnoreVelocityTransform()} />
                                <span
                                    class={`relative h-4 w-7 shrink-0 rounded-full border transition-colors ${ignoreVelocityTransform ? 'border-accent/60 bg-accent/25' : 'border-border-strong bg-surface-inset'}`}
                                    aria-hidden="true">
                                    {#if savingSections.has('ignore-velocity-transform')}
                                        <Icon
                                            icon="lucide:loader-circle"
                                            width="0.75rem"
                                            class="text-accent absolute inset-0 m-auto animate-spin" />
                                    {:else}<span
                                            class={`absolute top-0.5 left-0.5 size-2.5 rounded-full transition-all ${ignoreVelocityTransform ? 'bg-accent translate-x-3' : 'bg-foreground-icon'}`}>
                                        </span>{/if}
                                </span>
                            </label>
                        </div>
                    </div>
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] items-end gap-2">
                        <div class="grid min-w-0 gap-4">
                            <div
                                class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border"
                                role="radiogroup"
                                aria-label="Velocity transform">
                                {#each [{ value: 'Unit', label: 'Unchanged' }, { value: 'Linear', label: 'Linear range' }] as option, index}<label
                                        class={`has-focus-visible:outline-accent grid min-h-9 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:outline-offset-[-0.125rem] ${index > 0 ? 'border-border-strong border-l' : ''} ${velocityType === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                                        <input
                                            class="sr-only"
                                            type="radio"
                                            name="velocity-transform"
                                            value={option.value}
                                            bind:group={velocityType} />
                                        <span>{option.label}</span>
                                    </label>{/each}
                            </div>
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
                                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:mt-[-0.375rem] [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
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
                                            class="[&::-moz-range-thumb]:bg-accent [&::-webkit-slider-thumb]:bg-accent pointer-events-none absolute inset-0 h-full w-full appearance-none bg-transparent [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:size-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:h-1 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:mt-[-0.375rem] [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full"
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
                        {@render saveButton('velocity', velocityChanged)}
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
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] items-end gap-2">
                        <div class="grid min-w-0 gap-4">
                            <div
                                class="border-border-strong bg-surface-inset grid grid-cols-2 overflow-hidden rounded-lg border"
                                role="radiogroup"
                                aria-label="MIDI channel filter mode">
                                {#each [{ value: MidiFilterKind.Allowlist, label: 'Allow only' }, { value: MidiFilterKind.Denylist, label: 'Allow all except' }] as option, index}<label
                                        class={`has-focus-visible:outline-accent grid min-h-9 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:outline-offset-[-0.125rem] ${index > 0 ? 'border-border-strong border-l' : ''} ${channelFilterType === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                                        <input
                                            class="sr-only"
                                            type="radio"
                                            name="midi-channel-filter-mode"
                                            value={option.value}
                                            bind:group={channelFilterType} />
                                        <span>{option.label}</span>
                                    </label>{/each}
                            </div>
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
                                    <button
                                        class="border-border-strong bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={channels.length === midiChannels.length}
                                        onclick={() => {
                                            channels = [...midiChannels];
                                        }}>
                                        All
                                    </button>
                                    <button
                                        class="border-border-strong bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={channels.length === 0}
                                        onclick={() => {
                                            channels = [];
                                        }}>
                                        None
                                    </button>
                                </div>
                            </div>
                        </div>
                        {@render saveButton('channels', channelsChanged)}
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
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] items-end gap-2">
                        <div class="grid min-w-0 gap-4">
                            <div
                                class="border-border-strong bg-surface-inset grid h-9 grid-cols-2 overflow-hidden rounded-lg border"
                                role="radiogroup"
                                aria-label="MIDI controller filter mode">
                                {#each [{ value: MidiFilterKind.Allowlist, label: 'Allow only' }, { value: MidiFilterKind.Denylist, label: 'Allow all except' }] as option, index}<label
                                        class={`has-focus-visible:outline-accent grid h-9 min-w-0 cursor-pointer place-items-center px-2 text-center text-sm font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:outline-offset-[-0.125rem] ${index > 0 ? 'border-border-strong border-l' : ''} ${ccFilterType === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}>
                                        <input
                                            class="sr-only"
                                            type="radio"
                                            name="midi-controller-filter-mode"
                                            value={option.value}
                                            bind:group={ccFilterType} />
                                        <span class="max-w-full truncate">{option.label}</span>
                                    </label>{/each}
                            </div>
                            <div class="grid min-w-0 gap-1">
                                <div class="flex min-w-0 gap-1.5">
                                    <div class="relative min-w-0 flex-1">
                                        <input
                                            class="border-border-strong bg-surface-inset h-9 w-full min-w-0 rounded-lg border px-2.5 text-sm"
                                            type="text"
                                            role="combobox"
                                            aria-label="MIDI controller to add"
                                            aria-autocomplete="list"
                                            aria-controls="midi-controller-hints"
                                            aria-expanded={controllerHintsOpen &&
                                                controllerHints.length > 0}
                                            placeholder="CC number or name"
                                            bind:value={controllerQuery}
                                            disabled={availableControllers.length === 0}
                                            onfocus={() => (controllerHintsOpen = true)}
                                            onblur={() => (controllerHintsOpen = false)}
                                            oninput={() => {
                                                controllerHintsOpen = true;
                                                activeControllerHint = -1;
                                            }}
                                            onkeydown={handleControllerKeydown} />
                                        {#if controllerHintsOpen && controllerHints.length > 0}
                                            <div
                                                id="midi-controller-hints"
                                                class="border-border-strong bg-surface-card absolute top-[calc(100%+0.25rem)] right-0 left-0 z-20 max-h-48 overflow-y-auto rounded-lg border p-1 shadow-lg"
                                                role="listbox">
                                                {#each controllerHints as controller, index}<button
                                                        class={`text-foreground hover:bg-surface-hover grid w-full cursor-pointer grid-cols-[3.5rem_minmax(0,1fr)] items-center gap-2 rounded-md border-0 px-2 py-1.5 text-left text-sm ${index === activeControllerHint ? 'bg-surface-hover' : 'bg-transparent'}`}
                                                        type="button"
                                                        role="option"
                                                        aria-selected={index ===
                                                            activeControllerHint}
                                                        onpointerdown={(event) => {
                                                            event.preventDefault();
                                                            selectControllerHint(controller.id);
                                                        }}>
                                                        <span
                                                            class="text-foreground-muted font-mono whitespace-nowrap">
                                                            CC {controller.id}
                                                        </span>
                                                        <span class="truncate">
                                                            {controller.name}
                                                        </span>
                                                    </button>{/each}
                                            </div>
                                        {/if}
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
                                    <button
                                        class="bg-surface-button text-foreground enabled:hover:bg-surface-hover grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border-0 transition-colors disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        aria-label="Add MIDI controller"
                                        title="Add MIDI controller"
                                        disabled={controllerToAdd === undefined}
                                        onclick={addController}>
                                        <Icon icon="lucide:plus" width="1rem" />
                                    </button>
                                </div>
                                {#if selectedControllerHint}<small
                                        class="text-foreground-muted truncate px-1 text-xs">
                                        CC {selectedControllerHint.id} · {selectedControllerHint.name}
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
                                            <button
                                                class="text-foreground-muted hover:bg-danger/10 hover:text-danger grid size-7 shrink-0 cursor-pointer place-items-center rounded-md border-0 bg-transparent"
                                                type="button"
                                                aria-label={`Remove CC ${controller}`}
                                                title={`Remove CC ${controller}`}
                                                onclick={() => removeController(controller)}>
                                                <Icon icon="lucide:x" width="0.875rem" />
                                            </button>
                                        </span>{:else}<span
                                            class="text-foreground-muted self-center px-1 text-xs">
                                            No controllers selected
                                        </span>{/each}
                                </div>
                                <div class="flex justify-end gap-1.5">
                                    <button
                                        class="border-border-strong bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={controllers.length === midiControllers.length}
                                        onclick={() => {
                                            controllers = midiControllers.map(({ id }) => id);
                                        }}>
                                        All
                                    </button>
                                    <button
                                        class="border-border-strong bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={controllers.length === 0}
                                        onclick={() => {
                                            controllers = [];
                                        }}>
                                        None
                                    </button>
                                </div>
                            </div>
                        </div>
                        {@render saveButton('controllers', controllersChanged)}
                    </div>
                    <small class="text-foreground-muted">
                        Selected controllers are {ccFilterType === MidiFilterKind.Allowlist
                            ? 'allowed'
                            : 'blocked'}.
                    </small>
                </form>

                <form
                    class="border-border grid gap-5 border-b p-4"
                    use:submitControlGroupOnEnter
                    onsubmit={(event) => {
                        event.preventDefault();
                        void saveIgnoredMessages();
                    }}>
                    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
                        Ignored MIDI messages
                    </h3>
                    <div class="grid grid-cols-[minmax(0,1fr)_2.25rem] items-end gap-2">
                        <div class="grid min-w-0 grid-cols-2 gap-1.5">
                            <button
                                class={`flex min-h-9 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors ${ignorePitchBend ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'}`}
                                type="button"
                                aria-pressed={ignorePitchBend}
                                onclick={() => {
                                    ignorePitchBend = !ignorePitchBend;
                                }}>
                                <span
                                    class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${ignorePitchBend ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                    aria-hidden="true">
                                    {#if ignorePitchBend}<span
                                            class="bg-accent size-2 rounded-full">
                                        </span>{/if}
                                </span>
                                <span class="truncate">Pitch bend</span>
                            </button>
                            <button
                                class={`flex min-h-9 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors ${ignoreProgramChange ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'}`}
                                type="button"
                                aria-pressed={ignoreProgramChange}
                                onclick={() => {
                                    ignoreProgramChange = !ignoreProgramChange;
                                }}>
                                <span
                                    class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${ignoreProgramChange ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                    aria-hidden="true">
                                    {#if ignoreProgramChange}<span
                                            class="bg-accent size-2 rounded-full">
                                        </span>{/if}
                                </span>
                                <span class="truncate">Program change</span>
                            </button>
                            <button
                                class={`flex min-h-9 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors ${ignoreChannelAftertouch ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'}`}
                                type="button"
                                aria-pressed={ignoreChannelAftertouch}
                                onclick={() => {
                                    ignoreChannelAftertouch = !ignoreChannelAftertouch;
                                }}>
                                <span
                                    class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${ignoreChannelAftertouch ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                    aria-hidden="true">
                                    {#if ignoreChannelAftertouch}<span
                                            class="bg-accent size-2 rounded-full">
                                        </span>{/if}
                                </span>
                                <span class="truncate">Channel aftertouch</span>
                            </button>
                            <button
                                class={`flex min-h-9 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors ${ignorePolyphonicAftertouch ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'}`}
                                type="button"
                                aria-pressed={ignorePolyphonicAftertouch}
                                onclick={() => {
                                    ignorePolyphonicAftertouch = !ignorePolyphonicAftertouch;
                                }}>
                                <span
                                    class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${ignorePolyphonicAftertouch ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                    aria-hidden="true">
                                    {#if ignorePolyphonicAftertouch}<span
                                            class="bg-accent size-2 rounded-full">
                                        </span>{/if}
                                </span>
                                <span class="truncate">Poly aftertouch</span>
                            </button>
                        </div>
                        {@render saveButton('ignored', ignoredMessagesChanged)}
                    </div>
                </form>

                <section class="p-4">
                    {#if removeConfirming}
                        <div
                            bind:this={removeConfirmationElement}
                            class="border-danger/35 bg-danger/8 grid gap-3 rounded-lg border p-3">
                            <p class="text-danger m-0 text-sm">
                                Remove this instrument slot? This cannot be undone.
                            </p>
                            <div class="flex justify-end gap-2">
                                <button
                                    class="border-border-strong bg-surface-card hover:bg-surface-hover cursor-pointer rounded-lg border px-3 py-2 text-sm font-semibold transition-colors disabled:cursor-wait disabled:opacity-60"
                                    type="button"
                                    disabled={busy}
                                    onclick={() => (removeConfirming = false)}>
                                    Cancel
                                </button>
                                <button
                                    class="bg-danger hover:bg-danger/85 cursor-pointer rounded-lg border-0 px-3 py-2 text-sm font-bold text-white transition-colors disabled:cursor-wait disabled:opacity-60"
                                    type="button"
                                    disabled={busy}
                                    onclick={() => void deleteSelected()}>
                                    Remove instrument
                                </button>
                            </div>
                        </div>
                    {:else}
                        <button
                            class="text-danger hover:bg-danger/10 w-full cursor-pointer rounded-lg border-0 bg-transparent px-3 py-2 text-sm font-semibold"
                            type="button"
                            onclick={() => void showRemoveConfirmation()}>
                            Remove instrument
                        </button>
                    {/if}
                </section>
            </div>
        </aside>
    {/if}
</div>
