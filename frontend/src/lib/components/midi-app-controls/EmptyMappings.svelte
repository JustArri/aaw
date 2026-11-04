<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';

    interface Props {
        title: string;
        description: string;
        saving: boolean;
        onadd: () => void;
    }

    let { title, description, saving, onadd }: Props = $props();
</script>

<div class="grid min-h-52 place-items-center gap-3 px-6 py-8 text-center">
    <div>
        <Icon
            icon="lucide:radio-tower"
            width="2rem"
            class="text-foreground-inactive mx-auto mb-3" />
        <p class="text-foreground m-0 text-sm font-semibold">{title}</p>
        <p class="text-foreground-muted mt-1 mb-0 text-sm">{description}</p>
    </div>
    <Button variant="primary" size="sm" disabled={saving} onclick={onadd}>
        <Icon icon="lucide:plus" width="0.875rem" /> Add mapping
    </Button>
</div>
