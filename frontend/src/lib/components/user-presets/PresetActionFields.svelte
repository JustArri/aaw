<script lang="ts">
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { UserPresetAction } from '$lib/generated/models';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';

    interface Props {
        section: UserPresetsSection;
        action: UserPresetAction;
        presetIndex: number;
        actionIndex: number;
    }

    let { section, action, presetIndex, actionIndex }: Props = $props();
</script>

{#if action === 'DisableAllInstrumentControllers'}
    <p class="text-foreground-muted m-0 px-6 text-xs">Disables every controller in the project.</p>
{:else if 'SetInstrumentControllerEnabled' in action}
    <div class="grid gap-2 min-[37.5rem]:grid-cols-[minmax(0,1fr)_9rem]">
        <SelectDropdown
            value={section.controllerKey(
                action.SetInstrumentControllerEnabled.controller.instrument_id,
                action.SetInstrumentControllerEnabled.controller.controller_id
            )}
            options={section.controllerOptions(action)}
            searchable
            compact
            searchPlaceholder="Search controllers"
            noMatchesText="No matching controllers"
            ariaLabel="Instrument controller"
            onchange={(value) => section.setActionController(presetIndex, actionIndex, value)} />
        <SelectDropdown
            value={String(action.SetInstrumentControllerEnabled.flag)}
            options={section.enabledOptions}
            compact
            ariaLabel="Controller state"
            onchange={(value) => section.setActionEnabled(presetIndex, actionIndex, value)} />
    </div>
{:else if 'SetSoundFontProgram' in action}
    <div class="grid gap-2 min-[37.5rem]:grid-cols-2">
        <SelectDropdown
            value={String(action.SetSoundFontProgram.instrument)}
            options={section.instrumentOptions(action)}
            searchable
            compact
            searchPlaceholder="Search SoundFont instruments"
            noMatchesText="No SoundFont instruments"
            ariaLabel="SoundFont instrument"
            onchange={(value) => section.setActionInstrument(presetIndex, actionIndex, value)} />
        <SelectDropdown
            value={`${action.SetSoundFontProgram.program.bank}:${action.SetSoundFontProgram.program.preset_id}`}
            options={section.programOptions(action)}
            searchable
            compact
            searchPlaceholder="Search programs"
            noMatchesText="No programs available"
            ariaLabel="SoundFont program"
            onchange={(value) => section.setActionProgram(presetIndex, actionIndex, value)} />
    </div>
{:else}
    {@const parameter = section.clapParameter(action)}
    <div class="grid gap-2 min-[37.5rem]:grid-cols-2">
        <SelectDropdown
            value={String(action.SetClapParameter.instrument)}
            options={section.clapInstrumentOptions(action)}
            searchable
            compact
            searchPlaceholder="Search CLAP instruments"
            noMatchesText="No CLAP instruments"
            ariaLabel="CLAP instrument"
            onchange={(value) =>
                section.setActionClapInstrument(presetIndex, actionIndex, value)} />
        <SelectDropdown
            value={String(action.SetClapParameter.parameter_id)}
            options={section.clapParameterOptions(action)}
            searchable
            compact
            searchPlaceholder="Search parameters"
            noMatchesText="No writable parameters"
            ariaLabel="CLAP parameter"
            onchange={(value) => section.setActionClapParameter(presetIndex, actionIndex, value)} />
    </div>
    <NumericControl
        compact
        label={parameter?.name || `Parameter ${action.SetClapParameter.parameter_id}`}
        value={action.SetClapParameter.value}
        unit="value"
        smallStep={section.parameterSmallStep(parameter)}
        largeStep={section.parameterSmallStep(parameter) * 10}
        resetValue={parameter?.default_value ?? action.SetClapParameter.value}
        min={parameter?.min_value ?? Math.min(0, action.SetClapParameter.value)}
        max={parameter?.max_value ?? Math.max(1, action.SetClapParameter.value)}
        precision={section.parameterPrecision(parameter)}
        onchange={(value) => section.setActionClapValue(presetIndex, actionIndex, value)} />
{/if}
