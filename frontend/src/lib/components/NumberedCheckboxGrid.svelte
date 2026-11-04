<script lang="ts">
    import Button from '$lib/components/Button.svelte';

    interface Props {
        values: number[];
        selected: number[];
        itemLabel: string;
        ontoggle: (value: number, checked: boolean) => void;
        onsetall: (values: number[]) => void;
    }

    let { values, selected, itemLabel, ontoggle, onsetall }: Props = $props();
</script>

<div class="grid gap-1.5">
    <div class="grid grid-cols-8 gap-1">
        {#each values as value}
            <label
                class={`has-focus-visible:outline-accent relative grid min-h-9 cursor-pointer place-items-center rounded-md border font-mono text-xs font-semibold transition-colors has-focus-visible:outline-2 has-focus-visible:outline-offset-2 ${selected.includes(value) ? 'border-accent/60 bg-accent/12 text-accent' : 'border-border-strong bg-surface-inset text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}>
                <input
                    class="absolute inset-0 size-full cursor-pointer appearance-none opacity-0"
                    type="checkbox"
                    aria-label={`${itemLabel} ${value + 1}`}
                    checked={selected.includes(value)}
                    onchange={(event) => ontoggle(value, event.currentTarget.checked)} />
                <span>{value + 1}</span>
            </label>
        {/each}
    </div>
    <div class="flex justify-end gap-1.5">
        <Button
            variant="secondary"
            size="xs"
            class="text-foreground-muted"
            type="button"
            disabled={selected.length === values.length}
            onclick={() => onsetall(values)}>
            All
        </Button>
        <Button
            variant="secondary"
            size="xs"
            class="text-foreground-muted"
            type="button"
            disabled={selected.length === 0}
            onclick={() => onsetall([])}>
            None
        </Button>
    </div>
</div>
