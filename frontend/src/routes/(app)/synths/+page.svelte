<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import AssetPathInput from '$lib/components/AssetPathInput.svelte';
    import EffectsEditor from '$lib/components/EffectsEditor.svelte';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import { assetsForSynth, fetchAssets, refreshAssets } from '$lib/state/assets.svelte';
    import { setGlobalEffects, setSynthEffects } from '$lib/state/effects.svelte';
    import {
        addSynth,
        fetchSynths,
        instrumentSlots,
        removeSynth,
        replaceSynth,
        setFluidProgram,
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
    let filter = $state<'All' | SynthKindType>('All');
    let selectedKey = $state('');
    let inspectorOpen = $state(false);
    let addOpen = $state(false);
    let addKind = $state<SynthKindType>(SynthKind.Fluid);
    let addPath = $state('');
    let editName = $state('');
    let replacementKind = $state<SynthKindType>(SynthKind.Fluid);
    let replacementPath = $state('');
    let selectedProgram = $state('');
    let busy = $state(false);
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
    let references = $derived(
        selected
            ? Object.entries(instrumentSlots()).filter(([, slot]) => slot.synth_id === selected.id)
            : []
    );

    onMount(async () => {
        try {
            await Promise.all([fetchSynths(), fetchAssets()]);
            const first = synths()[0];
            if (first) {
                select(first, false);
            }
        } catch (cause) {
            showError(cause);
        } finally {
            loading = false;
        }
    });

    function key(entry: SynthEntry) {
        return String(entry.id);
    }

    function displayName(entry: SynthEntry) {
        return entry.synth.name.trim() || `Unnamed ${entry.kind.toLowerCase()} synth #${entry.id}`;
    }

    function select(entry: SynthEntry, open = true) {
        selectedKey = key(entry);
        editName = entry.synth.name;
        replacementKind = entry.kind;
        replacementPath = entry.kind === SynthKind.Dummy ? '' : entry.synth.file_path;
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
        }
    }

    function showError(cause: unknown) {
        error = cause instanceof Error ? cause.message : String(cause);
    }

    async function run(operation: () => Promise<unknown>, title: string, successMessage: string) {
        if (busy) {
            return;
        }
        busy = true;
        error = '';
        const notification = notify.info('In progress…', {
            duration: null,
            loading: true,
            title
        });
        try {
            const result = await operation();
            notification.success(successMessage);
            return result;
        } catch (cause) {
            showError(cause);
            notification.error(error);
        } finally {
            busy = false;
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
            `Preset ${entry.synth.program.preset_id}`
        );
    }

    function details(entry: SynthEntry) {
        if (entry.kind === SynthKind.Fluid) {
            const presets = entry.synth.info.banks.reduce(
                (total, bank) => total + bank.presets.length,
                0
            );
            return `${entry.synth.info.banks.length} banks · ${presets} presets`;
        }
        if (entry.kind === SynthKind.Sfizz) {
            return 'SFZ instrument';
        }
        return 'Diagnostic synth';
    }

    function referenceCount(entry: SynthEntry) {
        return Object.values(instrumentSlots()).filter((slot) => slot.synth_id === entry.id).length;
    }

    async function submitAdd() {
        const id = await run(
            () => addSynth(addKind, addKind === SynthKind.Dummy ? undefined : addPath),
            'Create synth',
            'Synth created.'
        );
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
        if (selected && editName.trim()) {
            await run(
                () => setSynthName(selected.id, editName.trim()),
                'Rename synth',
                'Synth renamed.'
            );
        }
    }

    async function submitReplacement() {
        if (!selected || (replacementKind !== SynthKind.Dummy && !replacementPath.trim())) {
            return;
        }
        const id = selected.id;
        let succeeded = false;
        await run(
            async () => {
                await replaceSynth(
                    replacementKind,
                    id,
                    replacementKind === SynthKind.Dummy ? undefined : replacementPath.trim()
                );
                succeeded = true;
            },
            'Replace synth',
            'Synth replaced.'
        );
        if (succeeded) {
            const replaced = synths().find((entry) => entry.id === id);
            if (replaced) {
                select(replaced);
            }
        }
    }

    async function submitProgram() {
        if (selected?.kind !== SynthKind.Fluid || programBusy) {
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
        await run(() => removeSynth(selected.id), 'Remove synth', 'Synth removed.');
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
        class="border-accent/30 bg-surface-card mb-3 grid gap-3 rounded-xl border p-3 min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_auto] min-[37.5rem]:items-end"
        onsubmit={(event) => {
            event.preventDefault();
            void submitAdd();
        }}>
        <label class="grid gap-1.5 text-sm font-semibold">
            Synth type
            <select
                class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5"
                bind:value={addKind}>
                <option value={SynthKind.Fluid}>FluidSynth</option>
                <option value={SynthKind.Sfizz}>Sfizz</option>
                <option value={SynthKind.Dummy}>Dummy</option>
            </select>
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
        <button
            class="bg-accent text-accent-foreground h-10 rounded-lg border-0 px-4 text-sm font-bold"
            disabled={busy}>
            Create
        </button>
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
            effects={project.synth_graph.effects}
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
                <h2 class="m-0 text-sm font-semibold">
                    {filter === 'All' ? 'Available synths' : `${filter} synths`}
                </h2>
                <span class="text-foreground-muted text-xs">
                    {visibleSynths.length} total
                </span>
            </div>
            <div class="flex items-center gap-1.5">
                <button
                    class="bg-accent text-accent-foreground grid size-8.5 cursor-pointer place-items-center rounded-lg border-0"
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
                    <button
                        class={`bg-surface-item grid w-full cursor-pointer grid-cols-[3.375rem_minmax(0,1fr)_1.125rem] items-center gap-2.5 rounded-[0.5625rem] border px-2.75 py-3 text-left min-[37.5rem]:grid-cols-[3.625rem_minmax(0,1fr)_minmax(7.5rem,.7fr)_3.25rem_1.125rem] ${key(entry) === selectedKey ? 'border-accent/40 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-transparent'}`}
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
                                {entry.kind === SynthKind.Fluid ? 'Selected program' : 'Details'}
                            </small>
                            <strong
                                class="text-foreground-strong block overflow-hidden text-xs text-ellipsis whitespace-nowrap">
                                {entry.kind === SynthKind.Fluid
                                    ? presetName(entry)
                                    : details(entry)}
                            </strong>
                        </div>
                        <span class="hidden text-center min-[37.5rem]:block">
                            <strong class="block font-mono text-sm">
                                {referenceCount(entry)}
                            </strong>
                            <small class="text-foreground-muted text-xs">instruments</small>
                        </span>
                        <Icon icon="lucide:chevron-right" width="1.0625rem" />
                    </button>
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
                    <div class="flex gap-2">
                        <input
                            id="synth-name"
                            class="border-border-strong bg-surface-inset text-foreground h-9 min-w-0 flex-1 rounded-lg border px-2.5 text-sm"
                            bind:value={editName}
                            required />
                        <button
                            class="bg-accent text-accent-foreground rounded-lg border-0 px-3 text-sm font-bold"
                            disabled={busy}>
                            Save
                        </button>
                    </div>
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
                        <select
                            class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5"
                            bind:value={replacementKind}
                            onchange={() => {
                                replacementPath = '';
                            }}>
                            <option value={SynthKind.Fluid}>FluidSynth</option>
                            <option value={SynthKind.Sfizz}>Sfizz</option>
                            <option value={SynthKind.Dummy}>Dummy</option>
                        </select>
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
                    <button
                        class="text-accent hover:bg-accent/10 cursor-pointer justify-self-end rounded-lg border-0 bg-transparent px-2.5 py-2 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                        disabled={busy}>
                        Replace
                    </button>
                </form>
                {#if selected.kind === SynthKind.Fluid}<form
                        class="border-border grid gap-2 border-b p-4"
                        onsubmit={(event) => {
                            event.preventDefault();
                            void submitProgram();
                        }}>
                        <label
                            for="synth-program"
                            class="text-foreground-strong text-xs font-semibold uppercase">
                            SoundFont program
                        </label>
                        <select
                            id="synth-program"
                            class="border-border-strong bg-surface-inset text-foreground h-10 rounded-lg border px-2.5 text-sm"
                            disabled={programBusy}
                            bind:value={selectedProgram}>
                            {#each selected.synth.info.banks as bank}<optgroup
                                    label={`Bank ${bank.number}`}>
                                    {#each bank.presets as preset}<option
                                            value={`${bank.number}:${preset.number}`}>
                                            {preset.name} · {preset.number}
                                        </option>{/each}
                                </optgroup>{/each}
                        </select>
                        <small class="text-foreground-muted flex items-center gap-1.5 text-xs">
                            <Icon icon="lucide:arrow-left" width="0.875rem" />
                            <span>Previous</span>
                            <span aria-hidden="true">·</span>
                            <span>Next</span>
                            <Icon icon="lucide:arrow-right" width="0.875rem" />
                        </small>
                        <button
                            class="bg-accent text-accent-foreground flex h-9 items-center justify-center gap-2 rounded-lg border-0 px-3 text-sm font-bold"
                            disabled={busy || programBusy}>
                            {#if programBusy}
                                <Icon
                                    icon="lucide:loader-circle"
                                    width="1rem"
                                    class="animate-spin" />
                                Setting…
                            {:else}
                                Set program
                            {/if}
                        </button>
                    </form>
                {/if}
                {#key selected.id}
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
                                href="/instruments">
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
                                {removing ? 'Removing…' : 'Remove'}
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
