<script module lang="ts">
    export type NavigationTitle = {
        type: 'title';
        label: string;
    };

    export type NavigationLink = {
        type: 'item';
        label: string;
        href: string;
        icon: string;
    };

    export type NavigationEntry = NavigationTitle | NavigationLink;
</script>

<script lang="ts">
    import EngineStatusIndicator from '$lib/components/EngineStatusIndicator.svelte';
    import { audioHostOutputs, audioOutputStream } from '$lib/state/audio.svelte';
    import { gatewayOnline } from '$lib/state/gateway.svelte';

    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';

    let {
        items,
        activeHref,
        open,
        onClose
    }: {
        items: NavigationEntry[];
        activeHref: string;
        open: boolean;
        onClose: () => void;
    } = $props();
    let outputStream = $derived(audioOutputStream());
    let hostOutputs = $derived(audioHostOutputs());
    let navigationElement: HTMLElement | undefined = $state();
    let canScrollUp = $state(false);
    let canScrollDown = $state(false);
    let activeOutput = $derived(
        hostOutputs
            .find((entry) => entry.host.id === outputStream?.host_id)
            ?.outputs.find((output) => output.id === outputStream?.output_id)
    );

    function formatSampleRate(sampleRate: number) {
        return `${(sampleRate / 1_000).toLocaleString(undefined, { maximumFractionDigits: 3 })} kHz`;
    }

    function updateScrollHints() {
        if (!navigationElement) {
            return;
        }

        canScrollUp = navigationElement.scrollTop > 1;
        canScrollDown =
            navigationElement.scrollTop + navigationElement.clientHeight <
            navigationElement.scrollHeight - 1;
    }

    onMount(() => {
        if (!navigationElement) {
            return;
        }

        const observer = new ResizeObserver(updateScrollHints);
        observer.observe(navigationElement);
        updateScrollHints();

        return () => observer.disconnect();
    });
</script>

{#if open}
    <button
        class="bg-scrim/60 fixed inset-0 z-40 cursor-pointer border-0 backdrop-blur-[0.125rem] min-[56.25rem]:hidden"
        type="button"
        aria-label="Close navigation"
        onclick={onClose}>
    </button>
{/if}

<aside
    class={`border-border bg-surface-1 fixed inset-y-0 left-0 z-50 flex h-dvh min-h-0 w-[min(15rem,calc(100vw-3rem))] flex-col border-r px-3.25 pt-6 pb-[max(0.9375rem,env(safe-area-inset-bottom))] shadow-2xl transition-[transform,visibility] duration-200 min-[56.25rem]:visible min-[56.25rem]:static min-[56.25rem]:z-auto min-[56.25rem]:w-auto min-[56.25rem]:translate-x-0 min-[56.25rem]:pb-3.75 min-[56.25rem]:shadow-none ${open ? 'visible translate-x-0' : 'invisible -translate-x-full'}`}>
    <div class="flex items-center gap-3 px-2 pb-8">
        <div
            class="border-accent/40 bg-accent/9 flex size-8.5 items-center justify-center gap-0.5 rounded-[0.5625rem] border"
            role="img"
            aria-label="AAW">
            <i class="bg-accent h-2 w-0.5 rounded-full"></i>
            <i class="bg-accent h-4.5 w-0.5 rounded-full"></i>
            <i class="bg-accent h-3 w-0.5 rounded-full"></i>
            <i class="bg-accent h-5 w-0.5 rounded-full"></i>
        </div>
        <div class="flex flex-col">
            <strong class="leading-none">AAW</strong>
            <small class="text-foreground-muted mt-1 block text-xs leading-tight">
                Arri's Audio Workstation
            </small>
        </div>
    </div>

    <div class="relative min-h-0 flex-1 overflow-hidden">
        <nav
            bind:this={navigationElement}
            class="grid h-full min-h-0 [scrollbar-width:none] content-start gap-1 overflow-y-auto [&::-webkit-scrollbar]:hidden"
            aria-label="Primary navigation"
            onscroll={updateScrollHints}>
            {#each items as item}
                {#if item.type === 'title'}
                    <span
                        class="text-foreground-inactive mx-3 mt-4 mb-1 block text-xs font-bold tracking-[.16em] uppercase first:mt-0">
                        {item.label}
                    </span>
                {:else}
                    <a
                        class={`hover:bg-surface-hover hover:text-foreground relative flex min-h-10 w-full cursor-pointer items-center gap-3 rounded-lg border-0 px-3 py-2.5 text-sm ${activeHref === item.href ? 'bg-accent/10 text-accent' : 'text-foreground-muted bg-transparent'}`}
                        href={item.href}
                        aria-label={item.label}
                        onclick={onClose}>
                        <Icon icon={item.icon} width="1.125rem" />
                        <span>{item.label}</span>
                    </a>
                {/if}
            {/each}
        </nav>

        {#if canScrollUp}
            <div
                class="from-surface-1 pointer-events-none absolute inset-x-0 top-0 flex h-8 items-start justify-center bg-linear-to-b to-transparent pt-0.5"
                aria-hidden="true">
                <Icon class="text-foreground-inactive" icon="lucide:chevron-up" width="1rem" />
            </div>
        {/if}
        {#if canScrollDown}
            <div
                class="from-surface-1 pointer-events-none absolute inset-x-0 bottom-0 flex h-10 items-end justify-center bg-linear-to-t to-transparent pb-0.5"
                aria-hidden="true">
                <Icon class="text-foreground-inactive" icon="lucide:chevron-down" width="1rem" />
            </div>
        {/if}
    </div>

    <div class="mt-auto">
        <div
            class="border-border mb-1.5 flex items-center gap-2.5 border-b px-2.5 py-4"
            title={`Engine ${gatewayOnline() ? 'online' : 'offline'}`}>
            <EngineStatusIndicator />
            <div class="min-w-0 flex-1">
                <strong class="block text-xs">
                    Engine {gatewayOnline() ? 'online' : 'offline'}
                </strong>
                {#if gatewayOnline()}
                    <small class="text-foreground-muted mt-1 block text-xs">
                        {#if outputStream}
                            <span
                                class="block truncate"
                                title={activeOutput?.name ?? outputStream.output_id}>
                                {activeOutput?.name ?? outputStream.output_id}
                            </span>
                            <span class="block">
                                {formatSampleRate(outputStream.sample_rate)} · {outputStream.buffer_size ===
                                0
                                    ? 'buffer unavailable'
                                    : `${outputStream.buffer_size} frames`}
                            </span>
                        {:else}
                            No output stream
                        {/if}
                    </small>
                {/if}
            </div>
        </div>
        <a
            class={`hover:bg-surface-hover hover:text-foreground flex min-h-10 w-full cursor-pointer items-center gap-3 rounded-lg border-0 px-3 py-2.5 text-sm ${activeHref === '/settings' ? 'bg-accent/10 text-accent' : 'text-foreground-muted bg-transparent'}`}
            href="/settings"
            aria-label="Settings"
            onclick={onClose}>
            <Icon icon="lucide:settings" width="1.125rem" />
            <span>Settings</span>
        </a>
    </div>
</aside>
