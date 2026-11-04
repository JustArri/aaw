import { onMount } from 'svelte';
import { notify } from '$lib/push-notifications.svelte';
import {
    connectMidiInput,
    deleteMidiInputAutoConnection,
    disconnectMidiInput,
    fetchMidiInputs,
    midiInputAutoConnections,
    midiInputPorts,
    midiInputSlots,
    setMidiInputAutoConnection
} from '$lib/state/midi-inputs.svelte';

export class MidiInputsSection {
    refreshing = $state(false);
    refreshDone = $state(false);
    busySlotId = $state<number | null>(null);
    deletingPortId = $state<string | null>(null);
    addingAutoConnection = $state(false);
    selectedPorts = $state<Record<number, string>>({});
    autoPortId = $state('');
    preferredSlotId = $state('');
    fixedSlot = $state(false);
    errorMessage = $state('');
    ports = $derived(midiInputPorts());
    slots = $derived(midiInputSlots());
    autoConnections = $derived(
        [...midiInputAutoConnections()].sort((left, right) =>
            this.portName(left.port_id).localeCompare(this.portName(right.port_id))
        )
    );
    connectedPortIds = $derived(
        new Set(
            this.slots
                .filter((slot) => slot.type === 'Connected')
                .map((slot) => (slot.type === 'Connected' ? slot.data.port_id : ''))
        )
    );

    constructor() {
        onMount(() => void this.refresh(false));
    }

    private messageFor(error: unknown) {
        return error instanceof Error ? error.message : 'The MIDI input operation failed';
    }

    portName(portId: string) {
        return this.ports.find((port) => port.id === portId)?.name ?? portId;
    }

    async refresh(userInitiated = true) {
        if (this.refreshing) return;
        this.refreshing = true;
        this.refreshDone = false;
        this.errorMessage = '';
        try {
            await fetchMidiInputs();
            if (userInitiated) {
                this.refreshDone = true;
                setTimeout(() => (this.refreshDone = false), 1200);
            }
        } catch (error) {
            this.errorMessage = this.messageFor(error);
            if (userInitiated) notify.error(this.errorMessage, { title: 'Refresh MIDI inputs' });
        } finally {
            this.refreshing = false;
        }
    }

    async connect(slotId: number) {
        const portId = this.selectedPorts[slotId];
        if (!portId || this.busySlotId !== null) return;
        this.busySlotId = slotId;
        this.errorMessage = '';
        try {
            await connectMidiInput(slotId, portId);
            delete this.selectedPorts[slotId];
        } catch (error) {
            this.errorMessage = this.messageFor(error);
            notify.error(this.errorMessage, { title: `Connect MIDI slot ${slotId + 1}` });
        } finally {
            this.busySlotId = null;
        }
    }

    async disconnect(slotId: number) {
        if (this.busySlotId !== null) return;
        this.busySlotId = slotId;
        this.errorMessage = '';
        try {
            await disconnectMidiInput(slotId);
        } catch (error) {
            this.errorMessage = this.messageFor(error);
            notify.error(this.errorMessage, { title: `Disconnect MIDI slot ${slotId + 1}` });
        } finally {
            this.busySlotId = null;
        }
    }

    async addAutoConnection() {
        const portId = this.autoPortId.trim();
        if (!portId || this.addingAutoConnection) return;
        this.addingAutoConnection = true;
        this.errorMessage = '';
        try {
            await setMidiInputAutoConnection({
                port_id: portId,
                preferred_slot_id:
                    this.preferredSlotId === '' ? null : Number(this.preferredSlotId),
                slot_id_fixed: this.preferredSlotId !== '' && this.fixedSlot
            });
            this.autoPortId = '';
            this.preferredSlotId = '';
            this.fixedSlot = false;
        } catch (error) {
            this.errorMessage = this.messageFor(error);
            notify.error(this.errorMessage, { title: 'Add MIDI auto-connection' });
        } finally {
            this.addingAutoConnection = false;
        }
    }

    async removeAutoConnection(portId: string) {
        if (this.deletingPortId !== null) return;
        this.deletingPortId = portId;
        this.errorMessage = '';
        try {
            await deleteMidiInputAutoConnection(portId);
        } catch (error) {
            this.errorMessage = this.messageFor(error);
            notify.error(this.errorMessage, { title: 'Remove MIDI auto-connection' });
        } finally {
            this.deletingPortId = null;
        }
    }
}
