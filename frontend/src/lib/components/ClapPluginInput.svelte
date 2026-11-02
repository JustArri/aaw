<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { ClapAsset } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import Button from './Button.svelte';
    import SelectDropdown from './SelectDropdown.svelte';

    type Props = {
        id: string;
        plugins: ClapAsset[];
        filePath?: string;
        pluginId?: string;
        compact?: boolean;
        required?: boolean;
        onreload?: () => unknown | Promise<unknown>;
    };

    let {
        id,
        plugins,
        filePath = $bindable(''),
        pluginId = $bindable(''),
        compact = false,
        required = false,
        onreload
    }: Props = $props();

    let selectedIndex = $state('');
    let reloading = $state(false);
    let reloadDone = $state(false);

    let options = $derived(
        plugins.map((plugin, index) => ({
            value: String(index),
            label: plugin.name || plugin.plugin_id,
            selectionLabel: plugin.version
                ? `${plugin.name || plugin.plugin_id} · ${plugin.version}`
                : plugin.name || plugin.plugin_id,
            description: [
                plugin.version ? `Version ${plugin.version}` : undefined,
                plugin.file_path,
                plugin.plugin_id
            ]
                .filter(Boolean)
                .join(' · ')
        }))
    );

    $effect(() => {
        const index = plugins.findIndex(
            (plugin) => plugin.file_path === filePath && plugin.plugin_id === pluginId
        );
        const nextIndex = index < 0 ? '' : String(index);
        if (selectedIndex !== nextIndex) {
            selectedIndex = nextIndex;
        }
    });

    function selectPlugin() {
        const plugin = plugins[Number(selectedIndex)];
        if (!plugin) {
            filePath = '';
            pluginId = '';
            return;
        }
        filePath = plugin.file_path;
        pluginId = plugin.plugin_id;
    }

    async function reload() {
        if (reloading) {
            return;
        }
        reloading = true;
        reloadDone = false;
        try {
            await onreload?.();
            reloadDone = true;
            setTimeout(() => {
                reloadDone = false;
            }, 1200);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : 'Could not reload plugins.', {
                title: 'Reload plugins'
            });
        } finally {
            reloading = false;
        }
    }
</script>

<div class="flex min-w-0 gap-2">
    <div class="relative min-w-0 flex-1">
        <SelectDropdown
            {id}
            bind:value={selectedIndex}
            {options}
            placeholder="Select a CLAP plugin"
            {compact}
            clearable
            searchable
            searchPlaceholder="Search plugins, versions, paths, or IDs"
            noMatchesText="No matching CLAP plugins"
            ariaLabel="CLAP plugin"
            onchange={selectPlugin} />
        {#if required}
            <input
                class="pointer-events-none absolute size-px opacity-0"
                type="text"
                tabindex="-1"
                aria-hidden="true"
                value={pluginId}
                required />
        {/if}
    </div>

    <Button
        variant="secondary"
        size={compact ? 'md' : 'lg'}
        square
        class="text-foreground-muted enabled:hover:border-accent/40 enabled:hover:bg-accent/10 enabled:hover:text-accent"
        type="button"
        aria-label="Reload CLAP plugins"
        title="Reload CLAP plugins"
        busy={reloading}
        disabled={reloading}
        onclick={reload}>
        <Icon
            icon={reloading
                ? 'lucide:loader-circle'
                : reloadDone
                  ? 'lucide:check'
                  : 'lucide:refresh-cw'}
            width="1.125rem"
            class={reloading ? 'animate-spin' : reloadDone ? 'text-success' : ''} />
    </Button>
</div>
