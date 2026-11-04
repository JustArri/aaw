<script lang="ts">
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-3 border-b px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(12rem,24.25rem)] min-[37.5rem]:items-end min-[56.25rem]:px-5"
    onsubmit={(event) => {
        event.preventDefault();
        void section.rename();
    }}>
    <div>
        <label class="block text-sm font-semibold" for="active-engine-midi-configuration-name">
            Active configuration name
        </label>
        <small class="text-foreground-muted mt-1 block text-sm">
            Rename before section.saving a new configuration.
        </small>
    </div>
    <AutoSaveTextInput
        id="active-engine-midi-configuration-name"
        bind:value={section.name}
        pending={section.nameChanged}
        saving={section.renaming}
        required
        oncommit={() => section.rename()} />
</form>
