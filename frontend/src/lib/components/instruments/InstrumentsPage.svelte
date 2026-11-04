<script lang="ts">
    import Icon from '@iconify/svelte';
    import AssetPathInput from '$lib/components/AssetPathInput.svelte';
    import ClapPluginInput from '$lib/components/ClapPluginInput.svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import Button from '$lib/components/Button.svelte';
    import EffectsEditor from '$lib/components/EffectsEditor.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import NumberInput from '$lib/components/NumberInput.svelte';
    import ProjectTransport from '$lib/components/ProjectTransport.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import { SynthKind, SynthsSection } from '$lib/sections/synths.svelte';

    const section = new SynthsSection();
</script>

<svelte:window onkeydown={(event) => section.handleProgramKeydown(event)} />

<div
    class="min-[56.25rem]:flex min-[56.25rem]:h-full min-[56.25rem]:min-h-0 min-[56.25rem]:flex-col">
    <ProjectTransport showBeatVisualizer class="mb-3 shrink-0" />

    {#if section.addOpen}
        <form
            class={`border-accent/30 bg-surface-card mb-3 grid gap-3 rounded-xl border p-3 min-[37.5rem]:items-end ${section.addKind === SynthKind.Dummy ? 'min-[37.5rem]:grid-cols-[9rem_auto] min-[37.5rem]:justify-start' : section.addKind === SynthKind.Clap ? 'min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_auto]' : 'min-[37.5rem]:grid-cols-[9rem_minmax(0,1fr)_7rem_auto]'}`}
            onsubmit={(event) => {
                event.preventDefault();
                void section.submitAdd();
            }}>
            <label class="grid gap-1.5 text-sm font-semibold">
                Synth type
                <SelectDropdown
                    bind:value={section.addKind}
                    options={[
                        { value: SynthKind.Dummy, label: 'Dummy' },
                        { value: SynthKind.Fluid, label: 'FluidSynth' },
                        { value: SynthKind.Sfizz, label: 'Sfizz' },
                        { value: SynthKind.Clap, label: 'CLAP' }
                    ]}
                    onchange={() => {
                        section.addPath = '';
                        section.addPluginId = '';
                        section.addMaxVoices = section.suggestedMaxVoices(section.addKind);
                        section.addPreloadSize = section.suggestedPreloadSize();
                    }}
                    ariaLabel="Synth type" />
            </label>
            {#if section.addKind === SynthKind.Clap}
                <label class="grid min-w-0 gap-1.5 text-sm font-semibold">
                    Plugin
                    <ClapPluginInput
                        id="new-clap-plugin"
                        plugins={section.clapPlugins()}
                        required
                        onreload={section.refreshAssets}
                        bind:filePath={section.addPath}
                        bind:pluginId={section.addPluginId} />
                </label>
            {:else if section.addKind !== SynthKind.Dummy}
                <label class="grid gap-1.5 text-sm font-semibold">
                    {section.addKind === SynthKind.Fluid ? 'SoundFont file' : 'SFZ file'}
                    <AssetPathInput
                        id="new-synth-asset"
                        assets={section.assetsForSynth(section.addKind)}
                        placeholder={section.assetPlaceholder(section.addKind)}
                        required
                        onreload={section.refreshAssets}
                        bind:value={section.addPath} />
                </label>
            {/if}
            {#if section.addKind === SynthKind.Fluid || section.addKind === SynthKind.Sfizz}<label
                    class="grid gap-1.5 text-sm font-semibold">
                    Max voices
                    <NumberInput
                        class="bg-surface-card w-full px-2.5 text-right"
                        min={1}
                        max={section.addKind === SynthKind.Sfizz ? 256 : 65535}
                        step={1}
                        required
                        bind:value={section.addMaxVoices} />
                </label>{/if}
            {#if section.addKind === SynthKind.Sfizz}
                <label class="grid gap-1.5 text-sm font-semibold">
                    Preload frames per sample
                    <NumberInput
                        class="bg-surface-card w-full px-2.5 text-right"
                        min={1}
                        max={4294967295}
                        step={1}
                        required
                        bind:value={section.addPreloadSize} />
                </label>
            {/if}
            <Button
                variant="primary"
                size="md"
                type="submit"
                busy={section.busy}
                disabled={section.busy}>
                {#if section.busyAction === 'Create instrument'}<Icon
                        icon="lucide:loader-circle"
                        width="0.875rem"
                        class="animate-spin" />{/if}
                {section.busyAction === 'Create instrument' ? 'Creating…' : 'Create'}
            </Button>
        </form>
    {/if}

    {#if section.project}
        <div class="mb-3">
            <EffectsEditor
                title="Global effects"
                effects={section.project.instrument_graph.global_effects}
                onsave={(effects) => section.saveGlobalEffects(effects)} />
        </div>
    {/if}

    <div
        class="-mx-3 mb-3 flex gap-1 overflow-x-auto px-3 pb-1 min-[37.5rem]:mx-0 min-[37.5rem]:px-0"
        role="tablist">
        {#each ['All', ...Object.values(SynthKind)] as graph}
            <button
                class={`hover:border-accent/40 hover:bg-accent/10 flex min-w-max cursor-pointer items-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors ${section.filter === graph ? 'border-accent/40 bg-accent/8 text-accent' : 'border-border bg-surface-card text-foreground-muted'}`}
                type="button"
                onclick={() => (section.filter = graph as typeof section.filter)}>
                {graph === 'All' ? 'All instruments' : graph}
                <small class="bg-surface-count grid size-4.25 place-items-center rounded text-xs">
                    {graph === 'All'
                        ? section.allSynths.length
                        : section.allSynths.filter((entry) => entry.kind === graph).length}
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
                        {section.visibleSynths.length} total
                    </span>
                </div>
                <div class="flex items-center gap-1.5">
                    <Button
                        variant="secondary"
                        size="sm"
                        square
                        class="size-8.5"
                        type="button"
                        aria-label={section.addOpen ? 'Cancel adding instrument' : 'Add instrument'}
                        title={section.addOpen ? 'Cancel' : 'Add instrument'}
                        onclick={() => section.toggleAdd()}>
                        <Icon
                            icon={section.addOpen ? 'lucide:x' : 'lucide:plus'}
                            width="1.0625rem" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="sm"
                        square
                        class="size-8.5"
                        type="button"
                        aria-label="Reload instruments"
                        busy={section.reloadingSynths}
                        disabled={section.reloadingSynths}
                        onclick={() => void section.reloadSynths()}>
                        <Icon
                            icon={section.loading || section.reloadingSynths
                                ? 'lucide:loader-circle'
                                : section.synthsReloaded
                                  ? 'lucide:check'
                                  : 'lucide:refresh-cw'}
                            width="1.0625rem"
                            class={section.loading || section.reloadingSynths
                                ? 'animate-spin'
                                : section.synthsReloaded
                                  ? 'text-success'
                                  : ''} />
                    </Button>
                </div>
            </header>
            <div
                class="grid gap-1.75 p-1.75 min-[56.25rem]:min-h-0 min-[56.25rem]:flex-1 min-[56.25rem]:content-start min-[56.25rem]:overflow-y-auto">
                {#if section.loading}<p class="text-foreground-muted m-3 text-sm">
                        Loading instruments…
                    </p>
                {:else}{#each section.visibleSynths as entry (section.key(entry))}
                        <article
                            class={`bg-surface-item grid w-full grid-cols-[minmax(0,1fr)_auto] overflow-hidden rounded-[0.5625rem] border ${section.key(entry) === section.selectedKey ? 'border-accent/40 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-transparent'}`}>
                            <button
                                class="hover:bg-surface-hover grid min-w-0 cursor-pointer grid-cols-[3.375rem_minmax(0,1fr)] items-center gap-2.5 border-0 bg-transparent px-2.75 py-3 text-left transition-colors min-[37.5rem]:grid-cols-[3.625rem_minmax(0,1fr)]"
                                type="button"
                                onclick={() => section.select(entry)}>
                                <span
                                    class="grid h-10.75 place-items-center rounded-lg border text-xs font-bold uppercase"
                                    style:border-color={`color-mix(in srgb, ${section.colors[entry.kind]} 35%, transparent)`}
                                    style:background={`color-mix(in srgb, ${section.colors[entry.kind]} 10%, transparent)`}
                                    style:color={section.colors[entry.kind]}>
                                    {entry.kind}
                                </span>
                                <div class="min-w-0">
                                    <strong
                                        class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap ${entry.synth.name.trim() ? '' : 'text-foreground-muted italic'}`}>
                                        {section.displayName(entry)}
                                    </strong>
                                    <small
                                        class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap">
                                        {entry.kind === SynthKind.Dummy
                                            ? section.details(entry)
                                            : entry.synth.file_path}
                                    </small>
                                </div>
                            </button>
                            <a
                                class="border-border text-foreground-muted hover:bg-surface-hover hover:text-accent flex cursor-pointer items-center gap-2 border-0 border-l bg-transparent px-3 text-xs transition-colors"
                                href={`/controllers?instrument=${entry.id}`}
                                title="Manage controllers"
                                aria-label={`Manage controllers for ${section.displayName(entry)}`}>
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

        {#if section.selected}
            <aside
                class={`border-border-strong bg-surface-overlay min-[56.25rem]:bg-surface-card fixed right-0 bottom-0 left-0 z-60 flex max-h-[min(90dvh,47.5rem)] min-h-0 flex-col overflow-hidden rounded-t-[1.125rem] border border-b-0 shadow-[0_-1.25rem_3.75rem_var(--overlay-shadow)] transition-transform min-[56.25rem]:static min-[56.25rem]:z-1 min-[56.25rem]:h-full min-[56.25rem]:max-h-full min-[56.25rem]:rounded-xl min-[56.25rem]:border-b min-[56.25rem]:shadow-none ${section.inspectorOpen ? 'translate-y-0' : 'translate-y-[110%] min-[56.25rem]:translate-y-0'}`}>
                <header
                    class="border-border flex min-h-16 shrink-0 items-center justify-between border-b px-4">
                    <div>
                        <small
                            class="text-foreground-muted mb-0.5 block text-[0.6875rem] uppercase">
                            Instrument #{section.selected.id + 1} · {section.instrumentType(
                                section.selected.kind
                            )}
                        </small>
                        <h2
                            class={`m-0 text-sm font-semibold ${section.selected.synth.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                            {section.displayName(section.selected)}
                        </h2>
                    </div>
                    <Button
                        variant="ghost"
                        size="md"
                        square
                        class="bg-surface-button min-[56.25rem]:hidden"
                        type="button"
                        aria-label="Close editor"
                        onclick={() => (section.inspectorOpen = false)}>
                        <Icon icon="lucide:x" width="1.1875rem" />
                    </Button>
                </header>
                <div class="min-h-0 flex-1 overflow-y-auto">
                    {#if section.selected.kind === SynthKind.Fluid}<section
                            class="border-border grid gap-2 border-b p-4">
                            <div class="flex min-h-4 items-center justify-between gap-2">
                                <label
                                    for="synth-program"
                                    class="text-foreground-strong text-xs font-semibold uppercase">
                                    SoundFont program
                                </label>
                                {#if section.programBusy}<Icon
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
                                    disabled={section.busy || section.programBusy}
                                    onclick={() => void section.stepProgram(-1)}>
                                    <Icon icon="lucide:arrow-left" width="1rem" />
                                </Button>
                                <SelectDropdown
                                    id="synth-program"
                                    class="flex-1"
                                    bind:value={section.selectedProgram}
                                    options={section.programSuggestions}
                                    placeholder="Select a program"
                                    disabled={section.busy || section.programBusy}
                                    searchable
                                    searchPlaceholder="Search programs"
                                    noMatchesText="No matching programs"
                                    onchange={() => void section.submitProgram()}
                                    ariaLabel="SoundFont program" />
                                <Button
                                    variant="secondary"
                                    size="lg"
                                    square
                                    class="text-foreground-muted enabled:hover:border-accent/40 enabled:hover:bg-accent/10 enabled:hover:text-accent"
                                    type="button"
                                    aria-label="Next SoundFont program"
                                    title="Next program"
                                    disabled={section.busy || section.programBusy}
                                    onclick={() => void section.stepProgram(1)}>
                                    <Icon icon="lucide:arrow-right" width="1rem" />
                                </Button>
                            </div>
                            {#if !section.hasSelectedProgram(section.selected)}
                                <small
                                    class="text-foreground-muted flex items-center gap-1.5 text-xs">
                                    <Icon icon="lucide:triangle-alert" width="0.875rem" />
                                    Using fallback because the requested program is unavailable
                                </small>
                            {/if}
                        </section>
                    {/if}
                    {#key section.selected.id}
                        <div class="border-border border-b p-4">
                            <NumericControl
                                compact
                                label="Gain"
                                value={section.gainDb(section.selected.gain)}
                                unit="dB"
                                smallStep={0.5}
                                largeStep={5}
                                resetValue={0}
                                min={-100}
                                max={100}
                                precision={1}
                                onchange={async (value) => {
                                    await section.setGain(section.selected.id, value);
                                    return value;
                                }} />
                        </div>
                        <div class="border-border border-b p-4">
                            <EffectsEditor
                                title="Instrument effects"
                                effects={section.selected.effects}
                                onsave={(effects) =>
                                    section.saveSynthEffects(section.selected.id, effects)} />
                        </div>
                    {/key}
                    <form
                        class="border-border grid gap-2 border-b p-4"
                        onsubmit={(event) => {
                            event.preventDefault();
                            void section.submitName();
                        }}>
                        <label
                            for="synth-name"
                            class="text-foreground-strong text-xs font-semibold uppercase">
                            Name
                        </label>
                        <AutoSaveTextInput
                            id="synth-name"
                            bind:value={section.editName}
                            pending={section.nameChanged}
                            saving={section.nameSaving}
                            showIndicator={false}
                            required
                            oncommit={() => section.submitName()} />
                    </form>
                    <form
                        class="border-border grid gap-3 border-b p-4"
                        onsubmit={(event) => {
                            event.preventDefault();
                            void section.submitReplacement();
                        }}>
                        <span class="text-foreground-strong text-xs font-semibold uppercase">
                            Replace instrument
                        </span>
                        <label class="grid gap-1.5 text-sm">
                            <span class="text-foreground-muted">Type</span>
                            <SelectDropdown
                                bind:value={section.replacementKind}
                                options={[
                                    { value: SynthKind.Dummy, label: 'Dummy' },
                                    { value: SynthKind.Fluid, label: 'FluidSynth' },
                                    { value: SynthKind.Sfizz, label: 'Sfizz' },
                                    { value: SynthKind.Clap, label: 'CLAP' }
                                ]}
                                onchange={() => {
                                    section.replacementPath = '';
                                    section.replacementPluginId = '';
                                    section.replacementMaxVoices = section.suggestedMaxVoices(
                                        section.replacementKind
                                    );
                                    section.replacementPreloadSize = section.suggestedPreloadSize();
                                }}
                                ariaLabel="Replacement instrument type" />
                        </label>
                        {#if section.replacementKind === SynthKind.Clap}
                            <label class="grid min-w-0 gap-1.5 text-sm">
                                <span class="text-foreground-muted">Plugin</span>
                                <ClapPluginInput
                                    id="replacement-clap-plugin"
                                    plugins={section.clapPlugins()}
                                    compact
                                    required
                                    onreload={section.refreshAssets}
                                    bind:filePath={section.replacementPath}
                                    bind:pluginId={section.replacementPluginId} />
                            </label>
                        {:else if section.replacementKind !== SynthKind.Dummy}
                            <label class="grid gap-1.5 text-sm" for="synth-asset">
                                <span class="text-foreground-muted">
                                    {section.replacementKind === SynthKind.Fluid
                                        ? 'SoundFont file'
                                        : 'SFZ file'}
                                </span>
                                <AssetPathInput
                                    id="synth-asset"
                                    assets={section.assetsForSynth(section.replacementKind)}
                                    placeholder={section.assetPlaceholder(section.replacementKind)}
                                    compact
                                    onreload={section.refreshAssets}
                                    bind:value={section.replacementPath}
                                    required />
                            </label>
                        {/if}
                        {#if section.replacementKind === SynthKind.Fluid || section.replacementKind === SynthKind.Sfizz}<label
                                class="grid gap-1.5 text-sm">
                                <span class="text-foreground-muted">Max voices</span>
                                <input
                                    class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                                    type="number"
                                    min="1"
                                    max={section.replacementKind === SynthKind.Sfizz ? 256 : 65535}
                                    step="1"
                                    required
                                    bind:value={section.replacementMaxVoices} />
                            </label>{/if}
                        {#if section.replacementKind === SynthKind.Sfizz}
                            <label class="grid gap-1.5 text-sm">
                                <span class="text-foreground-muted">Preload frames per sample</span>
                                <input
                                    class="border-border-strong bg-surface-card text-foreground h-10 w-full rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                                    type="number"
                                    min="1"
                                    max="4294967295"
                                    step="1"
                                    required
                                    bind:value={section.replacementPreloadSize} />
                            </label>
                        {/if}
                        <Button
                            variant="accent"
                            size="sm"
                            class="justify-self-end"
                            type="submit"
                            busy={section.busyAction === 'Replace instrument'}
                            disabled={section.busy}>
                            {#if section.busyAction === 'Replace instrument'}<Icon
                                    icon="lucide:loader-circle"
                                    width="0.875rem"
                                    class="animate-spin" />{/if}
                            {section.busyAction === 'Replace instrument' ? 'Replacing…' : 'Replace'}
                        </Button>
                    </form>
                    {#if section.selected.kind === SynthKind.Clap}
                        <section class="border-border border-b p-4">
                            <div
                                class="text-foreground-strong mb-3 text-xs font-semibold uppercase">
                                Plugin
                            </div>
                            <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-2 text-sm">
                                <dt class="text-foreground-muted">Name</dt>
                                <dd
                                    class="m-0 min-w-0 truncate"
                                    title={section.selected.synth.plugin_name ||
                                        section.selected.synth.plugin_id}>
                                    {section.selected.synth.plugin_name ||
                                        section.selected.synth.plugin_id}
                                </dd>
                                <dt class="text-foreground-muted">Version</dt>
                                <dd class="m-0">
                                    {section.selected.synth.plugin_version || 'Not reported'}
                                </dd>
                                <dt class="text-foreground-muted">File</dt>
                                <dd
                                    class="m-0 min-w-0 truncate font-mono text-xs"
                                    title={section.selected.synth.file_path}>
                                    {section.selected.synth.file_path}
                                </dd>
                                <dt class="text-foreground-muted">Plugin ID</dt>
                                <dd
                                    class="m-0 min-w-0 truncate font-mono text-xs"
                                    title={section.selected.synth.plugin_id}>
                                    {section.selected.synth.plugin_id}
                                </dd>
                            </dl>
                        </section>
                    {/if}
                    {#if section.selected.kind === SynthKind.Clap}
                        <details class="border-border group border-b" open>
                            <summary
                                class="hover:bg-surface-raised flex min-h-12 cursor-pointer list-none items-center justify-between gap-3 px-4 py-3 transition-colors [&::-webkit-details-marker]:hidden">
                                <span
                                    class="text-foreground-strong text-xs font-semibold uppercase">
                                    Plugin parameters
                                </span>
                                <span class="flex items-center gap-2">
                                    <small class="text-foreground-muted text-xs tabular-nums">
                                        {section.visibleClapParameters.length}
                                    </small>
                                    <Icon
                                        class="text-foreground-muted transition-transform group-open:rotate-180"
                                        icon="lucide:chevron-down"
                                        width="1rem" />
                                </span>
                            </summary>
                            <div class="border-border border-t px-4 py-2">
                                {#each section.visibleClapParameters as parameter, index (parameter.id)}
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
                                                title={section.parameterValue(parameter)}>
                                                <span class="block max-w-36 truncate">
                                                    {section.parameterValue(parameter)}
                                                </span>
                                            </output>
                                            {#if (parameter.flags & section.readonlyParameterFlag) === 0}
                                                <Button
                                                    variant="ghost"
                                                    size="xs"
                                                    square
                                                    busy={section.clapParameterBusy.has(
                                                        parameter.id
                                                    )}
                                                    disabled={section.clapParameterBusy.has(
                                                        parameter.id
                                                    ) ||
                                                        parameter.value === parameter.default_value}
                                                    aria-label={`Reset ${parameter.name || `parameter ${parameter.id}`}`}
                                                    title="Reset to default"
                                                    onclick={() =>
                                                        void section.submitClapParameter(
                                                            parameter.id,
                                                            parameter.default_value
                                                        )}>
                                                    <Icon
                                                        icon="lucide:rotate-ccw"
                                                        width="0.875rem" />
                                                </Button>
                                            {/if}
                                        </div>
                                        {#if (parameter.flags & section.readonlyParameterFlag) === 0}
                                            <input
                                                class="accent-accent col-span-2 h-7 w-full cursor-pointer disabled:cursor-wait disabled:opacity-60"
                                                type="range"
                                                min={parameter.min_value}
                                                max={parameter.max_value}
                                                step={(parameter.flags & 1) !== 0 ? 1 : 'any'}
                                                value={parameter.value ?? parameter.default_value}
                                                disabled={section.clapParameterBusy.has(
                                                    parameter.id
                                                )}
                                                aria-label={parameter.name ||
                                                    `Parameter ${parameter.id}`}
                                                onchange={(event) =>
                                                    void section.submitClapParameter(
                                                        parameter.id,
                                                        event.currentTarget.valueAsNumber
                                                    )} />
                                        {/if}
                                        <small
                                            class="text-foreground-muted col-span-2 font-mono text-[0.6875rem] tabular-nums">
                                            {section.formatParameterNumber(parameter.min_value)} –
                                            {section.formatParameterNumber(parameter.max_value)} · default
                                            {section.formatParameterNumber(parameter.default_value)}
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
                            busy={section.removing}
                            disabled={section.busy}
                            onconfirm={() => section.submitRemove()} />
                    </section>
                </div>
            </aside>
        {/if}
    </div>
</div>

{#if section.inspectorOpen}<button
        class="bg-scrim/60 fixed inset-0 z-50 cursor-default border-0 backdrop-blur-[0.125rem] min-[56.25rem]:hidden"
        type="button"
        aria-label="Close editor"
        onclick={() => (section.inspectorOpen = false)}>
    </button>{/if}
