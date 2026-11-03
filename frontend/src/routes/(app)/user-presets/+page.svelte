<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import { flip } from 'svelte/animate';
    import Button from '$lib/components/Button.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { ParameterInfo, UserPresetAction } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { controllerEntries, fetchSynths, synths, SynthKind } from '$lib/state/synths.svelte';
    import {
        activateUserPreset,
        cloneUserPresets,
        fetchUserPresets,
        initializeUserPresetsDraft,
        resetUserPresetsDraft,
        saveUserPresets,
        userPresets,
        userPresetsDraft as draft,
        type EditableUserPreset
    } from '$lib/state/user-presets.svelte';

    let loading = $state(true);
    let busy = $state(false);
    let activating = $state<number | null>(null);
    let draggedPreset = $state<EditableUserPreset | null>(null);
    let dragX = $state(0);
    let dragY = $state(0);
    let dropInsertionIndex = $state<number | null>(null);

    let instruments = $derived(synths().sort((left, right) => left.id - right.id));
    let controllers = $derived(
        controllerEntries().sort(
            (left, right) =>
                left.instrument_id - right.instrument_id || left.controller_id - right.controller_id
        )
    );
    let fluidInstruments = $derived(
        instruments.filter((instrument) => instrument.kind === SynthKind.Fluid)
    );
    let clapInstruments = $derived(
        instruments.filter((instrument) => instrument.kind === SynthKind.Clap)
    );
    let changed = $derived(JSON.stringify(draft) !== JSON.stringify(userPresets()));

    const actionTypeOptions = [
        {
            value: 'disable-all',
            label: 'Disable all controllers',
            description: 'Disable every controller before applying later actions.'
        },
        {
            value: 'set-controller',
            label: 'Set controller state',
            description: 'Enable or disable one instrument controller.'
        },
        {
            value: 'set-program',
            label: 'Set SoundFont program',
            description: 'Select a bank and program on a SoundFont instrument.'
        },
        {
            value: 'set-clap-parameter',
            label: 'Set CLAP parameter',
            description: 'Set one writable parameter on a CLAP plugin instrument.'
        }
    ];
    const enabledOptions = [
        { value: 'true', label: 'Enabled' },
        { value: 'false', label: 'Disabled' }
    ];

    onMount(async () => {
        try {
            const [presets] = await Promise.all([fetchUserPresets(), fetchSynths()]);
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
            actions: ['DisableAllInstrumentControllers']
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

    function startPresetDrag(event: PointerEvent, preset: EditableUserPreset) {
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

    function actionType(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers') {
            return 'disable-all';
        }
        if ('SetInstrumentControllerEnabled' in action) {
            return 'set-controller';
        }
        return 'SetSoundFontProgram' in action ? 'set-program' : 'set-clap-parameter';
    }

    function controllerKey(instrumentId: number, controllerId: number) {
        return String(instrumentId * 65_536 + controllerId);
    }

    function controllerName(entry: (typeof controllers)[number]) {
        const instrument = instruments.find((candidate) => candidate.id === entry.instrument_id);
        const instrumentName =
            instrument?.synth.name.trim() || `Instrument #${entry.instrument_id + 1}`;
        const type = entry.controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine';
        const name = entry.controller.data.name.trim();
        return `${instrumentName} · ${name ? `${type}: ${name}` : type}`;
    }

    function controllerOptions(action: UserPresetAction) {
        const options = controllers.map((entry) => ({
            value: controllerKey(entry.instrument_id, entry.controller_id),
            label: controllerName(entry)
        }));
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        ) {
            return options;
        }
        const address = action.SetInstrumentControllerEnabled.controller;
        const value = controllerKey(address.instrument_id, address.controller_id);
        return options.some((option) => option.value === value)
            ? options
            : [
                  {
                      value,
                      label: `Missing controller ${address.instrument_id}:${address.controller_id}`
                  },
                  ...options
              ];
    }

    function instrumentOptions(action: UserPresetAction) {
        const options = fluidInstruments.map((instrument) => ({
            value: String(instrument.id),
            label: instrument.synth.name.trim() || `SoundFont instrument #${instrument.id + 1}`
        }));
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action)) {
            return options;
        }
        const value = String(action.SetSoundFontProgram.instrument);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing instrument #${Number(value) + 1}` }, ...options];
    }

    function programOptions(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action)) {
            return [];
        }
        const data = action.SetSoundFontProgram;
        const instrument = fluidInstruments.find((candidate) => candidate.id === data.instrument);
        const options =
            instrument?.synth.info.banks.flatMap((bank) =>
                bank.presets.map((program) => ({
                    value: `${bank.number}:${program.number}`,
                    label: program.name,
                    description: `Bank ${bank.number} · Program ${program.number}`
                }))
            ) ?? [];
        const value = `${data.program.bank}:${data.program.preset_id}`;
        return options.some((option) => option.value === value)
            ? options
            : [
                  { value, label: `Bank ${data.program.bank} · Program ${data.program.preset_id}` },
                  ...options
              ];
    }

    function writableClapParameters(instrument: (typeof clapInstruments)[number] | undefined) {
        return (
            instrument?.synth.plugin_parameters.filter(
                (parameter) => (parameter.flags & 4) === 0 && (parameter.flags & 8) === 0
            ) ?? []
        );
    }

    function clapInstrumentOptions(action: UserPresetAction) {
        const options = clapInstruments.map((instrument) => ({
            value: String(instrument.id),
            label: instrument.synth.name.trim() || `CLAP instrument #${instrument.id + 1}`
        }));
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) {
            return options;
        }
        const value = String(action.SetClapParameter.instrument);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing CLAP instrument #${Number(value) + 1}` }, ...options];
    }

    function clapParameter(action: UserPresetAction): ParameterInfo | undefined {
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) {
            return undefined;
        }
        const data = action.SetClapParameter;
        return clapInstruments
            .find((instrument) => instrument.id === data.instrument)
            ?.synth.plugin_parameters.find((parameter) => parameter.id === data.parameter_id);
    }

    function clapParameterOptions(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) {
            return [];
        }
        const data = action.SetClapParameter;
        const instrument = clapInstruments.find((candidate) => candidate.id === data.instrument);
        const options = writableClapParameters(instrument).map((parameter) => ({
            value: String(parameter.id),
            label: parameter.name || `Parameter ${parameter.id}`,
            description: parameter.module || undefined
        }));
        const value = String(data.parameter_id);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing parameter ${value}` }, ...options];
    }

    function parameterPrecision(parameter: ParameterInfo | undefined) {
        return parameter && (parameter.flags & 1) !== 0 ? 0 : 3;
    }

    function parameterSmallStep(parameter: ParameterInfo | undefined) {
        if (!parameter) {
            return 0.01;
        }
        return (parameter.flags & 1) !== 0
            ? 1
            : Math.max((parameter.max_value - parameter.min_value) / 100, 0.001);
    }

    function replaceAction(presetIndex: number, actionIndex: number, action: UserPresetAction) {
        draft.entries[presetIndex].actions[actionIndex] = action;
    }

    function setActionType(presetIndex: number, actionIndex: number, type: string) {
        if (type === 'disable-all') {
            replaceAction(presetIndex, actionIndex, 'DisableAllInstrumentControllers');
            return;
        }
        if (type === 'set-controller') {
            const controller = controllers[0];
            replaceAction(presetIndex, actionIndex, {
                SetInstrumentControllerEnabled: {
                    controller: {
                        instrument_id: controller?.instrument_id ?? 0,
                        controller_id: controller?.controller_id ?? 0
                    },
                    flag: true
                }
            });
            return;
        }
        if (type === 'set-program') {
            const instrument = fluidInstruments[0];
            replaceAction(presetIndex, actionIndex, {
                SetSoundFontProgram: {
                    instrument: instrument?.id ?? 0,
                    program: { ...(instrument?.synth.program ?? { bank: 0, preset_id: 0 }) }
                }
            });
            return;
        }
        const instrument = clapInstruments[0];
        const parameter = writableClapParameters(instrument)[0];
        replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                instrument: instrument?.id ?? 0,
                parameter_id: parameter?.id ?? 0,
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }

    function setActionController(presetIndex: number, actionIndex: number, value: string) {
        const action = draft.entries[presetIndex].actions[actionIndex];
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        ) {
            return;
        }
        replaceAction(presetIndex, actionIndex, {
            SetInstrumentControllerEnabled: {
                ...action.SetInstrumentControllerEnabled,
                controller: {
                    instrument_id: Math.floor(Number(value) / 65_536),
                    controller_id: Number(value) % 65_536
                }
            }
        });
    }

    function setActionEnabled(presetIndex: number, actionIndex: number, value: string) {
        const action = draft.entries[presetIndex].actions[actionIndex];
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        ) {
            return;
        }
        replaceAction(presetIndex, actionIndex, {
            SetInstrumentControllerEnabled: {
                ...action.SetInstrumentControllerEnabled,
                flag: value === 'true'
            }
        });
    }

    function setActionInstrument(presetIndex: number, actionIndex: number, value: string) {
        const instrument = fluidInstruments.find((candidate) => candidate.id === Number(value));
        replaceAction(presetIndex, actionIndex, {
            SetSoundFontProgram: {
                instrument: Number(value),
                program: { ...(instrument?.synth.program ?? { bank: 0, preset_id: 0 }) }
            }
        });
    }

    function setActionProgram(presetIndex: number, actionIndex: number, value: string) {
        const action = draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action)) {
            return;
        }
        const [bank, presetId] = value.split(':').map(Number);
        replaceAction(presetIndex, actionIndex, {
            SetSoundFontProgram: {
                ...action.SetSoundFontProgram,
                program: { bank, preset_id: presetId }
            }
        });
    }

    function setActionClapInstrument(presetIndex: number, actionIndex: number, value: string) {
        const instrument = clapInstruments.find((candidate) => candidate.id === Number(value));
        const parameter = writableClapParameters(instrument)[0];
        replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                instrument: Number(value),
                parameter_id: parameter?.id ?? 0,
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }

    function setActionClapParameter(presetIndex: number, actionIndex: number, value: string) {
        const action = draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) {
            return;
        }
        const instrument = clapInstruments.find(
            (candidate) => candidate.id === action.SetClapParameter.instrument
        );
        const parameter = writableClapParameters(instrument).find(
            (candidate) => candidate.id === Number(value)
        );
        replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                ...action.SetClapParameter,
                parameter_id: Number(value),
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }

    function setActionClapValue(presetIndex: number, actionIndex: number, value: number) {
        const action = draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) {
            return;
        }
        replaceAction(presetIndex, actionIndex, {
            SetClapParameter: { ...action.SetClapParameter, value }
        });
    }

    function addAction(presetIndex: number) {
        draft.entries[presetIndex].actions.push('DisableAllInstrumentControllers');
    }

    function removeAction(presetIndex: number, actionIndex: number) {
        draft.entries[presetIndex].actions.splice(actionIndex, 1);
    }

    function moveAction(presetIndex: number, actionIndex: number, offset: number) {
        const actions = draft.entries[presetIndex].actions;
        const destination = actionIndex + offset;
        if (destination < 0 || destination >= actions.length) {
            return;
        }
        const [action] = actions.splice(actionIndex, 1);
        actions.splice(destination, 0, action);
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
        try {
            draft.entries = draft.entries.map((preset) => ({
                ...preset,
                name: preset.name.trim()
            }));
            await saveUserPresets(cloneUserPresets(draft));
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'User presets'
            });
        } finally {
            busy = false;
        }
    }

    async function activate(index: number) {
        if (activating !== null || changed) {
            return;
        }
        activating = index;
        try {
            await activateUserPreset(index);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: draft.entries[index].name
            });
        } finally {
            activating = null;
        }
    }
</script>

<section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
    <header class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
        <div class="flex items-baseline gap-2">
            <h2 class="m-0 text-sm font-semibold">User presets</h2>
            <span class="text-foreground-muted text-xs">{draft.entries.length} total</span>
        </div>
        <div class="flex items-center gap-1.5">
            <Button
                variant="secondary"
                size="sm"
                square
                class="size-8.5"
                type="button"
                aria-label="Add user preset"
                title="Add user preset"
                disabled={busy}
                onclick={addPreset}>
                <Icon icon="lucide:plus" width="1.125rem" />
            </Button>
            <Button
                variant="ghost"
                size="sm"
                square
                class="size-8.5"
                type="button"
                aria-label="Discard preset changes"
                title="Discard changes"
                disabled={!changed || busy}
                onclick={discardChanges}>
                <Icon icon="lucide:undo-2" width="1rem" />
            </Button>
            <Button
                variant="accent"
                size="sm"
                square
                class="size-8.5"
                type="button"
                aria-label="Save user presets"
                title="Save user presets"
                {busy}
                disabled={!changed || busy}
                onclick={() => void save()}>
                <Icon
                    icon={busy ? 'lucide:loader-circle' : 'lucide:check'}
                    width="1.125rem"
                    class={busy ? 'animate-spin' : ''} />
            </Button>
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
                <Button variant="accent" size="sm" class="mt-3" type="button" onclick={addPreset}>
                    Add first preset
                </Button>
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

                        <div class="mt-4 grid gap-2">
                            <div class="flex items-baseline justify-between gap-2">
                                <span
                                    class="text-foreground-strong text-xs font-semibold uppercase">
                                    Actions
                                </span>
                                <span class="text-foreground-muted text-xs">
                                    Applied from top to bottom
                                </span>
                            </div>
                            {#each preset.actions as action, actionIndex}
                                <div
                                    class="border-border bg-surface-card grid gap-2 rounded-lg border p-2.5">
                                    <div class="flex min-w-0 items-center gap-1.5">
                                        <span
                                            class="text-foreground-muted w-5 shrink-0 text-center font-mono text-xs">
                                            {actionIndex + 1}
                                        </span>
                                        <SelectDropdown
                                            class="min-w-0 flex-1"
                                            value={actionType(action)}
                                            options={actionTypeOptions}
                                            compact
                                            ariaLabel={`Preset ${index + 1} action ${actionIndex + 1} type`}
                                            onchange={(value) =>
                                                setActionType(index, actionIndex, value)} />
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            square
                                            type="button"
                                            aria-label="Move action up"
                                            title="Move action up"
                                            disabled={actionIndex === 0}
                                            onclick={() => moveAction(index, actionIndex, -1)}>
                                            <Icon icon="lucide:arrow-up" width="0.875rem" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            square
                                            type="button"
                                            aria-label="Move action down"
                                            title="Move action down"
                                            disabled={actionIndex === preset.actions.length - 1}
                                            onclick={() => moveAction(index, actionIndex, 1)}>
                                            <Icon icon="lucide:arrow-down" width="0.875rem" />
                                        </Button>
                                        <Button
                                            variant="danger-ghost"
                                            size="sm"
                                            square
                                            type="button"
                                            aria-label="Remove action"
                                            title="Remove action"
                                            onclick={() => removeAction(index, actionIndex)}>
                                            <Icon icon="lucide:x" width="0.875rem" />
                                        </Button>
                                    </div>

                                    {#if action === 'DisableAllInstrumentControllers'}
                                        <p class="text-foreground-muted m-0 px-6 text-xs">
                                            Disables every controller in the project.
                                        </p>
                                    {:else if 'SetInstrumentControllerEnabled' in action}
                                        <div
                                            class="grid gap-2 min-[37.5rem]:grid-cols-[minmax(0,1fr)_9rem]">
                                            <SelectDropdown
                                                value={controllerKey(
                                                    action.SetInstrumentControllerEnabled.controller
                                                        .instrument_id,
                                                    action.SetInstrumentControllerEnabled.controller
                                                        .controller_id
                                                )}
                                                options={controllerOptions(action)}
                                                searchable
                                                compact
                                                searchPlaceholder="Search controllers"
                                                noMatchesText="No matching controllers"
                                                ariaLabel="Instrument controller"
                                                onchange={(value) =>
                                                    setActionController(
                                                        index,
                                                        actionIndex,
                                                        value
                                                    )} />
                                            <SelectDropdown
                                                value={String(
                                                    action.SetInstrumentControllerEnabled.flag
                                                )}
                                                options={enabledOptions}
                                                compact
                                                ariaLabel="Controller state"
                                                onchange={(value) =>
                                                    setActionEnabled(index, actionIndex, value)} />
                                        </div>
                                    {:else if 'SetSoundFontProgram' in action}
                                        <div class="grid gap-2 min-[37.5rem]:grid-cols-2">
                                            <SelectDropdown
                                                value={String(
                                                    action.SetSoundFontProgram.instrument
                                                )}
                                                options={instrumentOptions(action)}
                                                searchable
                                                compact
                                                searchPlaceholder="Search SoundFont instruments"
                                                noMatchesText="No SoundFont instruments"
                                                ariaLabel="SoundFont instrument"
                                                onchange={(value) =>
                                                    setActionInstrument(
                                                        index,
                                                        actionIndex,
                                                        value
                                                    )} />
                                            <SelectDropdown
                                                value={`${action.SetSoundFontProgram.program.bank}:${action.SetSoundFontProgram.program.preset_id}`}
                                                options={programOptions(action)}
                                                searchable
                                                compact
                                                searchPlaceholder="Search programs"
                                                noMatchesText="No programs available"
                                                ariaLabel="SoundFont program"
                                                onchange={(value) =>
                                                    setActionProgram(index, actionIndex, value)} />
                                        </div>
                                    {:else}
                                        {@const parameter = clapParameter(action)}
                                        <div class="grid gap-2 min-[37.5rem]:grid-cols-2">
                                            <SelectDropdown
                                                value={String(action.SetClapParameter.instrument)}
                                                options={clapInstrumentOptions(action)}
                                                searchable
                                                compact
                                                searchPlaceholder="Search CLAP instruments"
                                                noMatchesText="No CLAP instruments"
                                                ariaLabel="CLAP instrument"
                                                onchange={(value) =>
                                                    setActionClapInstrument(
                                                        index,
                                                        actionIndex,
                                                        value
                                                    )} />
                                            <SelectDropdown
                                                value={String(action.SetClapParameter.parameter_id)}
                                                options={clapParameterOptions(action)}
                                                searchable
                                                compact
                                                searchPlaceholder="Search parameters"
                                                noMatchesText="No writable parameters"
                                                ariaLabel="CLAP parameter"
                                                onchange={(value) =>
                                                    setActionClapParameter(
                                                        index,
                                                        actionIndex,
                                                        value
                                                    )} />
                                        </div>
                                        <NumericControl
                                            compact
                                            label={parameter?.name ||
                                                `Parameter ${action.SetClapParameter.parameter_id}`}
                                            value={action.SetClapParameter.value}
                                            unit="value"
                                            smallStep={parameterSmallStep(parameter)}
                                            largeStep={parameterSmallStep(parameter) * 10}
                                            resetValue={parameter?.default_value ??
                                                action.SetClapParameter.value}
                                            min={parameter?.min_value ??
                                                Math.min(0, action.SetClapParameter.value)}
                                            max={parameter?.max_value ??
                                                Math.max(1, action.SetClapParameter.value)}
                                            precision={parameterPrecision(parameter)}
                                            onchange={(value) =>
                                                setActionClapValue(index, actionIndex, value)} />
                                    {/if}
                                </div>
                            {:else}
                                <p
                                    class="text-foreground-muted m-0 rounded-lg border border-dashed border-border p-3 text-center text-xs">
                                    This preset has no actions.
                                </p>
                            {/each}
                            <Button
                                variant="secondary"
                                size="sm"
                                class="justify-self-start"
                                type="button"
                                onclick={() => addAction(index)}>
                                <Icon icon="lucide:plus" width="0.875rem" />
                                Add action
                            </Button>
                        </div>

                        <div class="mt-3 flex justify-end gap-1">
                            <Button
                                variant="ghost"
                                size="sm"
                                square
                                type="button"
                                aria-label="Move preset up"
                                title="Move up"
                                disabled={index === 0}
                                onclick={() => movePreset(index, -1)}>
                                <Icon icon="lucide:arrow-up" width="0.9375rem" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="sm"
                                square
                                type="button"
                                aria-label="Move preset down"
                                title="Move down"
                                disabled={index === draft.entries.length - 1}
                                onclick={() => movePreset(index, 1)}>
                                <Icon icon="lucide:arrow-down" width="0.9375rem" />
                            </Button>
                            <Button
                                variant="danger-ghost"
                                size="sm"
                                square
                                type="button"
                                aria-label={`Remove ${preset.name}`}
                                title="Remove preset"
                                onclick={() => removePreset(index)}>
                                <Icon icon="lucide:trash-2" width="0.9375rem" />
                            </Button>
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
