<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';
    import ProjectImportForm from './ProjectImportForm.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card mb-3 overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Project file
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                Import or export complete project JSON files.
            </p>
        </div>
        <Button
            variant="secondary"
            size="md"
            class="bg-surface-card"
            type="button"
            busy={section.exporting}
            disabled={section.exporting}
            onclick={() => void section.exportCurrent()}>
            <span class="flex items-center gap-2">
                <Icon
                    class={section.exporting ? 'animate-spin' : ''}
                    icon={section.exporting ? 'lucide:loader-circle' : 'lucide:download'}
                    width="0.9375rem" />
                {section.exporting ? 'Exporting…' : 'Export active'}
            </span>
        </Button>
    </header>

    {#if section.fileErrorMessage}
        <div
            class="border-danger/35 bg-danger/8 text-danger flex items-center gap-2 border-b px-3.75 py-3 text-sm min-[56.25rem]:px-5"
            role="alert">
            <Icon class="shrink-0" icon="lucide:circle-alert" width="1rem" />
            <span>{section.fileErrorMessage}</span>
        </div>
    {/if}

    <ProjectImportForm {section} />
</section>
