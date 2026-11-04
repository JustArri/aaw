<script lang="ts">
    import PowerControls from '$lib/components/system/PowerControls.svelte';
    import SynthDefaults from '$lib/components/system/SynthDefaults.svelte';
    import { SystemSection } from '$lib/sections/system.svelte';

    const section = new SystemSection();
</script>

<div class="grid gap-3">
    <SynthDefaults {section} />
    <PowerControls {section} />
</div>
