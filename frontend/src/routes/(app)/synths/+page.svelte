<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import AssetPathInput from '$lib/components/AssetPathInput.svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import DotToggle from '$lib/components/DotToggle.svelte';
    import EffectsEditor from '$lib/components/EffectsEditor.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import { assetsForSynth, fetchAssets, refreshAssets } from '$lib/state/assets.svelte';
    import { setGlobalEffects, setSynthEffects } from '$lib/state/effects.svelte';
    import {
        addInstrument,
        setInstrumentName,
        setInstrumentSynth
    } from '$lib/state/instruments.svelte';
    import { fetchPreferences, preferences } from '$lib/state/system.svelte';
    import {
        addSynth,
        fetchSynths,
        instrumentSlots,
        removeSynth,
        replaceSynth,
        setFluidProgram,
        setSynthGain,
        setSynthName,
        synths,
        SynthKind,
        type SynthKind as SynthKindType,
        type SynthEntry
    } from '$lib/state/synths.svelte';

    const colors: Record<SynthKindType, string> = {
        [SynthKind.Fluid]: 'var(--color-synth-fluid)',
        [SynthKind.Sfizz]: 'var(--color-synth-sfizz)',
        [SynthKind.Dummy]: 'var(--color-synth-dummy)'
    };
    const MATCHING_INSTRUMENT_STORAGE_KEY = 'aaw-create-matching-instrument';
    let filter = $state<'All' | SynthKindType>('All');
    let selectedKey = $state('');
    let inspectorOpen = $state(false);
    let addOpen = $state(false);
    let addKind = $state<SynthKindType>(SynthKind.Fluid);
    let addPath = $state('');
    let addMaxVoices = $state(256);
    let addMatchingInstrument = $state(true);
    let editName = $state('');
    let replacementKind = $state<SynthKindType>(SynthKind.Fluid);
    let replacementPath = $state('');
    let replacementMaxVoices = $state(256);
    let selectedProgram = $state('');
    let busy = $state(false);
    let busyAction = $state(null as string | null);
    let nameSaving = $state(false);
    let programBusy = $state(false);
    let removeConfirming = $state(false);
    let removing = $state(false);
    let loading = $state(true);
    let reloadingSynths = $state(false);
    let synthsReloaded = $state(false);
    let error = $state('');

    let allSynths = $derived(synths());
    let project = $derived(activeProject());
    let visibleSynths = $derived(
        filter === 'All' ? allSynths : allSynths.filter((entry) => entry.kind === filter)
    );
    let selected = $derived(allSynths.find((entry) => key(entry) === selectedKey) ?? allSynths[0]);
    let nameChanged = $derived(Boolean(selected && editName.trim() !== selected.synth.name));
    let references = $derived(
        selected
            ? Object.entries(instrumentSlots()).filter(([, slot]) => slot.synth_id === selected.id)
            : []
    );
    let programSuggestions = $derived(
        selected?.kind === SynthKind.Fluid
            ? selected.synth.info.banks.flatMap((bank) =>
                  bank.presets.map((preset) => ({
                      value: `${bank.number}:${preset.number}`,
                      label: preset.name,
                      selectionLabel: preset.name,
                      description: `Bank ${bank.number} · Program ${preset.number}`
                  }))
              )
            : []
    );

    onMount(async () => {
        addMatchingInstrument = localStorage.getItem(MATCHING_INSTRUMENT_STORAGE_KEY) !== 'false';
        try {
            await Promise.all([fetchSynths(), fetchAssets(), fetchPreferences()]);
            addMaxVoices = suggestedMaxVoices(addKind);
            const requestedSynth = new URL(window.location.href).searchParams.get('synth');
            const initial =
                (requestedSynth === null
                    ? undefined
                    : synths().find((entry) => entry.id === Number(requestedSynth))) ?? synths()[0];
            if (initial) {
                select(initial, requestedSynth !== null);
            }
        } catch (cause) {
            showError(cause);
        } finally {
            loading = false;
        }
    });

    $effect(() => {
        if (programBusy || selected?.kind !== SynthKind.Fluid) {
            return;
        }
        selectedProgram = `${selected.synth.program.bank}:${selected.synth.program.preset_id}`;
    });

    function key(entry: SynthEntry) {
        return String(entry.id);
    }

    function displayName(entry: SynthEntry) {
        return entry.synth.name.trim() || `Unnamed ${entry.kind.toLowerCase()} synth #${entry.id}`;
    }

    function suggestedMaxVoices(kind: SynthKindType) {
        const value = preferences();
        if (kind === SynthKind.Dummy) {
            return value?.dummy_max_voices ?? 32;
        }
        if (kind === SynthKind.Fluid) {
            return value?.fluid_max_voices ?? 256;
        }
        return value?.sfizz_max_voices ?? 64;
    }

    function rememberMatchingInstrumentChoice(enabled: boolean) {
        localStorage.setItem(MATCHING_INSTRUMENT_STORAGE_KEY, String(enabled));
    }

    function select(entry: SynthEntry, open = true) {
        selectedKey = key(entry);
        editName = entry.synth.name;
        replacementKind = entry.kind;
        replacementPath = entry.kind === SynthKind.Dummy ? '' : entry.synth.file_path;
        replacementMaxVoices = entry.synth.max_voices;
        selectedProgram =
            entry.kind === SynthKind.Fluid
                ? `${entry.synth.program.bank}:${entry.synth.program.preset_id}`
                : '';
        removeConfirming = false;
        inspectorOpen = open;
        error = '';
    }

    function toggleAdd() {
        addOpen = !addOpen;
        if (addOpen && filter !== 'All') {
            addKind = filter;
            addMaxVoices = suggestedMaxVoices(addKind);
        }
    }

    function showError(cause: unknown) {
        error = cause instanceof Error ? cause.message : String(cause);
    }

    async function run(operation: () => Promise<unknown>, title: string) {
        if (busy) {
            return;
        }
        busy = true;
        busyAction = title;
        error = '';
        try {
            return await operation();
        } catch (cause) {
            showError(cause);
            notify.error(error, { title });
        } finally {
            busy = false;
            busyAction = null;
        }
    }

    function presetName(entry: SynthEntry) {
        if (entry.kind !== SynthKind.Fluid) {
            return '';
        }
        const bank = entry.synth.info.banks.find(
            (item) => item.number === entry.synth.program.bank
        );
        return (
            bank?.presets.find((item) => item.number === entry.synth.program.preset_id)?.name ??
            'Using fallback'
        );
    }

    function hasSelectedProgram(entry: SynthEntry) {
        if (entry.kind !== SynthKind.Fluid) {
            return false;
        }
        return entry.synth.info.banks
            .find((bank) => bank.number === entry.synth.program.bank)
            ?.presets.some((preset) => preset.number === entry.synth.program.preset_id);
    }

    function details(entry: SynthEntry) {
        if (entry.kind === SynthKind.Fluid) {
            const presets = entry.synth.info.banks.reduce(
                (total, bank) => total + bank.presets.length,
                0
            );
            return `${entry.synth.info.banks.length} banks · ${presets} presets · ${entry.synth.max_voices} voices`;
        }
        if (entry.kind === SynthKind.Sfizz) {
            return `SFZ instrument · ${entry.synth.max_voices} voices`;
        }
        return `Diagnostic synth · ${entry.synth.max_voices} voices`;
    }

    function instrumentReferences(entry: SynthEntry) {
        return Object.entries(instrumentSlots()).filter(([, slot]) => slot.synth_id === entry.id);
    }

    function instrumentName(id: string, name: string) {
        return name.trim() || `Unnamed instrument #${Number(id) + 1}`;
    }

    async function submitAdd() {
        let createdId: number | undefined;
        await run(async () => {
            createdId = await addSynth(
                addKind,
                addMaxVoices,
                addKind === SynthKind.Dummy ? undefined : addPath
            );

            if (addMatchingInstrument) {
                const createdSynth = synths().find((entry) => entry.id === createdId);
                if (!createdSynth) {
                    throw new Error('The created synth could not be found');
                }

                const instrumentId = await addInstrument();
                await setInstrumentSynth(instrumentId, createdId);
                await setInstrumentName(instrumentId, createdSynth.synth.name);
                await fetchSynths(true);
            }

            return createdId;
        }, 'Create synth');

        const id = createdId;
        if (typeof id === 'number') {
            const added = synths().find((entry) => entry.kind === addKind && entry.id === id);
            if (added) {
                select(added);
            }
            addOpen = false;
            addPath = '';
        }
    }

    async function submitName() {
        if (!selected || !editName.trim() || !nameChanged || nameSaving) {
            return;
        }

        nameSaving = true;
        const name = editName.trim();
        try {
            await run(() => setSynthName(selected.id, name), 'Rename synth');
            editName = name;
        } finally {
            nameSaving = false;
        }
    }

    async function submitReplacement() {
        if (!selected || (replacementKind !== SynthKind.Dummy && !replacementPath.trim())) {
            return;
        }
        const id = selected.id;
        let succeeded = false;
        await run(async () => {
            await replaceSynth(
                replacementKind,
                id,
                replacementMaxVoices,
                replacementKind === SynthKind.Dummy ? undefined : replacementPath.trim()
            );
            succeeded = true;
        }, 'Replace synth');
        if (succeeded) {
            const replaced = synths().find((entry) => entry.id === id);
            if (replaced) {
                select(replaced);
            }
        }
    }

    async function submitProgram() {
        if (selected?.kind !== SynthKind.Fluid || busy || programBusy) {
            return;
        }

        const previousProgram = `${selected.synth.program.bank}:${selected.synth.program.preset_id}`;
        const [bank, presetId] = selectedProgram.split(':').map(Number);
        programBusy = true;
        error = '';
        try {
            await setFluidProgram(selected.id, { bank, preset_id: presetId });
        } catch (cause) {
            selectedProgram = previousProgram;
            showError(cause);
            notify.error(error, { title: 'Select program' });
        } finally {
            programBusy = false;
        }
    }

    async function stepProgram(direction: -1 | 1) {
        if (busy || programBusy || selected?.kind !== SynthKind.Fluid) {
            return;
        }

        const programs = selected.synth.info.banks.flatMap((bank) =>
            bank.presets.map((preset) => `${bank.number}:${preset.number}`)
        );
        const currentIndex = programs.indexOf(selectedProgram);
        const nextIndex = Math.max(0, Math.min(programs.length - 1, currentIndex + direction));

        if (nextIndex === currentIndex || programs[nextIndex] === undefined) {
            return;
        }

        selectedProgram = programs[nextIndex];
        await submitProgram();
    }

    function handleProgramKeydown(event: KeyboardEvent) {
        if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') {
            return;
        }

        const target = event.target;
        if (
            target instanceof HTMLElement &&
            target.id !== 'synth-program' &&
            (target.matches('input, textarea, select') || target.isContentEditable)
        ) {
            return;
        }

        event.preventDefault();
        void stepProgram(event.key === 'ArrowLeft' ? -1 : 1);
    }

    async function submitRemove() {
        if (!selected) {
            return;
        }
        const removedKey = selectedKey;
        removing = true;
        await run(() => removeSynth(selected.id), 'Remove synth');
        removing = false;
        if (!synths().some((entry) => key(entry) === removedKey)) {
            selectedKey = '';
            inspectorOpen = false;
            const first = synths()[0];
            if (first) {
                select(first, false);
            }
        }
    }

    async function reloadSynths() {
        if (reloadingSynths) {
            return;
        }
        reloadingSynths = true;
        synthsReloaded = false;
        try {
            await fetchSynths();
            synthsReloaded = true;
            setTimeout(() => {
                synthsReloaded = false;
            }, 1200);
        } catch (cause) {
            showError(cause);
            notify.error(error, { title: 'Reload synths' });
        } finally {
            reloadingSynths = false;
        }
    }
</script>

<svelte:window onkeydown={handleProgramKeydown} />

{#if addOpen}
    <form
        class="border-accent/30 bg-surface-card mb-3 grid gap-3 rounded-xl border p-3 min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_7rem_auto] min-[37.5rem]:items-end"
        onsubmit={(event) => {
            event.preventDefault();
            void submitAdd();
        }}>
        <label class="grid gap-1.5 text-sm font-semibold">
            Synth type
            <SelectDropdown
                bind:value={addKind}
                options={[
                    { value: SynthKind.Fluid, label: 'FluidSynth' },
                    { value: SynthKind.Sfizz, label: 'Sfizz' },
                    { value: SynthKind.Dummy, label: 'Dummy' }
                ]}
                onchange={() => {
                    addPath = '';
                    addMaxVoices = suggestedMaxVoices(addKind);
                }}
                ariaLabel="Synth type" />
        </label>
        <label class="grid gap-1.5 text-sm font-semibold">
            Asset path
            <AssetPathInput
                id="new-synth-asset"
                assets={assetsForSynth(addKind)}
                placeholder={addKind === SynthKind.Dummy
                    ? undefined
                    : addKind === SynthKind.Fluid
                      ? 'soundfonts/piano.sf2'
                      : 'instruments/piano.sfz'}
                disabled={addKind === SynthKind.Dummy}
                required={addKind !== SynthKind.Dummy}
                onreload={refreshAssets}
                bind:value={addPath} />
        </label>
        <label class="grid gap-1.5 text-sm font-semibold">
            Max voices
            <input
                class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                type="number"
                min="1"
                max={addKind === SynthKind.Sfizz ? 256 : 65535}
                step="1"
                required
                bind:value={addMaxVoices} />
        </label>
        <button
            class="bg-accent text-accent-foreground inline-flex h-10 items-center justify-center gap-2 rounded-lg border-0 px-4 text-sm font-bold"
            disabled={busy}>
            {#if busyAction === 'Create synth'}<Icon
                    icon="lucide:loader-circle"
                    width="0.875rem"
                    class="animate-spin" />{/if}
            {busyAction === 'Create synth' ? 'Creating…' : 'Create'}
        </button>
        <DotToggle
            class="min-[37.5rem]:col-span-4 min-[37.5rem]:justify-self-start"
            label="Create matching instrument and assign this synth"
            disabled={busy}
            bind:checked={addMatchingInstrument}
            onchange={rememberMatchingInstrumentChoice} />
    </form>
{/if}

{#if error}<div
        class="border-danger/30 bg-danger/8 text-danger mb-3 rounded-lg border px-3 py-2 text-sm">
        {error}
    </div>{/if}

{#if project}
    <div class="mb-3">
        <EffectsEditor
            title="Global effects"
            effects={project.synth_graph.global_effects}
            onsave={setGlobalEffects} />
    </div>
{/if}

<div
    class="-mx-3 mb-3 flex gap-1 overflow-x-auto px-3 pb-1 min-[37.5rem]:mx-0 min-[37.5rem]:px-0"
    role="tablist">
    {#each ['All', ...Object.values(SynthKind)] as graph}
        <button
            class={`flex min-w-max cursor-pointer items-center gap-2 rounded-lg border px-3 py-2 text-sm ${filter === graph ? 'border-accent/40 bg-accent/8 text-accent' : 'border-border bg-surface-card text-foreground-muted'}`}
            type="button"
            onclick={() => (filter = graph as typeof filter)}>
            {graph === 'All' ? 'All synths' : graph}
            <small class="bg-surface-count grid size-4.25 place-items-center rounded text-xs">
                {graph === 'All'
                    ? allSynths.length
                    : allSynths.filter((entry) => entry.kind === graph).length}
            </small>
        </button>
    {/each}
</div>

<div
    class="min-[56.25rem]:grid min-[56.25rem]:grid-cols-[minmax(0,1fr)_20rem] min-[56.25rem]:items-start min-[56.25rem]:gap-3.5">
    <section class="border-border bg-surface-card rounded-xl border">
        <header class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
            <div class="flex items-baseline gap-2">
                <h2 class="m-0 text-sm font-semibold">Synths</h2>
                <span class="text-foreground-muted text-xs">
                    {visibleSynths.length} total
                </span>
            </div>
            <div class="flex items-center gap-1.5">
                <button
                    class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover grid size-8.5 cursor-pointer place-items-center rounded-lg border transition-colors"
                    type="button"
                    aria-label={addOpen ? 'Cancel adding synth' : 'Add synth'}
                    title={addOpen ? 'Cancel' : 'Add synth'}
                    onclick={toggleAdd}>
                    <Icon icon={addOpen ? 'lucide:x' : 'lucide:plus'} width="1.0625rem" />
                </button>
                <button
                    class="text-foreground-muted hover:bg-surface-hover grid size-8.5 cursor-pointer place-items-center rounded-lg border-0 bg-transparent"
                    type="button"
                    aria-label="Reload synths"
                    disabled={reloadingSynths}
                    onclick={() => void reloadSynths()}>
                    <Icon
                        icon={loading || reloadingSynths
                            ? 'lucide:loader-circle'
                            : synthsReloaded
                              ? 'lucide:check'
                              : 'lucide:refresh-cw'}
                        width="1.0625rem"
                        class={loading || reloadingSynths
                            ? 'animate-spin'
                            : synthsReloaded
                              ? 'text-success'
                              : ''} />
                </button>
            </div>
        </header>
        <div class="grid gap-1.75 p-1.75">
            {#if loading}<p class="text-foreground-muted m-3 text-sm">Loading synths…</p>
            {:else}{#each visibleSynths as entry (key(entry))}
                    {@const entryReferences = instrumentReferences(entry)}
                    {@const singleReference =
                        entryReferences.length === 1 ? entryReferences[0] : undefined}
                    <article
                        class={`bg-surface-item grid w-full grid-cols-1 overflow-hidden rounded-[0.5625rem] border min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(7.5rem,11rem)] ${key(entry) === selectedKey ? 'border-accent/40 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-transparent'}`}>
                        <button
                            class="grid min-w-0 cursor-pointer grid-cols-[3.375rem_minmax(0,1fr)_1.125rem] items-center gap-2.5 border-0 bg-transparent px-2.75 py-3 text-left min-[37.5rem]:grid-cols-[3.625rem_minmax(0,1fr)_minmax(7.5rem,.7fr)_1.125rem]"
                            type="button"
                            onclick={() => select(entry)}>
                            <span
                                class="grid h-10.75 place-items-center rounded-lg border text-xs font-bold uppercase"
                                style:border-color={`color-mix(in srgb, ${colors[entry.kind]} 35%, transparent)`}
                                style:background={`color-mix(in srgb, ${colors[entry.kind]} 10%, transparent)`}
                                style:color={colors[entry.kind]}>
                                {entry.kind}
                            </span>
                            <div class="min-w-0">
                                <strong
                                    class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap ${entry.synth.name.trim() ? '' : 'text-foreground-muted italic'}`}>
                                    {displayName(entry)}
                                </strong>
                                <small
                                    class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap">
                                    {entry.kind === SynthKind.Dummy
                                        ? details(entry)
                                        : entry.synth.file_path}
                                </small>
                            </div>
                            <div class="hidden min-w-0 min-[37.5rem]:block">
                                <small class="text-foreground-meta mb-1 block text-xs uppercase">
                                    {entry.kind === SynthKind.Fluid
                                        ? 'Selected program'
                                        : 'Details'}
                                </small>
                                <strong
                                    class="text-foreground-strong block overflow-hidden text-xs text-ellipsis whitespace-nowrap">
                                    {entry.kind === SynthKind.Fluid
                                        ? presetName(entry)
                                        : details(entry)}
                                </strong>
                            </div>
                            <Icon
                                class="justify-self-end"
                                icon="lucide:chevron-right"
                                width="1.0625rem" />
                        </button>
                        {#if singleReference}
                            <a
                                class="border-border text-foreground-muted hover:bg-surface-hover hover:text-accent hidden min-w-0 items-center gap-2 border-l px-3 text-xs transition-colors min-[37.5rem]:flex"
                                href={`/instruments?instrument=${singleReference[0]}`}
                                title={`Open ${instrumentName(singleReference[0], singleReference[1].name)}`}>
                                <span class="min-w-0 flex-1 truncate">
                                    {instrumentName(singleReference[0], singleReference[1].name)}
                                </span>
                                <Icon class="shrink-0" icon="lucide:arrow-up-right" width="1rem" />
                            </a>
                        {:else}
                            <span
                                class="border-border hidden items-center justify-end gap-1.5 border-l px-3 whitespace-nowrap min-[37.5rem]:flex">
                                <strong class="font-mono text-sm">{entryReferences.length}</strong>
                                <small class="text-foreground-muted text-xs">instruments</small>
                            </span>
                        {/if}
                    </article>
                {:else}<p class="text-foreground-muted m-3 text-sm">
                        No matching synths.
                    </p>{/each}{/if}
        </div>
    </section>

    {#if selected}
        <aside
            class={`border-border-strong bg-surface-overlay min-[56.25rem]:bg-surface-card fixed right-0 bottom-0 left-0 z-60 max-h-[min(90vh,47.5rem)] overflow-hidden rounded-t-[1.125rem] border border-b-0 shadow-[0_-1.25rem_3.75rem_var(--overlay-shadow)] transition-transform min-[56.25rem]:sticky min-[56.25rem]:top-20.5 min-[56.25rem]:z-1 min-[56.25rem]:max-h-[calc(100vh-6rem)] min-[56.25rem]:rounded-xl min-[56.25rem]:border-b min-[56.25rem]:shadow-none ${inspectorOpen ? 'translate-y-0' : 'translate-y-[110%] min-[56.25rem]:translate-y-0'}`}>
            <header class="border-border flex min-h-16 items-center justify-between border-b px-4">
                <div>
                    <small class="text-foreground-muted mb-0.5 block text-[0.6875rem] uppercase">
                        {selected.kind} synth · Synth #{selected.id}
                    </small>
                    <h2
                        class={`m-0 text-sm font-semibold ${selected.synth.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                        {displayName(selected)}
                    </h2>
                </div>
                <button
                    class="bg-surface-button grid size-9 place-items-center rounded-lg border-0 min-[56.25rem]:hidden"
                    type="button"
                    aria-label="Close editor"
                    onclick={() => (inspectorOpen = false)}>
                    <Icon icon="lucide:x" width="1.1875rem" />
                </button>
            </header>
            <div
                class="max-h-[calc(min(90vh,47.5rem)-4rem)] overflow-y-auto pb-[max(1rem,env(safe-area-inset-bottom))] min-[56.25rem]:max-h-[calc(100vh-10rem)]">
                <form
                    class="border-border grid gap-2 border-b p-4"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void submitName();
                    }}>
                    <label
                        for="synth-name"
                        class="text-foreground-strong text-xs font-semibold uppercase">
                        Name
                    </label>
                    <AutoSaveTextInput
                        id="synth-name"
                        bind:value={editName}
                        pending={nameChanged}
                        saving={nameSaving}
                        required
                        oncommit={submitName} />
                </form>
                <form
                    class="border-border grid gap-3 border-b p-4"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void submitReplacement();
                    }}>
                    <span class="text-foreground-strong text-xs font-semibold uppercase">
                        Replace synth
                    </span>
                    <label class="grid gap-1.5 text-sm">
                        <span class="text-foreground-muted">Type</span>
                        <SelectDropdown
                            bind:value={replacementKind}
                            options={[
                                { value: SynthKind.Fluid, label: 'FluidSynth' },
                                { value: SynthKind.Sfizz, label: 'Sfizz' },
                                { value: SynthKind.Dummy, label: 'Dummy' }
                            ]}
                            onchange={() => {
                                replacementPath = '';
                                replacementMaxVoices = suggestedMaxVoices(replacementKind);
                            }}
                            ariaLabel="Replacement synth type" />
                    </label>
                    <label class="grid gap-1.5 text-sm" for="synth-asset">
                        <span class="text-foreground-muted">Asset</span>
                        <AssetPathInput
                            id="synth-asset"
                            assets={assetsForSynth(replacementKind)}
                            placeholder={replacementKind === SynthKind.Dummy
                                ? undefined
                                : replacementKind === SynthKind.Fluid
                                  ? 'soundfonts/piano.sf2'
                                  : 'instruments/piano.sfz'}
                            disabled={replacementKind === SynthKind.Dummy}
                            compact
                            onreload={refreshAssets}
                            bind:value={replacementPath}
                            required={replacementKind !== SynthKind.Dummy} />
                    </label>
                    <label class="grid gap-1.5 text-sm">
                        <span class="text-foreground-muted">Max voices</span>
                        <input
                            class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                            type="number"
                            min="1"
                            max={replacementKind === SynthKind.Sfizz ? 256 : 65535}
                            step="1"
                            required
                            bind:value={replacementMaxVoices} />
                    </label>
                    <button
                        class="text-accent hover:bg-accent/10 inline-flex cursor-pointer items-center gap-2 justify-self-end rounded-lg border-0 bg-transparent px-2.5 py-2 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                        disabled={busy}>
                        {#if busyAction === 'Replace synth'}<Icon
                                icon="lucide:loader-circle"
                                width="0.875rem"
                                class="animate-spin" />{/if}
                        {busyAction === 'Replace synth' ? 'Replacing…' : 'Replace'}
                    </button>
                </form>
                {#if selected.kind === SynthKind.Fluid}<section
                        class="border-border grid gap-2 border-b p-4">
                        <div class="flex min-h-4 items-center justify-between gap-2">
                            <label
                                for="synth-program"
                                class="text-foreground-strong text-xs font-semibold uppercase">
                                SoundFont program
                            </label>
                            {#if programBusy}<Icon
                                    icon="lucide:loader-circle"
                                    width="1rem"
                                    class="text-accent animate-spin"
                                    aria-label="Setting program" />{/if}
                        </div>
                        <SelectDropdown
                            id="synth-program"
                            bind:value={selectedProgram}
                            options={programSuggestions}
                            placeholder="Select a program"
                            disabled={busy || programBusy}
                            searchable
                            searchPlaceholder="Search programs"
                            noMatchesText="No matching programs"
                            onchange={() => void submitProgram()}
                            ariaLabel="SoundFont program" />
                        {#if !hasSelectedProgram(selected)}
                            <small class="text-foreground-muted flex items-center gap-1.5 text-xs">
                                <Icon icon="lucide:triangle-alert" width="0.875rem" />
                                Using fallback because the requested program is unavailable
                            </small>
                        {/if}
                        <small class="text-foreground-muted flex items-center gap-1.5 text-xs">
                            <Icon icon="lucide:arrow-left" width="0.875rem" />
                            <span>Previous</span>
                            <span aria-hidden="true">·</span>
                            <span>Next</span>
                            <Icon icon="lucide:arrow-right" width="0.875rem" />
                        </small>
                    </section>
                {/if}
                {#key selected.id}
                    <div class="border-border border-b p-4">
                        <NumericControl
                            compact
                            label="Gain"
                            value={linearGainToDecibels(selected.gain)}
                            unit="dB"
                            smallStep={0.5}
                            largeStep={5}
                            resetValue={0}
                            min={-100}
                            max={100}
                            precision={1}
                            onchange={async (value) => {
                                await setSynthGain(selected.id, decibelsToLinearGain(value));
                                return value;
                            }} />
                    </div>
                    <div class="border-border border-b p-4">
                        <EffectsEditor
                            title="Synth effects"
                            effects={selected.effects}
                            onsave={(effects) => setSynthEffects(selected.id, effects)} />
                    </div>
                {/key}
                <section class="border-border border-b p-4">
                    <div
                        class="text-foreground-strong mb-3 flex justify-between text-xs font-semibold uppercase">
                        <span>Used by instruments</span>
                        <small class="text-accent">{references.length}</small>
                    </div>
                    <div class="grid gap-1">
                        {#each references as [id, slot]}<a
                                class="bg-surface-raised grid grid-cols-[minmax(0,1fr)_1rem] items-center rounded-md p-2 text-sm"
                                href={`/instruments?instrument=${id}`}>
                                <span
                                    class={slot.name.trim() ? '' : 'text-foreground-muted italic'}>
                                    {slot.name.trim() || `Unnamed instrument #${Number(id) + 1}`}
                                </span>
                                <Icon icon="lucide:chevron-right" width="0.875rem" />
                            </a>{:else}<p class="text-foreground-muted m-0 text-xs">
                                Not assigned to any instrument
                            </p>{/each}
                    </div>
                </section>
                {#if removeConfirming}
                    <section class="border-danger/30 bg-danger/8 m-4 rounded-lg border p-3">
                        <strong class="text-danger block text-sm">
                            Remove {displayName(selected)}?
                        </strong>
                        <p class="text-foreground-muted mt-1 mb-3 text-xs">
                            This synth will be removed from the active project.
                        </p>
                        {#if error}
                            <p class="text-danger mt-0 mb-3 text-xs">{error}</p>
                        {/if}
                        <div class="flex justify-end gap-2">
                            <button
                                class="border-border-strong bg-surface-card text-foreground hover:bg-surface-hover min-h-9 cursor-pointer rounded-lg border px-3 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                                type="button"
                                disabled={removing}
                                onclick={() => (removeConfirming = false)}>
                                Cancel
                            </button>
                            <button
                                class="bg-danger text-accent-foreground hover:bg-danger-foreground min-h-9 cursor-pointer rounded-lg border-0 px-3 text-sm font-bold transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                                type="button"
                                disabled={removing}
                                onclick={() => void submitRemove()}>
                                <span class="inline-flex items-center gap-2">
                                    {#if removing}<Icon
                                            icon="lucide:loader-circle"
                                            width="0.875rem"
                                            class="animate-spin" />{/if}
                                    {removing ? 'Removing…' : 'Remove'}
                                </span>
                            </button>
                        </div>
                    </section>
                {:else}
                    <button
                        class="text-danger hover:bg-danger/10 mx-auto my-4 block cursor-pointer rounded-lg border-0 bg-transparent px-3 py-2 text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                        type="button"
                        disabled={busy}
                        onclick={() => (removeConfirming = true)}>
                        Remove synth
                    </button>
                {/if}
            </div>
        </aside>
    {/if}
</div>

{#if inspectorOpen}<button
        class="bg-scrim/60 fixed inset-0 z-50 border-0 backdrop-blur-[0.125rem] min-[56.25rem]:hidden"
        type="button"
        aria-label="Close editor"
        onclick={() => (inspectorOpen = false)}>
    </button>{/if}
