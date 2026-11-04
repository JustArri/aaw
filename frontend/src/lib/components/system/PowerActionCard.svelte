<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { SystemSection } from '$lib/sections/system.svelte';

    type SystemAction = 'reboot' | 'shutdown';

    interface Props {
        section: SystemSection;
        action: SystemAction;
    }

    let { section, action }: Props = $props();
    const shutdown = $derived(action === 'shutdown');
    const title = $derived(shutdown ? 'Power off' : 'Reboot');
    const icon = $derived(shutdown ? 'lucide:power' : 'lucide:refresh-cw');
    const description = $derived(
        shutdown
            ? 'Shut down the device. Physical access is required to start it again.'
            : 'Restart the device and briefly interrupt audio and network connections.'
    );
    const runningLabel = $derived(shutdown ? 'Powering off…' : 'Rebooting…');
    const confirmLabel = $derived(shutdown ? 'Confirm power off' : 'Confirm reboot');
</script>

<article class="border-border bg-surface-inset flex flex-col rounded-xl border p-4">
    <div class="flex items-start gap-3">
        <span
            class={shutdown
                ? 'border-danger/30 bg-danger/8 text-danger grid size-10 shrink-0 place-items-center rounded-lg border'
                : 'border-border-strong bg-surface-button text-foreground grid size-10 shrink-0 place-items-center rounded-lg border'}>
            <Icon {icon} width="1.25rem" />
        </span>
        <div>
            <h3 class="m-0 text-base font-semibold">{title}</h3>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">{description}</p>
        </div>
    </div>

    <div class="mt-5 flex min-h-10 items-center justify-end gap-2">
        {#if section.confirming === action}
            <Button
                variant="secondary"
                size="md"
                type="button"
                disabled={section.running !== null}
                onclick={() => (section.confirming = null)}>
                Cancel
            </Button>
            <Button
                variant="danger"
                size="md"
                type="button"
                busy={section.running === action}
                disabled={section.running !== null}
                onclick={() => void section.run(action)}>
                <Icon
                    icon={section.running === action ? 'lucide:loader-circle' : icon}
                    width="1rem"
                    class={section.running === action ? 'animate-spin' : ''} />
                {section.running === action ? runningLabel : confirmLabel}
            </Button>
        {:else}
            <Button
                variant="secondary"
                size="md"
                class={shutdown
                    ? 'enabled:hover:border-danger/50 enabled:hover:bg-danger/10 enabled:hover:text-danger'
                    : 'enabled:hover:border-accent/50 enabled:hover:bg-accent/10 enabled:hover:text-accent'}
                type="button"
                disabled={section.running !== null}
                onclick={() => (section.confirming = action)}>
                <Icon {icon} width="1rem" />
                {title}
            </Button>
        {/if}
    </div>
</article>
