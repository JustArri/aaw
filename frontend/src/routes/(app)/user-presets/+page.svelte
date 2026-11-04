<script lang="ts">
    import UserPresetsPanel from '$lib/components/user-presets/UserPresetsPanel.svelte';
    import { UserPresetsSection } from '$lib/sections/user-presets.svelte';

    const section = new UserPresetsSection();
</script>

<UserPresetsPanel {section} />
