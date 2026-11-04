<script lang="ts">
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-2 border-b p-4"
    onsubmit={(event) => {
        event.preventDefault();
        void section.saveName();
    }}>
    <label class="text-foreground-strong text-xs font-semibold uppercase" for="instrument-name">
        Name
    </label>
    <AutoSaveTextInput
        id="instrument-name"
        bind:value={section.editName}
        pending={section.nameChanged}
        saving={section.savingSections.has('name')}
        showIndicator={false}
        required
        oncommit={() => section.saveName()} />
</form>
