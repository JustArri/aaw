<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="grid gap-3 p-3.75 min-[56.25rem]:p-5"
    onsubmit={(event) => {
        event.preventDefault();
        void section.importProject(false);
    }}>
    <input
        class="sr-only"
        id="project-file"
        bind:this={section.fileInput}
        type="file"
        accept="application/json,.json"
        onchange={(event) =>
            section.selectProjectFile(event.currentTarget.files?.item(0) ?? null)} />
    <label
        class={`grid min-h-36 cursor-pointer place-items-center rounded-xl border border-dashed px-4 py-5 text-center transition-colors ${section.draggingFile ? 'border-accent bg-accent/10' : section.selectedFile ? 'border-accent/50 bg-accent/5 hover:bg-accent/8' : 'border-border-strong bg-surface-inset hover:border-accent/50 hover:bg-surface-hover'}`}
        for="project-file"
        ondragenter={(event) => section.handleDragEnter(event)}
        ondragover={(event) => section.handleDragOver(event)}
        ondragleave={(event) => section.handleDragLeave(event)}
        ondrop={(event) => section.handleDrop(event)}>
        <span class="grid justify-items-center gap-2.5">
            <span
                class={`grid size-11 place-items-center rounded-xl ${section.selectedFile || section.draggingFile ? 'bg-accent/12 text-accent' : 'bg-surface-icon text-foreground-muted'}`}>
                <Icon
                    icon={section.draggingFile
                        ? 'lucide:download'
                        : section.selectedFile
                          ? 'lucide:file-check-2'
                          : 'lucide:file-up'}
                    width="1.25rem" />
            </span>
            {#if section.draggingFile}
                <span class="text-sm font-semibold">Drop the project here</span>
            {:else if section.selectedFile}
                <span class="grid min-w-0 gap-1">
                    <strong
                        class="max-w-full overflow-hidden text-sm text-ellipsis whitespace-nowrap">
                        {section.selectedFile.name}
                    </strong>
                    <small class="text-foreground-muted text-sm">
                        {section.formatFileSize(section.selectedFile.size)} · Drop another file or tap
                        to replace
                    </small>
                </span>
            {:else}
                <span class="grid gap-1">
                    <strong class="text-sm">Drop a project JSON file here</strong>
                    <small class="text-foreground-muted text-sm">or tap to browse files</small>
                </span>
            {/if}
        </span>
    </label>
    <div class="flex flex-wrap items-center justify-end gap-2">
        {#if section.selectedFile}
            <Button
                variant="ghost"
                size="lg"
                class="mr-auto"
                type="button"
                disabled={section.importing}
                onclick={() => section.clearSelectedFile()}>
                Remove file
            </Button>
        {/if}
        <Button
            variant="secondary"
            size="lg"
            class="bg-surface-card"
            type="button"
            busy={section.importing}
            disabled={!section.selectedFile || section.importing}
            onclick={() => void section.importProject(true)}>
            {section.importing ? 'Importing…' : 'Set active'}
        </Button>
        <Button
            variant="primary"
            size="lg"
            type="submit"
            busy={section.importing}
            disabled={!section.selectedFile || section.importing}>
            {section.importing ? 'Importing…' : 'Add to library'}
        </Button>
    </div>
</form>
