<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card mb-3 overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Engine MIDI configuration file
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                MIDI app controls and input auto-connections in one file.
            </p>
        </div>
        <Button
            variant="secondary"
            size="md"
            busy={section.exporting}
            disabled={section.exporting}
            onclick={() => void section.exportCurrent()}>
            <Icon
                icon={section.exporting ? 'lucide:loader-circle' : 'lucide:download'}
                width="1rem" />
            {section.exporting ? 'Exporting…' : 'Export active'}
        </Button>
    </header>

    <form
        class="grid gap-3 p-3.75 min-[37.5rem]:grid-cols-[minmax(0,1fr)_auto] min-[37.5rem]:items-end min-[56.25rem]:p-5"
        onsubmit={(event) => {
            event.preventDefault();
            void section.importConfiguration(false);
        }}>
        <div class="grid gap-1.5">
            <span class="text-sm font-semibold">Import configuration</span>
            <input
                id="engine-midi-configuration-file"
                bind:this={section.fileInput}
                class="sr-only"
                type="file"
                accept="application/json,.json"
                onchange={(event) =>
                    (section.selectedFile = event.currentTarget.files?.item(0) ?? null)} />
            <label
                class="border-border bg-surface-inset hover:bg-surface-hover focus-within:ring-accent flex h-11 min-w-0 cursor-pointer items-center gap-2 rounded-lg border px-3 text-sm transition-colors focus-within:ring-2"
                for="engine-midi-configuration-file">
                <Icon class="text-foreground-muted shrink-0" icon="lucide:file-up" width="1rem" />
                <span class={section.selectedFile ? 'truncate' : 'text-foreground-muted truncate'}>
                    {section.selectedFile?.name ?? 'Choose a JSON configuration file'}
                </span>
            </label>
        </div>
        <div class="flex gap-2">
            <Button
                variant="secondary"
                size="lg"
                type="button"
                busy={section.importing}
                disabled={!section.selectedFile || section.importing}
                onclick={() => void section.importConfiguration(true)}>
                Set active
            </Button>
            <Button
                variant="primary"
                size="lg"
                type="submit"
                busy={section.importing}
                disabled={!section.selectedFile || section.importing}>
                Add to library
            </Button>
        </div>
    </form>
</section>
