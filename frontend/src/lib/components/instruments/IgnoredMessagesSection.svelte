<script lang="ts">
    import DotToggle from '$lib/components/DotToggle.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border grid gap-5 border-b p-4">
    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">
        Ignored MIDI messages
    </h3>
    <div class="grid min-w-0 grid-cols-2 gap-1.5">
        <DotToggle
            label="Pitch bend"
            bind:checked={section.ignoredSection.pitchBend}
            onchange={() => void section.ignoredSection.save()} />
        <DotToggle
            label="Program change"
            bind:checked={section.ignoredSection.programChange}
            onchange={() => void section.ignoredSection.save()} />
        <DotToggle
            label="Channel aftertouch"
            bind:checked={section.ignoredSection.channelAftertouch}
            onchange={() => void section.ignoredSection.save()} />
        <DotToggle
            label="Poly aftertouch"
            bind:checked={section.ignoredSection.polyphonicAftertouch}
            onchange={() => void section.ignoredSection.save()} />
    </div>
</section>
