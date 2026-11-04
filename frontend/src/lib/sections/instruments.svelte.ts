import { onMount } from 'svelte';
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
    setInstrumentEnabled,
    setInstrumentName,
    setInstrumentSourceFilter,
    setInstrumentChannelFilter,
    setInstrumentUsesGlobalTransposition,
    setInstrumentCcFilter,
    setInstrumentIgnoreChannelAftertouch,
    setInstrumentIgnorePitchBend,
    setInstrumentIgnorePolyphonicAftertouch,
    setInstrumentIgnoreProgramChange,
    setInstrumentIgnoreKeyTransform,
    setInstrumentKeyTransform,
    setInstrumentIgnoreVelocityTransform,
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
import { midiControllerNames, midiControllers } from '$lib/content/midi-controller';
import { midiChannels, midiSources } from '$lib/content/midi';
export { MidiFilterKind, midiControllerNames };

export type InstrumentsSectionProps = {
    instrumentId: () => number | null;
    embedded: () => boolean;
    showAddButton: () => boolean;
};

export class InstrumentsSection {
    readonly midiChannels = midiChannels;
    readonly midiSources = midiSources;

    selectedId = $state<number | null>(null);
    loading = $state(true);
    busy = $state(false);
    addingInstrument = $state(false);
    removingInstrument = $state(false);
    togglingId = $state<number | null>(null);
    reloading = $state(false);
    reloadDone = $state(false);
    editName = $state('');
    savingSections = $state(new Set<string>());

    private queuedOperations = 0;
    private operationQueue: Promise<void> = Promise.resolve();
    private savingSectionCounts = new Map<string, number>();

    entries = $derived(
        Object.entries(instrumentSlots())
            .map(([id, slot]) => ({ id: Number(id), slot }))
            .filter(
                (entry) =>
                    !this.props.embedded() ||
                    this.props.instrumentId() === null ||
                    entry.slot.instrument_id === this.props.instrumentId()
            )
            .sort((left, right) => left.id - right.id)
    );
    selectedSlot = $derived(
        this.selectedId === null ? undefined : instrumentSlots()[String(this.selectedId)]
    );
    availableSynths = $derived(
        [...synths()].sort(
            (left, right) => left.kind.localeCompare(right.kind) || left.id - right.id
        )
    );
    nameChanged = $derived(
        Boolean(this.selectedSlot && this.editName.trim() !== this.selectedSlot.name)
    );

    readonly behaviorSection;
    readonly sourceSection;
    readonly keySection;
    readonly velocitySection;
    readonly channelSection;
    readonly controllerSection;
    readonly ignoredSection;

    constructor(readonly props: InstrumentsSectionProps) {
        const context = {
            selection: () =>
                this.selectedId === null || !this.selectedSlot
                    ? undefined
                    : { id: this.selectedId, slot: this.selectedSlot },
            run: this.run.bind(this)
        };
        this.behaviorSection = new ControllerBehaviorSection(context);
        this.sourceSection = new MidiListFilterSection(context, {
            title: 'Update MIDI source filter',
            section: 'sources',
            read: (slot) => slot.source_filter,
            write: setInstrumentSourceFilter
        });
        this.keySection = new KeyTransformSection(context);
        this.velocitySection = new VelocityTransformSection(context);
        this.channelSection = new MidiListFilterSection(context, {
            title: 'Update channel filter',
            section: 'channels',
            read: (slot) => slot.channel_filter,
            write: setInstrumentChannelFilter
        });
        this.controllerSection = new ControllerFilterSection(context);
        this.ignoredSection = new IgnoredMidiMessagesSection(context);

        onMount(async () => {
            try {
                await Promise.all([fetchInstruments(), fetchSynths()]);
                const instrumentId = this.props.instrumentId();
                const requestedInstrument =
                    instrumentId === null
                        ? new URL(window.location.href).searchParams.get('instrument')
                        : String(instrumentId);
                const initial =
                    (requestedInstrument === null
                        ? undefined
                        : this.entries.find((entry) =>
                              this.props.embedded()
                                  ? entry.slot.instrument_id === Number(requestedInstrument)
                                  : entry.id === Number(requestedInstrument)
                          )) ?? this.entries[0];
                if (initial) this.selectInstrument(initial.id);
            } catch (cause) {
                notify.error(errorToUserMessage(cause));
            } finally {
                this.loading = false;
            }
        });

        onMount(() =>
            subscribeMidiEvents(({ event }) => {
                if (event.type === 'NoteOn' && event.data.velocity > 0) {
                    this.keySection.record(event.data.key);
                }
                if (event.type === 'ControlChange') {
                    this.controllerSection.learn(event.data.controller);
                }
            })
        );
    }

    get embedded() {
        return this.props.embedded();
    }

    get showAddButton() {
        return this.props.showAddButton();
    }

    displayName(slot: ControllerSlot) {
        const name = slot.name.trim();
        return name ? `MIDI Input: ${name}` : 'MIDI Input';
    }

    synthEntry(id: number | null) {
        if (id === null) return undefined;
        return this.availableSynths.find((entry) => entry.id === id);
    }

    synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `${entry.kind} instrument #${entry.id + 1}` : 'Instrument')
        );
    }

    synthColor(id: number | null) {
        switch (this.synthEntry(id)?.kind) {
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

    selectInstrument(id: number) {
        const slot = instrumentSlots()[String(id)];
        if (!slot) return;
        this.selectedId = id;
        this.editName = slot.name;
        this.behaviorSection.load(slot);
        this.sourceSection.load(slot);
        this.keySection.load(slot);
        this.velocitySection.load(slot);
        this.channelSection.load(slot);
        this.controllerSection.load(slot);
        this.ignoredSection.load(slot);
    }

    async run(title: string, operation: () => Promise<unknown>, section?: string) {
        this.queuedOperations += 1;
        this.busy = true;
        if (section) {
            this.savingSectionCounts.set(section, (this.savingSectionCounts.get(section) ?? 0) + 1);
            this.savingSections = new Set(this.savingSections).add(section);
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
        const result = this.operationQueue.then(execute, execute);
        this.operationQueue = result.then(() => undefined);
        try {
            return await result;
        } finally {
            this.queuedOperations -= 1;
            this.busy = this.queuedOperations > 0;
            if (section) {
                const remaining = (this.savingSectionCounts.get(section) ?? 1) - 1;
                if (remaining > 0) {
                    this.savingSectionCounts.set(section, remaining);
                } else {
                    this.savingSectionCounts.delete(section);
                    const next = new Set(this.savingSections);
                    next.delete(section);
                    this.savingSections = next;
                }
            }
        }
    }

    async appendInstrument() {
        this.addingInstrument = true;
        let id: number | undefined;
        try {
            const succeeded = await this.run('Add controller', async () => {
                id = await addInstrument(this.props.instrumentId() ?? undefined);
            });
            if (succeeded && id !== undefined) this.selectInstrument(id);
        } finally {
            this.addingInstrument = false;
        }
    }

    async reloadInstruments() {
        if (this.reloading) return;
        this.reloading = true;
        this.reloadDone = false;
        try {
            await Promise.all([fetchInstrumentSlots(), fetchSynths()]);
            this.reloadDone = true;
            setTimeout(() => (this.reloadDone = false), 1200);
            if (this.selectedId !== null && !instrumentSlots()[String(this.selectedId)]) {
                const first = this.entries[0];
                this.selectedId = null;
                if (first) this.selectInstrument(first.id);
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Reload controllers' });
        } finally {
            this.reloading = false;
        }
    }

    async toggleEnabled(id: number, enabled: boolean) {
        if (this.togglingId !== null) return;
        this.togglingId = id;
        try {
            await setInstrumentEnabled(id, enabled);
        } catch (cause) {
            notify.error(errorToUserMessage(cause), {
                title: enabled ? 'Enable controller' : 'Disable controller'
            });
        } finally {
            this.togglingId = null;
        }
    }

    async saveName() {
        if (this.selectedId === null || !this.editName.trim() || !this.nameChanged) return;
        const id = this.selectedId;
        const name = this.editName.trim();
        const succeeded = await this.run(
            'Rename controller',
            () => setInstrumentName(id, name),
            'name'
        );
        if (succeeded && this.selectedId === id) this.editName = name;
    }

    async deleteSelected() {
        if (this.selectedId === null || !this.selectedSlot) return;
        const id = this.selectedId;
        this.removingInstrument = true;
        try {
            const succeeded = await this.run('Remove controller', () => removeInstrument(id));
            if (succeeded) {
                this.selectedId = null;
                const first = this.entries[0];
                if (first) this.selectInstrument(first.id);
            }
        } finally {
            this.removingInstrument = false;
        }
    }
}

type ControllerSelection = { id: number; slot: ControllerSlot };
type ControllerSectionContext = {
    selection: () => ControllerSelection | undefined;
    run: (title: string, operation: () => Promise<unknown>, section: string) => Promise<boolean>;
};

function sameNumberList(left: number[], right: number[]) {
    const sortedLeft = [...left].sort((a, b) => a - b);
    const sortedRight = [...right].sort((a, b) => a - b);
    return (
        sortedLeft.length === sortedRight.length &&
        sortedLeft.every((value, index) => value === sortedRight[index])
    );
}

export function submitControlGroupOnEnter(form: HTMLFormElement) {
    const handleKeydown = (event: KeyboardEvent) => {
        const target = event.target;
        if (
            event.key !== 'Enter' ||
            !(target instanceof HTMLButtonElement) ||
            target.type === 'submit'
        )
            return;
        event.preventDefault();
        form.requestSubmit();
    };
    form.addEventListener('keydown', handleKeydown);
    return { destroy: () => form.removeEventListener('keydown', handleKeydown) };
}

class ControllerBehaviorSection {
    useGlobalTransposition = $state(true);
    constructor(private context: ControllerSectionContext) {}
    get changed() {
        const selection = this.context.selection();
        return Boolean(
            selection && this.useGlobalTransposition !== selection.slot.use_global_transposition
        );
    }
    load(slot: ControllerSlot) {
        this.useGlobalTransposition = slot.use_global_transposition;
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        await this.context.run(
            'Global transposition',
            () => setInstrumentUsesGlobalTransposition(selection.id, this.useGlobalTransposition),
            'behavior'
        );
    }
}

class ControllerFilterSection {
    type = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    values = $state<number[]>([]);
    query = $state('');
    learning = $state(false);
    constructor(private context: ControllerSectionContext) {}
    get changed() {
        const saved = this.context.selection()?.slot.cc_filter;
        return Boolean(
            saved && (this.type !== saved.type || !sameNumberList(this.values, saved.list))
        );
    }
    get available() {
        return midiControllers.filter((controller) => !this.values.includes(controller.id));
    }
    get toAdd() {
        const numeric = Number(this.query.trim());
        return Number.isInteger(numeric) && this.available.some(({ id }) => id === numeric)
            ? numeric
            : undefined;
    }
    get selectedHint() {
        return this.available.find((controller) => controller.id === this.toAdd);
    }
    get addedHint() {
        const query = this.query.trim().toLocaleLowerCase();
        if (!query || this.selectedHint) return undefined;
        const numeric = Number(query);
        if (Number.isInteger(numeric)) {
            return midiControllers.find(
                (controller) => controller.id === numeric && this.values.includes(controller.id)
            );
        }
        return midiControllers.find(
            (controller) =>
                this.values.includes(controller.id) &&
                controller.name.toLocaleLowerCase().includes(query)
        );
    }
    load(slot: ControllerSlot) {
        this.type = slot.cc_filter.type;
        this.values = [...slot.cc_filter.list];
        this.query = '';
        this.learning = false;
    }
    add() {
        if (this.toAdd === undefined) return;
        this.values = [...this.values, this.toAdd].sort((a, b) => a - b);
        this.query = '';
    }
    remove(controller: number) {
        this.values = this.values.filter((item) => item !== controller);
    }
    toggleLearning() {
        this.learning = !this.learning;
    }
    learn(controller: number) {
        if (!this.learning) return;
        this.query = String(controller);
        this.learning = false;
        this.add();
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        const filter: MidiFilter = {
            type: this.type,
            list: [...this.values].sort((a, b) => a - b)
        };
        const succeeded = await this.context.run(
            'Update controller filter',
            () => setInstrumentCcFilter(selection.id, filter),
            'controllers'
        );
        if (succeeded && this.context.selection()?.id === selection.id) {
            this.values = [...filter.list];
        }
    }
}

class IgnoredMidiMessagesSection {
    pitchBend = $state(false);
    programChange = $state(false);
    channelAftertouch = $state(false);
    polyphonicAftertouch = $state(false);
    constructor(private context: ControllerSectionContext) {}
    get changed() {
        const slot = this.context.selection()?.slot;
        return Boolean(
            slot &&
                (this.pitchBend !== slot.ignore_pitch_bend ||
                    this.programChange !== slot.ignore_program_change ||
                    this.channelAftertouch !== slot.ignore_channel_aftertouch ||
                    this.polyphonicAftertouch !== slot.ignore_polyphonic_aftertouch)
        );
    }
    load(slot: ControllerSlot) {
        this.pitchBend = slot.ignore_pitch_bend;
        this.programChange = slot.ignore_program_change;
        this.channelAftertouch = slot.ignore_channel_aftertouch;
        this.polyphonicAftertouch = slot.ignore_polyphonic_aftertouch;
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        const { id, slot } = selection;
        const pitchBend = this.pitchBend;
        const programChange = this.programChange;
        const channelAftertouch = this.channelAftertouch;
        const polyphonicAftertouch = this.polyphonicAftertouch;
        await this.context.run(
            'Update MIDI handling',
            async () => {
                const updates: Promise<void>[] = [];
                if (pitchBend !== slot.ignore_pitch_bend)
                    updates.push(setInstrumentIgnorePitchBend(id, pitchBend));
                if (programChange !== slot.ignore_program_change)
                    updates.push(setInstrumentIgnoreProgramChange(id, programChange));
                if (channelAftertouch !== slot.ignore_channel_aftertouch)
                    updates.push(setInstrumentIgnoreChannelAftertouch(id, channelAftertouch));
                if (polyphonicAftertouch !== slot.ignore_polyphonic_aftertouch)
                    updates.push(setInstrumentIgnorePolyphonicAftertouch(id, polyphonicAftertouch));
                await Promise.all(updates);
            },
            'ignored'
        );
    }
}

function formatList(values: number[]) {
    return [...values].sort((a, b) => a - b).join(', ');
}
function formatMapping(mapping: MidiKeyMapping) {
    if (mapping.type !== 'Custom') return '';
    return Object.entries(mapping.data.map)
        .sort(([left], [right]) => Number(left) - Number(right))
        .map(([input, output]) => `${input}:${output}`)
        .join(', ');
}
function parseList(value: string) {
    if (!value.trim()) return [];
    const values = value
        .split(/[\s,]+/)
        .filter(Boolean)
        .map(Number);
    if (values.some((item) => !Number.isInteger(item) || item < 0 || item > 127))
        throw new Error('Enter whole numbers from 0 to 127, separated by commas');
    return [...new Set(values)];
}
function parseMapping(value: string) {
    const map: Record<string, number> = {};
    if (!value.trim()) return map;
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
        )
            throw new Error('Use input:output pairs from 0 to 127, separated by commas');
        map[String(input)] = output;
    }
    return map;
}
function parseValidMappingEntries(value: string) {
    const map: Record<string, number> = {};
    for (const pair of value.split(/[\s,]+/).filter(Boolean)) {
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
        )
            map[String(input)] = output;
    }
    return map;
}

class KeyTransformSection {
    selectionType = $state<MidiKeySelection['type']>('All');
    rangeMin = $state(0);
    rangeMax = $state(127);
    selectedKeys = $state('');
    mappingType = $state<MidiKeyMapping['type']>('Identity');
    mapping = $state('');
    transposition = $state(0);
    ignored = $state(false);
    recordingSelection = $state<'Range' | 'Keys' | null>(null);
    firstRangeKey = $state<number | null>(null);
    recordingMapping = $state(false);
    firstMappingKey = $state<number | null>(null);
    constructor(private context: ControllerSectionContext) {}
    get changed() {
        const slot = this.context.selection()?.slot;
        if (!slot) return false;
        const transform = slot.key_transform;
        return (
            this.selectionType !== transform.selection.type ||
            (transform.selection.type === 'Range' &&
                (this.rangeMin !== transform.selection.data.min ||
                    this.rangeMax !== transform.selection.data.max)) ||
            (transform.selection.type === 'Keys' &&
                this.selectedKeys !== formatList(transform.selection.data.keys)) ||
            this.mappingType !== transform.mapping.type ||
            this.mapping !== formatMapping(transform.mapping) ||
            this.transposition !== transform.transposition
        );
    }
    get ignoreChanged() {
        const slot = this.context.selection()?.slot;
        return Boolean(slot && this.ignored !== slot.ignore_key_transform);
    }
    load(slot: ControllerSlot) {
        this.stopRecording();
        this.selectionType = slot.key_transform.selection.type;
        this.rangeMin =
            slot.key_transform.selection.type === 'Range'
                ? slot.key_transform.selection.data.min
                : 0;
        this.rangeMax =
            slot.key_transform.selection.type === 'Range'
                ? slot.key_transform.selection.data.max
                : 127;
        this.selectedKeys =
            slot.key_transform.selection.type === 'Keys'
                ? formatList(slot.key_transform.selection.data.keys)
                : '';
        this.mappingType = slot.key_transform.mapping.type;
        this.mapping = formatMapping(slot.key_transform.mapping);
        this.transposition = slot.key_transform.transposition;
        this.ignored = slot.ignore_key_transform;
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        try {
            let keySelection: MidiKeySelection;
            if (this.selectionType === 'Range') {
                if (this.rangeMin > this.rangeMax)
                    throw new Error('The first key must not be higher than the last key');
                keySelection = {
                    type: 'Range',
                    data: { min: this.rangeMin, max: this.rangeMax }
                };
            } else if (this.selectionType === 'Keys') {
                keySelection = { type: 'Keys', data: { keys: parseList(this.selectedKeys) } };
            } else keySelection = { type: 'All' };
            const mapping: MidiKeyMapping =
                this.mappingType === 'Custom'
                    ? { type: 'Custom', data: { map: parseMapping(this.mapping) } }
                    : { type: 'Identity' };
            const transform: MidiKeyTransform = {
                selection: keySelection,
                mapping,
                transposition: this.transposition
            };
            const succeeded = await this.context.run(
                'Update key transform',
                () => setInstrumentKeyTransform(selection.id, transform),
                'keys'
            );
            if (succeeded && this.context.selection()?.id === selection.id) {
                this.selectedKeys =
                    keySelection.type === 'Keys' ? formatList(keySelection.data.keys) : '';
                this.mapping = formatMapping(mapping);
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update key transform' });
        }
    }
    async saveIgnored() {
        const selection = this.context.selection();
        if (!selection || !this.ignoreChanged) return;
        const ignored = this.ignored;
        const succeeded = await this.context.run(
            'Bypass key transform',
            () => setInstrumentIgnoreKeyTransform(selection.id, ignored),
            'ignore-key-transform'
        );
        if (!succeeded && this.context.selection()?.id === selection.id) this.ignored = !ignored;
    }
    stopRecording() {
        this.recordingSelection = null;
        this.firstRangeKey = null;
        this.recordingMapping = false;
        this.firstMappingKey = null;
    }
    toggleRecording(mode: 'Range' | 'Keys') {
        if (this.recordingSelection === mode) {
            this.stopRecording();
            return;
        }
        this.stopRecording();
        this.recordingSelection = mode;
    }
    toggleMappingRecording() {
        if (this.recordingMapping) {
            this.stopRecording();
            return;
        }
        this.stopRecording();
        this.recordingMapping = true;
    }
    record(key: number) {
        if (this.recordingSelection === 'Range') {
            if (this.firstRangeKey === null) {
                this.firstRangeKey = key;
                this.rangeMin = key;
                this.rangeMax = key;
                return;
            }
            this.rangeMin = Math.min(this.firstRangeKey, key);
            this.rangeMax = Math.max(this.firstRangeKey, key);
            this.stopRecording();
            return;
        }
        if (this.recordingSelection === 'Keys') {
            const keys = this.selectedKeys
                .split(',')
                .map((value) => Number(value.trim()))
                .filter((value) => Number.isInteger(value) && value >= 0 && value <= 127);
            this.selectedKeys = formatList([...new Set([...keys, key])]);
            return;
        }
        if (!this.recordingMapping) return;
        if (this.firstMappingKey === null) {
            this.firstMappingKey = key;
            return;
        }
        const map = parseValidMappingEntries(this.mapping);
        map[String(this.firstMappingKey)] = key;
        this.mapping = formatMapping({ type: 'Custom', data: { map } });
        this.firstMappingKey = null;
    }
}

type FilterConfig = {
    title: string;
    section: string;
    read: (slot: ControllerSlot) => MidiFilter;
    write: (id: number, filter: MidiFilter) => Promise<void>;
};

class MidiListFilterSection {
    type = $state<MidiFilterKind>(MidiFilterKind.Denylist);
    values = $state<number[]>([]);
    constructor(
        private context: ControllerSectionContext,
        private config: FilterConfig
    ) {}
    get changed() {
        const selection = this.context.selection();
        if (!selection) return false;
        const saved = this.config.read(selection.slot);
        return this.type !== saved.type || !sameNumberList(this.values, saved.list);
    }
    load(slot: ControllerSlot) {
        const filter = this.config.read(slot);
        this.type = filter.type;
        this.values = [...filter.list];
    }
    toggle(value: number, checked: boolean) {
        this.values = checked
            ? [...this.values, value].sort((left, right) => left - right)
            : this.values.filter((item) => item !== value);
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        const filter: MidiFilter = {
            type: this.type,
            list: [...this.values].sort((left, right) => left - right)
        };
        const succeeded = await this.context.run(
            this.config.title,
            () => this.config.write(selection.id, filter),
            this.config.section
        );
        if (succeeded && this.context.selection()?.id === selection.id)
            this.values = [...filter.list];
    }
}

class VelocityTransformSection {
    type = $state<MidiVelocityTransform['type']>('Unit');
    min = $state(0);
    max = $state(127);
    ignored = $state(false);
    constructor(private context: ControllerSectionContext) {}
    get changed() {
        const slot = this.context.selection()?.slot;
        if (!slot) return false;
        const transform = slot.velocity_transform;
        return (
            this.type !== transform.type ||
            (transform.type === 'Linear' &&
                (this.min !== transform.data.min || this.max !== transform.data.max))
        );
    }
    get ignoreChanged() {
        const slot = this.context.selection()?.slot;
        return Boolean(slot && this.ignored !== slot.ignore_velocity_transform);
    }
    load(slot: ControllerSlot) {
        this.type = slot.velocity_transform.type;
        this.min = slot.velocity_transform.type === 'Linear' ? slot.velocity_transform.data.min : 0;
        this.max =
            slot.velocity_transform.type === 'Linear' ? slot.velocity_transform.data.max : 127;
        this.ignored = slot.ignore_velocity_transform;
    }
    async save() {
        const selection = this.context.selection();
        if (!selection || !this.changed) return;
        if (this.type === 'Linear' && this.min > this.max) {
            notify.error('Minimum velocity must not exceed maximum velocity', {
                title: 'Update velocity transform'
            });
            return;
        }
        const transform: MidiVelocityTransform =
            this.type === 'Linear'
                ? { type: 'Linear', data: { min: this.min, max: this.max } }
                : { type: 'Unit' };
        await this.context.run(
            'Update velocity transform',
            () => setInstrumentVelocityTransform(selection.id, transform),
            'velocity'
        );
    }
    async saveIgnored() {
        const selection = this.context.selection();
        if (!selection || !this.ignoreChanged) return;
        const ignored = this.ignored;
        const succeeded = await this.context.run(
            'Bypass velocity transform',
            () => setInstrumentIgnoreVelocityTransform(selection.id, ignored),
            'ignore-velocity-transform'
        );
        if (!succeeded && this.context.selection()?.id === selection.id) this.ignored = !ignored;
    }
}
