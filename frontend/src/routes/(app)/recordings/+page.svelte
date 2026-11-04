<script lang="ts">
    import RecordingsPanel from '$lib/components/recordings/RecordingsPanel.svelte';
    import { RecordingsSection } from '$lib/sections/recordings.svelte';

    const section = new RecordingsSection();
</script>

<RecordingsPanel {section} />
