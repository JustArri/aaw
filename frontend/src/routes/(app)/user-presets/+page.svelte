<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import { flip } from 'svelte/animate';
    import type { StoredUserPreset } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { fetchInstrumentSlots, instrumentSlots } from '$lib/state/instruments.svelte';
    import {
        activateUserPreset,
        cloneUserPresets,
        fetchUserPresets,
        initializeUserPresetsDraft,
        resetUserPresetsDraft,
        saveUserPresets,
        userPresets,
        userPresetsDraft as draft
    } from '$lib/state/user-presets.svelte';

    let loading = $state(true);
    let busy = $state(false);
    let activating = $state<number | null>(null);
    let draggedPreset = $state<StoredUserPreset | null>(null);
    let dragX = $state(0);
    let dragY = $state(0);
    let dropInsertionIndex = $state<number | null>(null);

    let instruments = $derived(
        Object.entries(instrumentSlots())
            .map(([id, slot]) => ({ id: Number(id), slot }))
            .sort((left, right) => left.id - right.id)
    );
    let changed = $derived(JSON.stringify(draft) !== JSON.stringify(userPresets()));

    onMount(async () => {
        try {
            const [presets] = await Promise.all([fetchUserPresets(), fetchInstrumentSlots()]);
            initializeUserPresetsDraft(presets);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'Load user presets'
            });
        } finally {
            loading = false;
        }
    });

    function addPreset() {
        draft.entries.push({
            name: `Preset ${draft.entries.length + 1}`,
            instruments: instruments.filter(({ slot }) => slot.enabled).map(({ id }) => id)
        });
    }

    function removePreset(index: number) {
        draft.entries.splice(index, 1);
    }

    function movePreset(index: number, offset: number) {
        const destination = index + offset;
        if (destination < 0 || destination >= draft.entries.length) {
            return;
        }
        const [preset] = draft.entries.splice(index, 1);
        draft.entries.splice(destination, 0, preset);
    }

    function startPresetDrag(event: PointerEvent, preset: StoredUserPreset) {
        if (event.button !== 0) {
            return;
        }
        const handle = event.currentTarget as HTMLElement;
        handle.setPointerCapture(event.pointerId);
        event.preventDefault();
        draggedPreset = preset;
        dragX = event.clientX;
        dragY = event.clientY;
        dropInsertionIndex = draft.entries.indexOf(preset);
    }

    function dragPreset(event: PointerEvent) {
        if (!draggedPreset) {
            return;
        }
        dragX = event.clientX;
        dragY = event.clientY;
        const target = document
            .elementFromPoint(event.clientX, event.clientY)
            ?.closest<HTMLElement>('[data-preset-index]');
        const targetIndex = Number(target?.dataset.presetIndex);
        if (!target || !Number.isInteger(targetIndex)) {
            return;
        }
        const bounds = target.getBoundingClientRect();
        dropInsertionIndex = targetIndex + (event.clientY > bounds.top + bounds.height / 2 ? 1 : 0);
    }

    function stopPresetDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId)) {
            handle.releasePointerCapture(event.pointerId);
        }
        if (draggedPreset && dropInsertionIndex !== null) {
            const source = draft.entries.indexOf(draggedPreset);
            let destination = dropInsertionIndex;
            if (destination > source) {
                destination -= 1;
            }
            if (source >= 0 && destination !== source) {
                const [preset] = draft.entries.splice(source, 1);
                draft.entries.splice(destination, 0, preset);
            }
        }
        draggedPreset = null;
        dropInsertionIndex = null;
    }

    function cancelPresetDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId)) {
            handle.releasePointerCapture(event.pointerId);
        }
        draggedPreset = null;
        dropInsertionIndex = null;
    }

    function toggleInstrument(presetIndex: number, instrumentId: number, enabled: boolean) {
        const preset = draft.entries[presetIndex];
        if (enabled) {
            preset.instruments = [...preset.instruments, instrumentId].sort(
                (left, right) => left - right
            );
        } else {
            preset.instruments = preset.instruments.filter((id) => id !== instrumentId);
        }
    }

    function selectAllInstruments(presetIndex: number) {
        draft.entries[presetIndex].instruments = instruments.map(({ id }) => id);
    }

    function clearInstruments(presetIndex: number) {
        draft.entries[presetIndex].instruments = [];
    }

    function discardChanges() {
        resetUserPresetsDraft();
    }

    async function save() {
        if (busy || !changed) {
            return;
        }
        if (draft.entries.some((preset) => !preset.name.trim())) {
            notify.error('Every preset needs a name.', { title: 'Save user presets' });
            return;
        }
        busy = true;
        const notification = notify.info('Saving presets…', {
            duration: null,
            loading: true,
            title: 'User presets'
        });
        try {
            draft.entries = draft.entries.map((preset) => ({
                ...preset,
                name: preset.name.trim()
            }));
            await saveUserPresets(cloneUserPresets(draft));
            notification.success('Presets saved.');
        } catch (error) {
            notification.error(error instanceof Error ? error.message : String(error));
        } finally {
            busy = false;
        }
    }

    async function activate(index: number) {
        if (activating !== null || changed) {
            return;
        }
        activating = index;
        const notification = notify.info('Activating preset…', {
            duration: null,
            loading: true,
            title: draft.entries[index].name
        });
        try {
            await activateUserPreset(index);
            notification.success('Preset activated.');
        } catch (error) {
            notification.error(error instanceof Error ? error.message : String(error));
        } finally {
            activating = null;
        }
    }
</script>

<section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                User presets
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                {draft.entries.length}
                {draft.entries.length === 1 ? 'preset' : 'presets'}
            </p>
        </div>
        <div class="flex items-center gap-1">
            <button
                class="text-foreground-muted enabled:hover:bg-surface-hover grid size-9 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-default disabled:opacity-40"
                type="button"
                aria-label="Discard preset changes"
                title="Discard changes"
                disabled={!changed || busy}
                onclick={discardChanges}>
                <Icon icon="lucide:undo-2" width="1rem" />
            </button>
            <button
                class="text-accent enabled:hover:bg-accent/10 disabled:text-foreground-icon grid size-9 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-default disabled:opacity-50"
                type="button"
                aria-label="Save user presets"
                title="Save user presets"
                disabled={!changed || busy}
                onclick={() => void save()}>
                <Icon
                    icon={busy ? 'lucide:loader-circle' : 'lucide:check'}
                    width="1.125rem"
                    class={busy ? 'animate-spin' : ''} />
            </button>
            <button
                class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover grid size-9 cursor-pointer place-items-center rounded-lg border transition-colors disabled:cursor-wait disabled:opacity-60"
                type="button"
                aria-label="Add user preset"
                title="Add user preset"
                disabled={busy}
                onclick={addPreset}>
                <Icon icon="lucide:plus" width="1.125rem" />
            </button>
        </div>
    </header>

    {#if loading}
        <div class="text-foreground-muted grid min-h-52 place-items-center text-sm">
            Loading presets…
        </div>
    {:else if draft.entries.length === 0}
        <div class="text-foreground-muted grid min-h-52 place-items-center p-6 text-center">
            <div>
                <Icon icon="lucide:list-music" width="1.5rem" class="mx-auto mb-2" />
                <p class="m-0 text-sm">No user presets yet.</p>
                <button
                    class="text-accent hover:bg-accent/10 mt-3 cursor-pointer rounded-lg border-0 bg-transparent px-3 py-2 text-sm font-semibold"
                    type="button"
                    onclick={addPreset}>
                    Add first preset
                </button>
            </div>
        </div>
    {:else}
        <div class="grid p-2 min-[56.25rem]:p-3">
            {#each draft.entries as preset, index (preset)}
                <div animate:flip={{ duration: 160 }}>
                    {#if index > 0}<div class="bg-border mx-4 my-3 h-px"></div>{/if}
                    <article
                        class={`bg-surface-inset-deep relative min-w-0 rounded-xl p-4 transition-opacity ${draggedPreset === preset ? 'opacity-40' : ''}`}
                        data-preset-index={index}>
                        {#if draggedPreset && dropInsertionIndex === index}<span
                                class="bg-accent pointer-events-none absolute -top-1.5 right-2 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
                            </span>{/if}
                        {#if draggedPreset && dropInsertionIndex === draft.entries.length && index === draft.entries.length - 1}<span
                                class="bg-accent pointer-events-none absolute right-2 -bottom-1.5 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
                            </span>{/if}
                        <div class="flex min-w-0 items-center gap-2">
                            <button
                                class="text-foreground-muted hover:bg-surface-hover grid size-8 shrink-0 cursor-grab touch-none place-items-center rounded-lg border-0 bg-transparent active:cursor-grabbing"
                                type="button"
                                aria-label={`Drag ${preset.name}`}
                                title="Drag to reorder"
                                onpointerdown={(event) => startPresetDrag(event, preset)}
                                onpointermove={dragPreset}
                                onpointerup={stopPresetDrag}
                                onpointercancel={cancelPresetDrag}>
                                <Icon icon="lucide:grip-vertical" width="1rem" />
                            </button>
                            <span
                                class="text-foreground-muted grid size-6 shrink-0 place-items-center font-mono text-sm">
                                {index + 1}
                            </span>
                            <input
                                class="bg-surface-card text-foreground placeholder:text-foreground-muted h-9 min-w-0 flex-1 rounded-lg border-0 px-2.5 text-sm placeholder:italic"
                                aria-label={`Preset ${index + 1} name`}
                                placeholder={`Unnamed preset #${index + 1}`}
                                bind:value={preset.name} />
                            <button
                                class={`grid size-9 shrink-0 place-items-center rounded-lg border-0 bg-transparent ${changed ? 'text-foreground-icon cursor-default' : 'text-accent hover:bg-accent/10 cursor-pointer disabled:cursor-wait'}`}
                                type="button"
                                aria-label={`Activate ${preset.name}`}
                                title={changed
                                    ? 'Save changes before activating'
                                    : 'Activate preset'}
                                disabled={changed || activating !== null}
                                onclick={() => void activate(index)}>
                                <Icon
                                    icon={activating === index
                                        ? 'lucide:loader-circle'
                                        : 'lucide:play'}
                                    width="1rem"
                                    class={activating === index ? 'animate-spin' : ''} />
                            </button>
                        </div>

                        <div class="mt-4 grid gap-1.5">
                            <div class="grid grid-cols-2 gap-1.5">
                                {#each instruments as instrument}
                                    {@const selected = preset.instruments.includes(instrument.id)}
                                    <button
                                        class={`flex min-h-9 min-w-0 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors ${selected ? 'border-accent/50 bg-accent/10 text-foreground' : 'border-border bg-surface-card text-foreground hover:bg-surface-hover'}`}
                                        type="button"
                                        aria-pressed={selected}
                                        onclick={() => {
                                            toggleInstrument(index, instrument.id, !selected);
                                        }}>
                                        <span
                                            class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${selected ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
                                            aria-hidden="true">
                                            {#if selected}<span
                                                    class="bg-accent size-2 rounded-full">
                                                </span>{/if}
                                        </span>
                                        <span
                                            class={`truncate ${instrument.slot.name.trim() ? '' : 'text-foreground-muted italic'}`}>
                                            {instrument.slot.name.trim() ||
                                                `Unnamed instrument #${instrument.id + 1}`}
                                        </span>
                                    </button>
                                {:else}<p class="text-foreground-muted col-span-2 m-0 text-sm">
                                        No instruments available.
                                    </p>{/each}
                            </div>
                            {#if instruments.length > 0}<div class="flex justify-end gap-1.5">
                                    <button
                                        class="bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border-0 px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={instruments.every(({ id }) =>
                                            preset.instruments.includes(id)
                                        )}
                                        onclick={() => selectAllInstruments(index)}>
                                        All
                                    </button>
                                    <button
                                        class="bg-surface-button text-foreground-muted hover:bg-surface-hover hover:text-foreground min-h-7 cursor-pointer rounded-md border-0 px-2.5 text-xs font-semibold disabled:cursor-default disabled:opacity-40"
                                        type="button"
                                        disabled={preset.instruments.length === 0}
                                        onclick={() => clearInstruments(index)}>
                                        None
                                    </button>
                                </div>{/if}
                        </div>

                        <div class="mt-3 flex justify-end gap-1">
                            <button
                                class="text-foreground-muted enabled:hover:bg-surface-hover grid size-8 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-default disabled:opacity-30"
                                type="button"
                                aria-label="Move preset up"
                                title="Move up"
                                disabled={index === 0}
                                onclick={() => movePreset(index, -1)}>
                                <Icon icon="lucide:arrow-up" width="0.9375rem" />
                            </button>
                            <button
                                class="text-foreground-muted enabled:hover:bg-surface-hover grid size-8 cursor-pointer place-items-center rounded-lg border-0 bg-transparent disabled:cursor-default disabled:opacity-30"
                                type="button"
                                aria-label="Move preset down"
                                title="Move down"
                                disabled={index === draft.entries.length - 1}
                                onclick={() => movePreset(index, 1)}>
                                <Icon icon="lucide:arrow-down" width="0.9375rem" />
                            </button>
                            <button
                                class="text-danger hover:bg-danger/10 grid size-8 cursor-pointer place-items-center rounded-lg border-0 bg-transparent"
                                type="button"
                                aria-label={`Remove ${preset.name}`}
                                title="Remove preset"
                                onclick={() => removePreset(index)}>
                                <Icon icon="lucide:trash-2" width="0.9375rem" />
                            </button>
                        </div>
                    </article>
                </div>
            {/each}
        </div>
    {/if}
</section>

{#if draggedPreset}
    <div
        class="border-accent/50 bg-surface-overlay pointer-events-none fixed z-80 flex w-[min(20rem,calc(100vw-2rem))] -translate-x-4 -translate-y-4 items-center gap-2 rounded-xl border p-3 shadow-[0_1rem_2.5rem_var(--overlay-shadow)]"
        style:left={`${dragX}px`}
        style:top={`${dragY}px`}>
        <Icon icon="lucide:grip-vertical" width="1rem" class="text-accent shrink-0" />
        <span class="text-foreground min-w-0 flex-1 truncate text-sm font-semibold">
            {draggedPreset.name}
        </span>
        {#if dropInsertionIndex !== null}<small
                class="text-foreground-muted shrink-0 font-mono text-xs">
                #{Math.min(dropInsertionIndex + 1, draft.entries.length)}
            </small>{/if}
    </div>
{/if}
