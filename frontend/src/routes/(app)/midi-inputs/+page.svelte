<script lang="ts">
    import Icon from '@iconify/svelte';
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

    let refreshing = $state(false);
    let refreshDone = $state(false);
    let busySlotId = $state(null as number | null);
    let deletingPortId = $state(null as string | null);
    let addingAutoConnection = $state(false);
    let selectedPorts = $state<Record<number, string>>({});
    let autoPortId = $state('');
    let preferredSlotId = $state('');
    let fixedSlot = $state(false);
    let errorMessage = $state('');

    let ports = $derived(midiInputPorts());
    let slots = $derived(midiInputSlots());
    let autoConnections = $derived(
        [...midiInputAutoConnections()].sort((left, right) =>
            portName(left.port_id).localeCompare(portName(right.port_id))
        )
    );
    let connectedPortIds = $derived(
        new Set(
            slots
                .filter((slot) => slot.type === 'Connected')
                .map((slot) => (slot.type === 'Connected' ? slot.data.port_id : ''))
        )
    );
    onMount(() => {
        void refresh(false);
    });

    function messageFor(error: unknown) {
        if (error instanceof Error) {
            return error.message;
        }
        return 'The MIDI input operation failed';
    }

    function portName(portId: string) {
        return ports.find((port) => port.id === portId)?.name ?? portId;
    }

    async function refresh(userInitiated = true) {
        if (refreshing) {
            return;
        }
        refreshing = true;
        refreshDone = false;
        errorMessage = '';

        try {
            await fetchMidiInputs();
            if (userInitiated) {
                refreshDone = true;
                setTimeout(() => {
                    refreshDone = false;
                }, 1200);
            }
        } catch (error) {
            errorMessage = messageFor(error);
            if (userInitiated) {
                notify.error(errorMessage, { title: 'Refresh MIDI inputs' });
            }
        } finally {
            refreshing = false;
        }
    }

    async function connect(slotId: number) {
        const portId = selectedPorts[slotId];
        if (!portId || busySlotId !== null) {
            return;
        }
        busySlotId = slotId;
        errorMessage = '';
        const notification = notify.info('Connecting the selected input…', {
            duration: null,
            loading: true,
            title: `Connect MIDI slot ${slotId + 1}`
        });

        try {
            await connectMidiInput(slotId, portId);
            delete selectedPorts[slotId];
            notification.success('MIDI input connected.');
        } catch (error) {
            errorMessage = messageFor(error);
            notification.error(errorMessage);
        } finally {
            busySlotId = null;
        }
    }

    async function disconnect(slotId: number) {
        if (busySlotId !== null) {
            return;
        }
        busySlotId = slotId;
        errorMessage = '';
        const notification = notify.info('Disconnecting the input…', {
            duration: null,
            loading: true,
            title: `Disconnect MIDI slot ${slotId + 1}`
        });

        try {
            await disconnectMidiInput(slotId);
            notification.success('MIDI input disconnected.');
        } catch (error) {
            errorMessage = messageFor(error);
            notification.error(errorMessage);
        } finally {
            busySlotId = null;
        }
    }

    async function addAutoConnection() {
        const portId = autoPortId.trim();
        if (!portId || addingAutoConnection) {
            return;
        }
        addingAutoConnection = true;
        errorMessage = '';
        const notification = notify.info('Saving the connection rule…', {
            duration: null,
            loading: true,
            title: 'Add MIDI auto-connection'
        });

        try {
            await setMidiInputAutoConnection({
                port_id: portId,
                preferred_slot_id: preferredSlotId === '' ? null : Number(preferredSlotId),
                slot_id_fixed: preferredSlotId !== '' && fixedSlot
            });
            autoPortId = '';
            preferredSlotId = '';
            fixedSlot = false;
            notification.success('Auto-connection added.');
        } catch (error) {
            errorMessage = messageFor(error);
            notification.error(errorMessage);
        } finally {
            addingAutoConnection = false;
        }
    }

    async function removeAutoConnection(portId: string) {
        if (deletingPortId !== null) {
            return;
        }
        deletingPortId = portId;
        errorMessage = '';
        const notification = notify.info('Removing the connection rule…', {
            duration: null,
            loading: true,
            title: 'Remove MIDI auto-connection'
        });

        try {
            await deleteMidiInputAutoConnection(portId);
            notification.success('Auto-connection removed.');
        } catch (error) {
            errorMessage = messageFor(error);
            notification.error(errorMessage);
        } finally {
            deletingPortId = null;
        }
    }
</script>

{#if errorMessage}
    <div
        class="border-danger/30 bg-danger/10 text-danger-foreground mb-3 flex items-start gap-2 rounded-xl border px-3.75 py-3 text-sm"
        role="alert">
        <Icon icon="lucide:circle-alert" width="1rem" class="mt-px shrink-0" />
        <span>{errorMessage}</span>
    </div>
{/if}

<div class="grid min-w-0 gap-3 min-[56.25rem]:grid-cols-[minmax(0,1.15fr)_minmax(20rem,0.85fr)]">
    <section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Input slots
                </h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                    {ports.length}
                    {ports.length === 1 ? 'port' : 'ports'} available
                </p>
            </div>
            <div class="flex items-center gap-2">
                <span
                    class="bg-surface-count text-foreground-muted rounded-full px-2.5 py-1 font-mono text-sm">
                    {slots.filter((slot) => slot.type === 'Connected').length} / {slots.length}
                </span>
                <button
                    class="border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50"
                    type="button"
                    aria-label="Refresh MIDI inputs"
                    disabled={refreshing}
                    onclick={() => void refresh()}>
                    <Icon
                        icon={refreshing
                            ? 'lucide:loader-circle'
                            : refreshDone
                              ? 'lucide:check'
                              : 'lucide:refresh-cw'}
                        width="1rem"
                        class={refreshing ? 'animate-spin' : refreshDone ? 'text-success' : ''} />
                </button>
            </div>
        </header>

        {#if slots.length === 0}
            <div class="grid min-h-40 place-items-center p-5 text-center">
                <div>
                    <Icon
                        icon="lucide:unplug"
                        width="1.5rem"
                        class="text-foreground-inactive mx-auto mb-2" />
                    <p class="text-foreground-muted m-0 text-sm">
                        No MIDI input slots are configured.
                    </p>
                </div>
            </div>
        {:else}
            <div class="divide-border divide-y">
                {#each slots as slot, slotId}
                    <article
                        class="grid gap-3 px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(7rem,0.45fr)_minmax(0,1fr)_auto] min-[37.5rem]:items-center min-[56.25rem]:px-5">
                        <div class="flex min-w-0 items-center gap-3">
                            <span
                                class={`grid size-8.5 shrink-0 place-items-center rounded-lg ${slot.type === 'Connected' ? 'bg-success/10 text-success' : 'bg-surface-icon text-foreground-muted'}`}>
                                <Icon icon="mdi:midi-port" width="1.125rem" />
                            </span>
                            <div class="min-w-0">
                                <strong class="block text-sm">Slot {slotId + 1}</strong>
                                <small class="text-foreground-muted mt-0.5 block text-sm">
                                    {slot.type === 'Connected' ? 'Connected' : 'Available'}
                                </small>
                            </div>
                        </div>

                        {#if slot.type === 'Connected'}
                            <div class="min-w-0">
                                <strong class="block truncate text-sm font-medium">
                                    {portName(slot.data.port_id)}
                                </strong>
                                <small
                                    class="text-foreground-muted mt-0.5 block truncate font-mono text-xs">
                                    {slot.data.port_id}
                                </small>
                            </div>
                            <button
                                class="border-border-strong text-foreground-muted hover:border-danger/50 hover:bg-danger/10 hover:text-danger-foreground min-h-9 cursor-pointer rounded-lg border bg-transparent px-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
                                type="button"
                                disabled={busySlotId !== null}
                                onclick={() => void disconnect(slotId)}>
                                {busySlotId === slotId ? 'Disconnecting…' : 'Disconnect'}
                            </button>
                        {:else}
                            <select
                                class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
                                aria-label={`Port for MIDI input slot ${slotId + 1}`}
                                bind:value={selectedPorts[slotId]}>
                                <option value="">Select an input port</option>
                                {#each ports as port}
                                    <option
                                        value={port.id}
                                        disabled={connectedPortIds.has(port.id)}>
                                        {port.name}{connectedPortIds.has(port.id)
                                            ? ' · Connected'
                                            : ''}
                                    </option>
                                {/each}
                            </select>
                            <button
                                class="bg-accent text-accent-foreground min-h-9 cursor-pointer rounded-lg border-0 px-3 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50"
                                type="button"
                                disabled={!selectedPorts[slotId] || busySlotId !== null}
                                onclick={() => void connect(slotId)}>
                                {busySlotId === slotId ? 'Connecting…' : 'Connect'}
                            </button>
                        {/if}
                    </article>
                {/each}
            </div>
        {/if}
    </section>

    <section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
        <header
            class="border-border min-h-16.75 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Auto-connections
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                Reconnect known devices when they become available.
            </p>
        </header>

        <form
            class="border-border grid gap-3 border-b p-3.75 min-[56.25rem]:p-5"
            onsubmit={(event) => {
                event.preventDefault();
                void addAutoConnection();
            }}>
            <label class="grid gap-1.5">
                <span class="text-sm font-semibold">Input port</span>
                <input
                    class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
                    type="text"
                    list="midi-input-port-hints"
                    placeholder="MIDI port ID"
                    autocomplete="off"
                    bind:value={autoPortId} />
                <datalist id="midi-input-port-hints">
                    {#each ports as port}
                        <option value={port.id}>{port.name}</option>
                    {/each}
                </datalist>
                <small class="text-foreground-muted text-xs">
                    Enter any port ID or choose a currently available input.
                </small>
            </label>

            <div class="grid gap-3 min-[31.25rem]:grid-cols-2">
                <label class="grid gap-1.5">
                    <span class="text-sm font-semibold">Preferred slot</span>
                    <select
                        class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
                        bind:value={preferredSlotId}
                        onchange={() => {
                            if (preferredSlotId === '') {
                                fixedSlot = false;
                            }
                        }}>
                        <option value="">Any available slot</option>
                        {#each slots.keys() as slotId}
                            <option value={String(slotId)}>Slot {slotId + 1}</option>
                        {/each}
                    </select>
                </label>

                <label
                    class="border-border-strong bg-surface-inset flex min-h-10 cursor-pointer items-center gap-2.5 self-end rounded-lg border px-3 text-sm has-disabled:cursor-not-allowed has-disabled:opacity-50">
                    <input
                        class="accent-accent size-4"
                        type="checkbox"
                        bind:checked={fixedSlot}
                        disabled={preferredSlotId === ''} />
                    <span>Require preferred slot</span>
                </label>
            </div>

            <button
                class="bg-accent text-accent-foreground min-h-10 cursor-pointer rounded-lg border-0 px-4 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50 min-[31.25rem]:w-fit"
                type="submit"
                disabled={!autoPortId.trim() || addingAutoConnection}>
                {addingAutoConnection ? 'Adding…' : 'Add auto-connection'}
            </button>
        </form>

        {#if autoConnections.length === 0}
            <div class="p-5 text-center">
                <p class="text-foreground-muted m-0 text-sm">
                    No automatic connections configured.
                </p>
            </div>
        {:else}
            <div class="divide-border divide-y">
                {#each autoConnections as connection (connection.port_id)}
                    <article class="flex items-center gap-3 px-3.75 py-3 min-[56.25rem]:px-5">
                        <span
                            class="bg-surface-icon text-foreground-muted grid size-8.5 shrink-0 place-items-center rounded-lg">
                            <Icon icon="lucide:refresh-cw" width="0.875rem" />
                        </span>
                        <div class="min-w-0 flex-1">
                            <strong class="block truncate text-sm font-semibold">
                                {portName(connection.port_id)}
                            </strong>
                            <small class="text-foreground-muted mt-0.5 block text-xs">
                                {connection.preferred_slot_id === null
                                    ? 'Any available slot'
                                    : `Slot ${connection.preferred_slot_id + 1}${connection.slot_id_fixed ? ' only' : ' preferred'}`}
                            </small>
                        </div>
                        <button
                            class="text-foreground-muted hover:bg-danger/10 hover:text-danger-foreground grid size-8.5 shrink-0 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            aria-label={`Delete auto-connection for ${portName(connection.port_id)}`}
                            disabled={deletingPortId !== null}
                            onclick={() => void removeAutoConnection(connection.port_id)}>
                            <Icon icon="lucide:trash-2" width="0.875rem" />
                        </button>
                    </article>
                {/each}
            </div>
        {/if}
    </section>
</div>
