<script lang="ts">
    import type { KeyboardSection } from '$lib/sections/keyboard.svelte';

    interface Props {
        section: KeyboardSection;
    }

    let { section }: Props = $props();
</script>

<div
    class="bg-surface-inset-deep w-full touch-pan-x overflow-x-auto overscroll-x-contain p-3.75 min-[56.25rem]:p-5"
    bind:this={section.keyboardViewport}>
    <div
        class="relative mx-auto h-40 select-none min-[56.25rem]:h-56"
        style:width={`${section.keyboard.whiteCount * section.keyWidth}rem`}>
        <div class="absolute inset-0 flex">
            {#each section.whiteKeys as key (key.note)}
                {@const active = section.notesByKey.has(key.note)}
                {@const keyNotes = section.notesByKey.get(key.note) ?? []}
                <div
                    class={`relative h-full shrink-0 border-r border-b first:border-l ${active ? 'border-keyboard-black/35 shadow-[inset_0_-0.3rem_0_rgb(0_0_0/0.22)]' : 'border-keyboard-black/35 bg-keyboard-white'}`}
                    style:width={`${section.keyWidth}rem`}
                    style:background={active ? section.noteColor(keyNotes) : undefined}
                    data-midi-key={key.note}
                    role="img"
                    aria-label={`${section.noteName(key.note)}${active ? ', playing' : ''}`}
                    title={`${section.noteName(key.note)} · MIDI ${key.note}`}>
                    {#if section.keyWidth >= 1 && (active || key.note % 12 === 0)}
                        <span
                            class="text-keyboard-black/65 absolute right-0 bottom-2 left-0 text-center font-mono text-xs font-semibold">
                            {section.noteName(key.note)}
                        </span>
                    {/if}
                </div>
            {/each}
        </div>

        {#each section.blackKeys as key (key.note)}
            {@const active = section.notesByKey.has(key.note)}
            {@const keyNotes = section.notesByKey.get(key.note) ?? []}
            <div
                class={`absolute top-0 z-10 h-24 rounded-b-md border border-t-0 shadow-md min-[56.25rem]:h-36 ${active ? 'border-keyboard-black/50 shadow-[0_0_0.75rem_var(--accent-glow-soft)]' : 'border-keyboard-black bg-keyboard-black'}`}
                style:width={`${section.keyWidth * section.blackKeyWidthRatio}rem`}
                style:background={active ? section.noteColor(keyNotes) : undefined}
                style:left={`${(key.left / section.whiteKeyWidthRem) * section.keyWidth}rem`}
                data-midi-key={key.note}
                role="img"
                aria-label={`${section.noteName(key.note)}${active ? ', playing' : ''}`}
                title={`${section.noteName(key.note)} · MIDI ${key.note}`}>
            </div>
        {/each}
    </div>
</div>
