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

    let { items, activeLabel }: { items: NavigationEntry[]; activeLabel: string } = $props();
    let links = $derived(items.filter((item): item is NavigationLink => item.type === 'item'));
    let outputStream = $derived(audioOutputStream());
    let hostOutputs = $derived(audioHostOutputs());
    let activeOutput = $derived(
        hostOutputs
            .find((entry) => entry.host.id === outputStream?.host_id)
            ?.outputs.find((output) => output.id === outputStream?.output_id)
    );

    function formatSampleRate(sampleRate: number) {
        return `${(sampleRate / 1_000).toLocaleString(undefined, { maximumFractionDigits: 3 })} kHz`;
    }
</script>

<aside
    class="min-[56.25rem]:border-border min-[56.25rem]:bg-surface-1 hidden min-[56.25rem]:sticky min-[56.25rem]:top-0 min-[56.25rem]:flex min-[56.25rem]:h-screen min-[56.25rem]:flex-col min-[56.25rem]:border-r min-[56.25rem]:px-3.25 min-[56.25rem]:pt-6 min-[56.25rem]:pb-3.75">
    <div class="flex items-center gap-3 px-2 pt-0 pb-8">
        <div
            class="border-accent/40 bg-accent/9 [&_i]:bg-accent flex size-8.5 items-center justify-center gap-0.5 rounded-[0.5625rem] border [&_i]:w-0.5 [&_i]:rounded-full"
            aria-hidden="true">
            <i class="h-2"></i>
            <i class="h-4.5"></i>
            <i class="h-3"></i>
            <i class="h-5"></i>
        </div>
        <div class="flex flex-col">
            <strong class="leading-none">AAW</strong>
            <small class="text-foreground-muted mt-1 block text-xs leading-tight">
                Arri's Audio Workstation
            </small>
        </div>
    </div>

    <nav class="grid gap-1" aria-label="Primary navigation">
        {#each items as item}
            {#if item.type === 'title'}
                <span
                    class="text-foreground-inactive mx-3 mt-4 mb-1 block text-xs font-bold tracking-[.16em] uppercase first:mt-0">
                    {item.label}
                </span>
            {:else}
                <a
                    class={`hover:bg-surface-hover hover:text-foreground relative flex w-full cursor-pointer items-center gap-3 rounded-lg border-0 px-3 py-2.5 text-sm ${activeLabel === item.label ? 'bg-accent/10 text-accent' : 'text-foreground-muted bg-transparent'}`}
                    href={item.href}>
                    <Icon icon={item.icon} width="1.125rem" />
                    <span>{item.label}</span>
                </a>
            {/if}
        {/each}
    </nav>

    <div class="mt-auto">
        <div
            class="border-border [&_small]:text-foreground-muted mb-1.5 flex items-center gap-2.5 border-b px-2.5 py-4 [&_small]:mt-1 [&_small]:block [&_small]:text-xs [&_strong]:block [&_strong]:text-xs">
            <EngineStatusIndicator />
            <div class="min-w-0 flex-1">
                <strong>Engine {gatewayOnline() ? 'online' : 'offline'}</strong>
                {#if gatewayOnline()}
                    <small>
                        {#if outputStream}
                            <span
                                class="block truncate"
                                title={activeOutput?.name ?? outputStream.output_id}>
                                {activeOutput?.name ?? outputStream.output_id}
                            </span>
                            <span class="block">
                                {formatSampleRate(outputStream.sample_rate)} · {outputStream.buffer_size ===
                                null
                                    ? 'default buffer'
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
            class={`hover:bg-surface-hover hover:text-foreground flex w-full cursor-pointer items-center gap-3 rounded-lg border-0 px-3 py-2.5 text-sm ${activeLabel === 'Settings' ? 'bg-accent/10 text-accent' : 'text-foreground-muted bg-transparent'}`}
            href="/settings">
            <Icon icon="lucide:settings" width="1.125rem" />
            <span>Settings</span>
        </a>
    </div>
</aside>

<nav
    class="border-border-strong bg-surface-1/97 fixed right-0 bottom-0 left-0 z-40 flex min-h-17.25 overflow-x-auto border-t px-[max(0.25rem,env(safe-area-inset-right))] pt-1.25 pb-[max(0.3125rem,env(safe-area-inset-bottom))] backdrop-blur-md min-[56.25rem]:hidden"
    aria-label="Primary navigation">
    {#each links as item}
        <a
            class={`relative flex min-w-17 flex-1 cursor-pointer flex-col items-center justify-center gap-1 border-0 bg-transparent text-xs ${activeLabel === item.label ? 'text-accent' : 'text-foreground-nav'}`}
            href={item.href}>
            <Icon icon={item.icon} width="1.1875rem" />
            <span>{item.label}</span>
        </a>
    {/each}
</nav>
