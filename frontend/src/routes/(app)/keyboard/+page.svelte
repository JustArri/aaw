<script lang="ts">
    import KeyboardActivityBar from '$lib/components/keyboard/KeyboardActivityBar.svelte';
    import KeyboardHeader from '$lib/components/keyboard/KeyboardHeader.svelte';
    import PianoKeyboard from '$lib/components/keyboard/PianoKeyboard.svelte';
    import { KeyboardSection } from '$lib/sections/keyboard.svelte';

    const section = new KeyboardSection();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <KeyboardHeader {section} />
    <KeyboardActivityBar {section} />
    <PianoKeyboard {section} />
</section>
