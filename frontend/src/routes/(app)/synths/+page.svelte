<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import AssetPathInput from '$lib/components/AssetPathInput.svelte';
    import ClapPluginInput from '$lib/components/ClapPluginInput.svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import Button from '$lib/components/Button.svelte';
    import EffectsEditor from '$lib/components/EffectsEditor.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import {
        assetsForSynth,
        clapPlugins,
        fetchAssets,
        refreshAssets
    } from '$lib/state/assets.svelte';
    import { setGlobalEffects, setSynthEffects } from '$lib/state/effects.svelte';
    import { fetchPreferences, preferences } from '$lib/state/system.svelte';
    import {
        addSynth,
        fetchSynths,
        instrumentSlots,
        removeSynth,
        replaceSynth,
        setClapParameter,
        setFluidProgram,
        setSynthGain,
        setSynthName,
        synths,
        SynthKind,
        type SynthKind as SynthKindType,
        type SynthEntry
    } from '$lib/state/synths.svelte';
    import { unreachable } from '$lib/util';
    import type { InstrumentPushRequest as SynthPushParams } from '$lib/generated/models';
    import { errorToUserMessage } from '$lib/error';

    const colors: Record<SynthKindType, string> = {
        [SynthKind.Fluid]: 'var(--color-synth-fluid)',
        [SynthKind.Sfizz]: 'var(--color-synth-sfizz)',
        [SynthKind.Dummy]: 'var(--color-synth-dummy)',
        [SynthKind.Clap]: 'var(--color-synth-clap)'
    };
    let filter = $state<'All' | SynthKindType>('All');
    let selectedKey = $state('');
    let inspectorOpen = $state(false);
    let addOpen = $state(false);
    let addKind = $state<SynthKindType>(SynthKind.Fluid);
    let addPath = $state('');
    let addPluginId = $state('');
    let addMaxVoices = $state(256);
    let addPreloadSize = $state(65_536);
    let editName = $state('');
    let replacementKind = $state<SynthKindType>(SynthKind.Fluid);
    let replacementPath = $state('');
    let replacementPluginId = $state('');
    let replacementMaxVoices = $state(256);
    let replacementPreloadSize = $state(65_536);
    let selectedProgram = $state('');
    let busy = $state(false);
    let busyAction = $state(null as string | null);
    let nameSaving = $state(false);
    let programBusy = $state(false);
    let removing = $state(false);
    let loading = $state(true);
    let reloadingSynths = $state(false);
    let synthsReloaded = $state(false);
    let clapParameterBusy = $state(new Set<number>());

    let allSynths = $derived(synths());
    let project = $derived(activeProject());
    let visibleSynths = $derived(
        filter === 'All' ? allSynths : allSynths.filter((entry) => entry.kind === filter)
    );
    let selected = $derived(allSynths.find((entry) => key(entry) === selectedKey) ?? allSynths[0]);
    let selectedName = $derived(selected?.synth.name ?? '');
    let nameChanged = $derived(Boolean(selected && editName.trim() !== selected.synth.name));
    let references = $derived(
        selected
            ? Object.entries(instrumentSlots()).filter(([, slot]) => slot.synth_id === selected.id)
            : []
    );
    let visibleClapParameters = $derived(
        selected?.kind === SynthKind.Clap
            ? selected.synth.plugin_parameters.filter((parameter) => (parameter.flags & 4) === 0)
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

    const CLAP_PARAMETER_READ_ONLY = 1 << 3;

    async function submitClapParameter(parameterId: number, value: number) {
        if (!selected || selected.kind !== SynthKind.Clap) {
            return;
        }

        clapParameterBusy = new Set(clapParameterBusy).add(parameterId);
        try {
            await setClapParameter(selected.id, parameterId, value);
        } catch (error) {
            notify.error(errorToUserMessage(error), { title: 'Set plugin parameter' });
        } finally {
            const next = new Set(clapParameterBusy);
            next.delete(parameterId);
            clapParameterBusy = next;
        }
    }

    onMount(async () => {
        try {
            await Promise.all([fetchSynths(), fetchAssets(), fetchPreferences()]);
            addMaxVoices = suggestedMaxVoices(addKind);
            addPreloadSize = suggestedPreloadSize();
            const requestedSynth = new URL(window.location.href).searchParams.get('synth');
            const initial =
                (requestedSynth === null
                    ? undefined
                    : synths().find((entry) => entry.id === Number(requestedSynth))) ?? synths()[0];
            if (initial) {
                select(initial, requestedSynth !== null);
            }
        } catch (cause) {
            notify.error(errorToUserMessage(cause));
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

    $effect(() => {
        editName = selectedName;
    });

    function key(entry: SynthEntry) {
        return String(entry.id);
    }

    function displayName(entry: SynthEntry) {
        return entry.synth.name.trim() || `${entry.kind} instrument #${entry.id + 1}`;
    }

    function instrumentType(kind: SynthKindType) {
        switch (kind) {
            case 'Dummy':
                return 'Dummy Synth';
            case 'Fluid':
                return 'Fluid Synth';
            case 'Sfizz':
                return 'Sfizz Synth';
            case 'Clap':
                return 'CLAP Plugin';
            default:
                unreachable(kind);
        }
    }

    function suggestedMaxVoices(kind: SynthKindType) {
        const value = preferences();

        switch (kind) {
            case 'Dummy':
                return value?.dummy_max_voices ?? 32;
            case 'Fluid':
                return value?.fluid_max_voices ?? 256;
            case 'Sfizz':
                return value?.sfizz_max_voices ?? 64;
            case 'Clap':
                return 1;
            default:
                unreachable(kind);
        }
    }

    function suggestedPreloadSize() {
        return preferences()?.sfizz_preload_size ?? 65_536;
    }

    function select(entry: SynthEntry, open = true) {
        selectedKey = key(entry);
        editName = entry.synth.name;
        replacementKind = entry.kind;
        replacementPath = entry.kind === SynthKind.Dummy ? '' : entry.synth.file_path;
        replacementPluginId = entry.kind === SynthKind.Clap ? entry.synth.plugin_id : '';
        replacementMaxVoices = entry.kind === SynthKind.Clap ? 1 : entry.synth.max_voices;
        replacementPreloadSize =
            entry.kind === SynthKind.Sfizz ? entry.synth.preload_size : suggestedPreloadSize();
        selectedProgram =
            entry.kind === SynthKind.Fluid
                ? `${entry.synth.program.bank}:${entry.synth.program.preset_id}`
                : '';
        inspectorOpen = open;
    }

    function toggleAdd() {
        addOpen = !addOpen;
        if (addOpen && filter !== 'All') {
            addKind = filter;
            addMaxVoices = suggestedMaxVoices(addKind);
            addPreloadSize = suggestedPreloadSize();
        }
    }

    async function run(operation: () => Promise<unknown>, title: string) {
        if (busy) {
            return;
        }
        busy = true;
        busyAction = title;
        try {
            return await operation();
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title });
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
            return `SFZ instrument · ${entry.synth.max_voices} voices · ${entry.synth.preload_size.toLocaleString()} preload frames`;
        }
        if (entry.kind === SynthKind.Clap) {
            const parameterCount = entry.synth.plugin_parameters.filter(
                (parameter) => (parameter.flags & 4) === 0
            ).length;
            return [
                entry.synth.plugin_name || entry.synth.plugin_id,
                entry.synth.plugin_version,
                `${parameterCount} parameters`
            ]
                .filter(Boolean)
                .join(' · ');
        }
        return `Diagnostic synth · ${entry.synth.max_voices} voices`;
    }

    function formatParameterNumber(value: number) {
        if (Number.isInteger(value)) {
            return String(value);
        }
        return value.toLocaleString(undefined, { maximumFractionDigits: 4 });
    }

    function parameterValue(parameter: { value: number | null; value_text: string | null }) {
        const text = parameter.value_text?.trim();
        if (text) {
            return text;
        }
        if (parameter.value === null) {
            return 'Unavailable';
        }
        return formatParameterNumber(parameter.value);
    }

    async function submitAdd() {
        let createdId: number | undefined;
        await run(async () => {
            let addDetails = createDetails(
                addKind,
                addMaxVoices,
                addPreloadSize,
                addPath,
                addPluginId
            );

            createdId = await addSynth(addDetails);

            // Every new instrument already contains its default MIDI input controller.

            return createdId;
        }, 'Create instrument');

        const id = createdId;
        if (typeof id === 'number') {
            const added = synths().find((entry) => entry.kind === addKind && entry.id === id);
            if (added) {
                select(added);
            }
            addOpen = false;
            addPath = '';
            addPluginId = '';
        }
    }

    async function submitName() {
        if (!selected || !editName.trim() || !nameChanged || nameSaving) {
            return;
        }

        nameSaving = true;
        const name = editName.trim();
        try {
            await run(() => setSynthName(selected.id, name), 'Rename instrument');
            editName = name;
        } finally {
            nameSaving = false;
        }
    }

    async function submitReplacement() {
        if (
            !selected ||
            (replacementKind !== SynthKind.Dummy && !replacementPath.trim()) ||
            (replacementKind === SynthKind.Clap && !replacementPluginId)
        ) {
            return;
        }
        const id = selected.id;
        let succeeded = false;
        await run(async () => {
            await replaceSynth(
                id,
                createDetails(
                    replacementKind,
                    replacementMaxVoices,
                    replacementPreloadSize,
                    replacementPath.trim(),
                    replacementPluginId
                )
            );
            succeeded = true;
        }, 'Replace instrument');
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
        try {
            await setFluidProgram(selected.id, { bank, preset_id: presetId });
        } catch (cause) {
            selectedProgram = previousProgram;
            notify.error(errorToUserMessage(cause), {
                title: 'Select program'
            });
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
            notify.error(errorToUserMessage(cause), { title: 'Reload instruments' });
        } finally {
            reloadingSynths = false;
        }
    }

    function createDetails(
        type: SynthKind,
        maxVoices: number,
        preloadSize: number,
        path: string,
        pluginId: string
    ): SynthPushParams {
        switch (type) {
            case 'Dummy':
                return { type, data: { max_voices: maxVoices } };
            case 'Fluid':
                return {
                    type,
                    data: { max_voices: maxVoices, path }
                };
            case 'Sfizz':
                return {
                    type,
                    data: { max_voices: maxVoices, preload_size: preloadSize, path }
                };
            case 'Clap': {
                return {
                    type,
                    data: { path, plugin_id: pluginId }
                };
            }
            default:
                unreachable(type);
        }
    }

    function assetPlaceholder(type: SynthKind): string | undefined {
        switch (type) {
            case 'Dummy':
                return undefined;
            case 'Fluid':
                return 'fluid/soundfont.sf2';
            case 'Sfizz':
                return 'sfizz/instrument.sfz';
            case 'Clap':
                return undefined;
            default:
                unreachable(type);
        }
    }
</script>

<svelte:window onkeydown={handleProgramKeydown} />

<div
    class="min-[56.25rem]:flex min-[56.25rem]:h-full min-[56.25rem]:min-h-0 min-[56.25rem]:flex-col">
    {#if addOpen}
        <form
            class={`border-accent/30 bg-surface-card mb-3 grid gap-3 rounded-xl border p-3 min-[37.5rem]:items-end ${addKind === SynthKind.Dummy ? 'min-[37.5rem]:grid-cols-[9rem_auto] min-[37.5rem]:justify-start' : addKind === SynthKind.Clap ? 'min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_auto]' : 'min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_7rem_auto]'}`}
            onsubmit={(event) => {
                event.preventDefault();
                void submitAdd();
            }}>
            <label class="grid gap-1.5 text-sm font-semibold">
                Synth type
                <SelectDropdown
                    bind:value={addKind}
                    options={[
                        { value: SynthKind.Dummy, label: 'Dummy' },
                        { value: SynthKind.Fluid, label: 'FluidSynth' },
                        { value: SynthKind.Sfizz, label: 'Sfizz' },
                        { value: SynthKind.Clap, label: 'CLAP' }
                    ]}
                    onchange={() => {
                        addPath = '';
                        addPluginId = '';
                        addMaxVoices = suggestedMaxVoices(addKind);
                        addPreloadSize = suggestedPreloadSize();
                    }}
                    ariaLabel="Synth type" />
            </label>
            {#if addKind === SynthKind.Clap}
                <label class="grid min-w-0 gap-1.5 text-sm font-semibold">
                    Plugin
                    <ClapPluginInput
                        id="new-clap-plugin"
                        plugins={clapPlugins()}
                        required
                        onreload={refreshAssets}
                        bind:filePath={addPath}
                        bind:pluginId={addPluginId} />
                </label>
            {:else if addKind !== SynthKind.Dummy}
                <label class="grid gap-1.5 text-sm font-semibold">
                    {addKind === SynthKind.Fluid ? 'SoundFont file' : 'SFZ file'}
                    <AssetPathInput
                        id="new-synth-asset"
                        assets={assetsForSynth(addKind)}
                        placeholder={assetPlaceholder(addKind)}
                        required
                        onreload={refreshAssets}
                        bind:value={addPath} />
                </label>
            {/if}
            {#if addKind === SynthKind.Fluid || addKind === SynthKind.Sfizz}<label
                    class="grid gap-1.5 text-sm font-semibold">
                    Max voices
                    <input
                        class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                        type="number"
                        min="1"
                        max={addKind === SynthKind.Sfizz ? 256 : 65535}
                        step="1"
                        required
                        bind:value={addMaxVoices} />
                </label>{/if}
            {#if addKind === SynthKind.Sfizz}
                <label class="grid gap-1.5 text-sm font-semibold">
                    Preload frames per sample
                    <input
                        class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                        type="number"
                        min="1"
                        max="4294967295"
                        step="1"
                        required
                        bind:value={addPreloadSize} />
                </label>
            {/if}
            <Button variant="primary" size="md" type="submit" {busy} disabled={busy}>
                {#if busyAction === 'Create instrument'}<Icon
                        icon="lucide:loader-circle"
                        width="0.875rem"
                        class="animate-spin" />{/if}
                {busyAction === 'Create instrument' ? 'Creating…' : 'Create'}
            </Button>
        </form>
    {/if}

    {#if project}
        <div class="mb-3">
            <EffectsEditor
                title="Global effects"
                effects={project.instrument_graph.global_effects}
                onsave={setGlobalEffects} />
        </div>
    {/if}

    <div
        class="-mx-3 mb-3 flex gap-1 overflow-x-auto px-3 pb-1 min-[37.5rem]:mx-0 min-[37.5rem]:px-0"
        role="tablist">
        {#each ['All', ...Object.values(SynthKind)] as graph}
            <button
                class={`hover:border-accent/40 hover:bg-accent/10 flex min-w-max cursor-pointer items-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors ${filter === graph ? 'border-accent/40 bg-accent/8 text-accent' : 'border-border bg-surface-card text-foreground-muted'}`}
                type="button"
                onclick={() => (filter = graph as typeof filter)}>
                {graph === 'All' ? 'All instruments' : graph}
                <small class="bg-surface-count grid size-4.25 place-items-center rounded text-xs">
                    {graph === 'All'
                        ? allSynths.length
                        : allSynths.filter((entry) => entry.kind === graph).length}
                </small>
            </button>
        {/each}
    </div>

    <div
        class="min-[56.25rem]:grid min-[56.25rem]:min-h-0 min-[56.25rem]:flex-1 min-[56.25rem]:grid-cols-[minmax(16rem,2fr)_minmax(20rem,3fr)] min-[56.25rem]:items-stretch min-[56.25rem]:gap-3.5">
        <section
            class="border-border bg-surface-card rounded-xl border min-[56.25rem]:flex min-[56.25rem]:min-h-0 min-[56.25rem]:flex-col min-[56.25rem]:overflow-hidden">
            <header
                class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
                <div class="flex items-baseline gap-2">
                    <h2 class="m-0 text-sm font-semibold">Instruments</h2>
                    <span class="text-foreground-muted text-xs">
                        {visibleSynths.length} total
                    </span>
                </div>
                <div class="flex items-center gap-1.5">
                    <Button
                        variant="secondary"
                        size="sm"
                        square
                        class="size-8.5"
                        type="button"
                        aria-label={addOpen ? 'Cancel adding instrument' : 'Add instrument'}
                        title={addOpen ? 'Cancel' : 'Add instrument'}
                        onclick={toggleAdd}>
                        <Icon icon={addOpen ? 'lucide:x' : 'lucide:plus'} width="1.0625rem" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="sm"
                        square
                        class="size-8.5"
                        type="button"
                        aria-label="Reload instruments"
                        busy={reloadingSynths}
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
                    </Button>
                </div>
            </header>
            <div
                class="grid gap-1.75 p-1.75 min-[56.25rem]:min-h-0 min-[56.25rem]:flex-1 min-[56.25rem]:content-start min-[56.25rem]:overflow-y-auto">
                {#if loading}<p class="text-foreground-muted m-3 text-sm">Loading instruments…</p>
                {:else}{#each visibleSynths as entry (key(entry))}
                        <article
                            class={`bg-surface-item grid w-full grid-cols-[minmax(0,1fr)_auto] overflow-hidden rounded-[0.5625rem] border ${key(entry) === selectedKey ? 'border-accent/40 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-transparent'}`}>
                            <button
                                class="hover:bg-surface-hover grid min-w-0 cursor-pointer grid-cols-[3.375rem_minmax(0,1fr)] items-center gap-2.5 border-0 bg-transparent px-2.75 py-3 text-left transition-colors min-[37.5rem]:grid-cols-[3.625rem_minmax(0,1fr)]"
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
                            </button>
                            <a
                                class="border-border text-foreground-muted hover:bg-surface-hover hover:text-accent flex cursor-pointer items-center gap-2 border-0 border-l bg-transparent px-3 text-xs transition-colors"
                                href={`/controllers?synth=${entry.id}`}
                                title="Manage controllers"
                                aria-label={`Manage controllers for ${displayName(entry)}`}>
                                <span class="hidden whitespace-nowrap min-[37.5rem]:inline">
                                    {entry.controller_count}
                                    {entry.controller_count === 1 ? 'controller' : 'controllers'}
                                </span>
                                <Icon
                                    class="shrink-0"
                                    icon="lucide:arrow-down-right"
                                    width="1rem" />
                            </a>
                        </article>
                    {:else}<p class="text-foreground-muted m-3 text-sm">
                            No matching instruments.
                        </p>{/each}{/if}
            </div>
        </section>

        {#if selected}
            <aside
                class={`border-border-strong bg-surface-overlay min-[56.25rem]:bg-surface-card fixed right-0 bottom-0 left-0 z-60 flex max-h-[min(90dvh,47.5rem)] min-h-0 flex-col overflow-hidden rounded-t-[1.125rem] border border-b-0 shadow-[0_-1.25rem_3.75rem_var(--overlay-shadow)] transition-transform min-[56.25rem]:static min-[56.25rem]:z-1 min-[56.25rem]:h-full min-[56.25rem]:max-h-full min-[56.25rem]:rounded-xl min-[56.25rem]:border-b min-[56.25rem]:shadow-none ${inspectorOpen ? 'translate-y-0' : 'translate-y-[110%] min-[56.25rem]:translate-y-0'}`}>
                <header
                    class="border-border flex min-h-16 shrink-0 items-center justify-between border-b px-4">
                    <div>
                        <small
                            class="text-foreground-muted mb-0.5 block text-[0.6875rem] uppercase">
                            Instrument #{selected.id + 1} · {instrumentType(selected.kind)}
                        </small>
                        <h2
                            class={`m-0 text-sm font-semibold ${selected.synth.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                            {displayName(selected)}
                        </h2>
                    </div>
                    <Button
                        variant="ghost"
                        size="md"
                        square
                        class="bg-surface-button min-[56.25rem]:hidden"
                        type="button"
                        aria-label="Close editor"
                        onclick={() => (inspectorOpen = false)}>
                        <Icon icon="lucide:x" width="1.1875rem" />
                    </Button>
                </header>
                <div class="min-h-0 flex-1 overflow-y-auto">
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
                            <div class="flex min-w-0 items-center gap-2">
                                <Button
                                    variant="secondary"
                                    size="lg"
                                    square
                                    class="text-foreground-muted enabled:hover:border-accent/40 enabled:hover:bg-accent/10 enabled:hover:text-accent"
                                    type="button"
                                    aria-label="Previous SoundFont program"
                                    title="Previous program"
                                    disabled={busy || programBusy}
                                    onclick={() => void stepProgram(-1)}>
                                    <Icon icon="lucide:arrow-left" width="1rem" />
                                </Button>
                                <SelectDropdown
                                    id="synth-program"
                                    class="flex-1"
                                    bind:value={selectedProgram}
                                    options={programSuggestions}
                                    placeholder="Select a program"
                                    disabled={busy || programBusy}
                                    searchable
                                    searchPlaceholder="Search programs"
                                    noMatchesText="No matching programs"
                                    onchange={() => void submitProgram()}
                                    ariaLabel="SoundFont program" />
                                <Button
                                    variant="secondary"
                                    size="lg"
                                    square
                                    class="text-foreground-muted enabled:hover:border-accent/40 enabled:hover:bg-accent/10 enabled:hover:text-accent"
                                    type="button"
                                    aria-label="Next SoundFont program"
                                    title="Next program"
                                    disabled={busy || programBusy}
                                    onclick={() => void stepProgram(1)}>
                                    <Icon icon="lucide:arrow-right" width="1rem" />
                                </Button>
                            </div>
                            {#if !hasSelectedProgram(selected)}
                                <small
                                    class="text-foreground-muted flex items-center gap-1.5 text-xs">
                                    <Icon icon="lucide:triangle-alert" width="0.875rem" />
                                    Using fallback because the requested program is unavailable
                                </small>
                            {/if}
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
                                title="Instrument effects"
                                effects={selected.effects}
                                onsave={(effects) => setSynthEffects(selected.id, effects)} />
                        </div>
                    {/key}
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
                            showIndicator={false}
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
                            Replace instrument
                        </span>
                        <label class="grid gap-1.5 text-sm">
                            <span class="text-foreground-muted">Type</span>
                            <SelectDropdown
                                bind:value={replacementKind}
                                options={[
                                    { value: SynthKind.Dummy, label: 'Dummy' },
                                    { value: SynthKind.Fluid, label: 'FluidSynth' },
                                    { value: SynthKind.Sfizz, label: 'Sfizz' },
                                    { value: SynthKind.Clap, label: 'CLAP' }
                                ]}
                                onchange={() => {
                                    replacementPath = '';
                                    replacementPluginId = '';
                                    replacementMaxVoices = suggestedMaxVoices(replacementKind);
                                    replacementPreloadSize = suggestedPreloadSize();
                                }}
                                ariaLabel="Replacement instrument type" />
                        </label>
                        {#if replacementKind === SynthKind.Clap}
                            <label class="grid min-w-0 gap-1.5 text-sm">
                                <span class="text-foreground-muted">Plugin</span>
                                <ClapPluginInput
                                    id="replacement-clap-plugin"
                                    plugins={clapPlugins()}
                                    compact
                                    required
                                    onreload={refreshAssets}
                                    bind:filePath={replacementPath}
                                    bind:pluginId={replacementPluginId} />
                            </label>
                        {:else if replacementKind !== SynthKind.Dummy}
                            <label class="grid gap-1.5 text-sm" for="synth-asset">
                                <span class="text-foreground-muted">
                                    {replacementKind === SynthKind.Fluid
                                        ? 'SoundFont file'
                                        : 'SFZ file'}
                                </span>
                                <AssetPathInput
                                    id="synth-asset"
                                    assets={assetsForSynth(replacementKind)}
                                    placeholder={assetPlaceholder(replacementKind)}
                                    compact
                                    onreload={refreshAssets}
                                    bind:value={replacementPath}
                                    required />
                            </label>
                        {/if}
                        {#if replacementKind === SynthKind.Fluid || replacementKind === SynthKind.Sfizz}<label
                                class="grid gap-1.5 text-sm">
                                <span class="text-foreground-muted">Max voices</span>
                                <input
                                    class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                                    type="number"
                                    min="1"
                                    max={replacementKind === SynthKind.Sfizz ? 256 : 65535}
                                    step="1"
                                    required
                                    bind:value={replacementMaxVoices} />
                            </label>{/if}
                        {#if replacementKind === SynthKind.Sfizz}
                            <label class="grid gap-1.5 text-sm">
                                <span class="text-foreground-muted">Preload frames per sample</span>
                                <input
                                    class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                                    type="number"
                                    min="1"
                                    max="4294967295"
                                    step="1"
                                    required
                                    bind:value={replacementPreloadSize} />
                            </label>
                        {/if}
                        <Button
                            variant="accent"
                            size="sm"
                            class="justify-self-end"
                            type="submit"
                            busy={busyAction === 'Replace instrument'}
                            disabled={busy}>
                            {#if busyAction === 'Replace instrument'}<Icon
                                    icon="lucide:loader-circle"
                                    width="0.875rem"
                                    class="animate-spin" />{/if}
                            {busyAction === 'Replace instrument' ? 'Replacing…' : 'Replace'}
                        </Button>
                    </form>
                    {#if selected.kind === SynthKind.Clap}
                        <section class="border-border border-b p-4">
                            <div
                                class="text-foreground-strong mb-3 text-xs font-semibold uppercase">
                                Plugin
                            </div>
                            <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-2 text-sm">
                                <dt class="text-foreground-muted">Name</dt>
                                <dd
                                    class="m-0 min-w-0 truncate"
                                    title={selected.synth.plugin_name || selected.synth.plugin_id}>
                                    {selected.synth.plugin_name || selected.synth.plugin_id}
                                </dd>
                                <dt class="text-foreground-muted">Version</dt>
                                <dd class="m-0">
                                    {selected.synth.plugin_version || 'Not reported'}
                                </dd>
                                <dt class="text-foreground-muted">File</dt>
                                <dd
                                    class="m-0 min-w-0 truncate font-mono text-xs"
                                    title={selected.synth.file_path}>
                                    {selected.synth.file_path}
                                </dd>
                                <dt class="text-foreground-muted">Plugin ID</dt>
                                <dd
                                    class="m-0 min-w-0 truncate font-mono text-xs"
                                    title={selected.synth.plugin_id}>
                                    {selected.synth.plugin_id}
                                </dd>
                            </dl>
                        </section>
                    {/if}
                    {#if selected.kind === SynthKind.Clap}
                        <details class="border-border group border-b" open>
                            <summary
                                class="hover:bg-surface-raised flex min-h-12 cursor-pointer list-none items-center justify-between gap-3 px-4 py-3 transition-colors [&::-webkit-details-marker]:hidden">
                                <span
                                    class="text-foreground-strong text-xs font-semibold uppercase">
                                    Plugin parameters
                                </span>
                                <span class="flex items-center gap-2">
                                    <small class="text-foreground-muted text-xs tabular-nums">
                                        {visibleClapParameters.length}
                                    </small>
                                    <Icon
                                        class="text-foreground-muted transition-transform group-open:rotate-180"
                                        icon="lucide:chevron-down"
                                        width="1rem" />
                                </span>
                            </summary>
                            <div class="border-border border-t px-4 py-2">
                                {#each visibleClapParameters as parameter, index (parameter.id)}
                                    <article
                                        class={`grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(5rem,auto)] items-center gap-x-3 gap-y-1 py-2.5 ${index === 0 ? '' : 'border-border/60 border-t'}`}>
                                        <div class="min-w-0">
                                            <div
                                                class="text-foreground truncate text-sm font-medium">
                                                {parameter.name || `Parameter ${parameter.id}`}
                                            </div>
                                            {#if parameter.module}
                                                <div
                                                    class="text-foreground-muted mt-0.5 truncate text-xs"
                                                    title={parameter.module}>
                                                    {parameter.module}
                                                </div>
                                            {/if}
                                        </div>
                                        <div class="flex min-w-0 items-center justify-end gap-1">
                                            <output
                                                class="bg-surface-inset text-foreground min-w-0 rounded-md px-2 py-1 text-right font-mono text-xs"
                                                title={parameterValue(parameter)}>
                                                <span class="block max-w-36 truncate">
                                                    {parameterValue(parameter)}
                                                </span>
                                            </output>
                                            {#if (parameter.flags & CLAP_PARAMETER_READ_ONLY) === 0}
                                                <Button
                                                    variant="ghost"
                                                    size="xs"
                                                    square
                                                    busy={clapParameterBusy.has(parameter.id)}
                                                    disabled={clapParameterBusy.has(parameter.id) ||
                                                        parameter.value === parameter.default_value}
                                                    aria-label={`Reset ${parameter.name || `parameter ${parameter.id}`}`}
                                                    title="Reset to default"
                                                    onclick={() =>
                                                        void submitClapParameter(
                                                            parameter.id,
                                                            parameter.default_value
                                                        )}>
                                                    <Icon
                                                        icon="lucide:rotate-ccw"
                                                        width="0.875rem" />
                                                </Button>
                                            {/if}
                                        </div>
                                        {#if (parameter.flags & CLAP_PARAMETER_READ_ONLY) === 0}
                                            <input
                                                class="accent-accent col-span-2 h-7 w-full cursor-pointer disabled:cursor-wait disabled:opacity-60"
                                                type="range"
                                                min={parameter.min_value}
                                                max={parameter.max_value}
                                                step={(parameter.flags & 1) !== 0 ? 1 : 'any'}
                                                value={parameter.value ?? parameter.default_value}
                                                disabled={clapParameterBusy.has(parameter.id)}
                                                aria-label={parameter.name ||
                                                    `Parameter ${parameter.id}`}
                                                onchange={(event) =>
                                                    void submitClapParameter(
                                                        parameter.id,
                                                        event.currentTarget.valueAsNumber
                                                    )} />
                                        {/if}
                                        <small
                                            class="text-foreground-muted col-span-2 font-mono text-[0.6875rem] tabular-nums">
                                            {formatParameterNumber(parameter.min_value)} –
                                            {formatParameterNumber(parameter.max_value)} · default
                                            {formatParameterNumber(parameter.default_value)}
                                        </small>
                                    </article>
                                {:else}
                                    <p class="text-foreground-muted my-2 text-sm">
                                        This plugin exposes no visible parameters.
                                    </p>
                                {/each}
                            </div>
                        </details>
                    {/if}
                    <section class="p-4">
                        <TimedConfirmButton
                            class="w-full"
                            label="Remove instrument"
                            confirmLabel="Confirm remove instrument"
                            busyLabel="Removing…"
                            busy={removing}
                            disabled={busy}
                            onconfirm={submitRemove} />
                    </section>
                </div>
            </aside>
        {/if}
    </div>
</div>

{#if inspectorOpen}<button
        class="bg-scrim/60 fixed inset-0 z-50 cursor-default border-0 backdrop-blur-[0.125rem] min-[56.25rem]:hidden"
        type="button"
        aria-label="Close editor"
        onclick={() => (inspectorOpen = false)}>
    </button>{/if}
