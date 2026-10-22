<script lang="ts">
    import Icon from '@iconify/svelte';
    import {
        dismissNotification,
        notifications,
        type NotificationKind
    } from '$lib/push-notifications.svelte';

    const icons: Record<NotificationKind, string> = {
        info: 'lucide:info',
        success: 'lucide:circle-check',
        warning: 'lucide:triangle-alert',
        error: 'lucide:circle-alert'
    };

    function tone(kind: NotificationKind) {
        switch (kind) {
            case 'success':
                return 'border-success/35 text-success';
            case 'warning':
                return 'border-synth-warm/40 text-synth-warm';
            case 'error':
                return 'border-danger/40 text-danger';
            case 'info':
                return 'border-accent/35 text-accent';
        }
    }
</script>

<aside
    class="pointer-events-none fixed right-3 bottom-24 left-3 z-50 flex flex-col-reverse items-stretch gap-2 min-[37.5rem]:left-auto min-[37.5rem]:w-92 min-[56.25rem]:top-20 min-[56.25rem]:right-5 min-[56.25rem]:bottom-auto min-[56.25rem]:flex-col"
    aria-label="Notifications"
    aria-live="polite">
    {#each notifications as notification (notification.id)}
        <article
            class={`bg-surface-overlay text-foreground pointer-events-auto grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3 rounded-xl border p-3 shadow-[0_0.75rem_2rem_var(--overlay-shadow)] ${tone(notification.kind)}`}
            role={notification.kind === 'error' ? 'alert' : 'status'}>
            <span class="mt-0.5 grid size-7 place-items-center rounded-lg bg-current/10">
                <Icon
                    icon={notification.loading ? 'lucide:loader-circle' : icons[notification.kind]}
                    width="1rem"
                    class={notification.loading ? 'animate-spin' : ''} />
            </span>
            <div class="min-w-0">
                {#if notification.title}
                    <strong class="text-foreground block text-sm font-semibold">
                        {notification.title}
                    </strong>
                {/if}
                <p class="text-foreground-muted m-0 text-sm leading-snug">
                    {notification.message}
                </p>
            </div>
            <button
                class="text-foreground-muted hover:bg-surface-hover hover:text-foreground grid size-7 cursor-pointer place-items-center rounded-md border-0 bg-transparent"
                type="button"
                aria-label="Dismiss notification"
                onclick={() => dismissNotification(notification.id)}>
                <Icon icon="lucide:x" width="1rem" />
            </button>
        </article>
    {/each}
</aside>
