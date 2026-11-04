<script lang="ts">
    import Icon from '@iconify/svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
    }

    let { section, machine }: Props = $props();
</script>

<div class="w-full max-w-full min-w-0 overflow-x-auto overscroll-x-contain pb-1">
    <div class="grid gap-1.5">
        {#each machine.data.lanes as lane (lane.id)}
            {@const laneSequence = section.sequence(machine, lane)}
            <div
                class="grid items-center gap-1"
                style:grid-template-columns={`15rem repeat(${laneSequence?.length_steps ?? 0}, minmax(1.75rem, 1fr))`}
                style:min-width={`${15 + (laneSequence?.length_steps ?? 0) * 2}rem`}>
                <div class="flex min-w-0 items-center gap-1">
                    <button
                        class="text-foreground hover:text-accent min-w-0 flex-1 cursor-pointer truncate text-left text-xs"
                        onclick={() => section.selectVoice(machine, lane)}
                        title="Edit voice routing">
                        {lane.name}
                    </button>
                    <button
                        class={`border-border text-foreground-muted hover:border-accent/60 hover:text-accent grid size-6 cursor-pointer place-items-center rounded border ${section.selectedVoice?.machineId === machine.id && section.selectedVoice.laneId === lane.id ? 'border-accent text-accent' : ''}`}
                        onclick={() => section.selectVoice(machine, lane)}
                        aria-label={`Edit ${lane.name}`}>
                        <Icon icon="lucide:pencil" width="0.75rem" />
                    </button>
                    <button
                        class={`size-6 cursor-pointer rounded border text-[0.625rem] transition-colors ${lane.muted ? 'border-danger bg-danger/25 text-danger-foreground ring-danger/40 ring-1' : 'border-border text-foreground-muted hover:border-danger/60 hover:text-danger-foreground'}`}
                        onclick={() => {
                            lane.muted = !lane.muted;
                            return section.saveMachine(machine);
                        }}
                        aria-pressed={lane.muted}
                        aria-label={`Mute ${lane.name}`}>
                        M
                    </button>
                    <button
                        class={`size-6 cursor-pointer rounded border text-[0.625rem] transition-colors ${lane.solo ? 'border-accent bg-accent/25 text-accent ring-accent/40 ring-1' : 'border-border text-foreground-muted hover:border-accent/60 hover:text-accent'}`}
                        onclick={() => {
                            lane.solo = !lane.solo;
                            return section.saveMachine(machine);
                        }}
                        aria-pressed={lane.solo}
                        aria-label={`Solo ${lane.name}`}>
                        S
                    </button>
                    <TimedConfirmButton
                        size="xs"
                        label="Clear"
                        confirmLabel="Sure?"
                        busyLabel="Clearing"
                        disabled={section.saving.has(machine.id) ||
                            !section.laneHasSteps(machine, lane)}
                        onconfirm={() => section.clearLane(machine, lane)} />
                </div>
                {#each laneSequence?.steps.slice(0, laneSequence.length_steps) ?? [] as step, index}
                    <button
                        class={`border-border hover:border-accent/60 aspect-square cursor-pointer rounded border transition-colors ${step.enabled ? 'bg-accent' : 'bg-surface-input'} ${section.selected(machine, lane, index) ? 'ring-accent/40 ring-2' : ''} ${section.previewed(laneSequence, index) ? 'outline-accent outline-2 -outline-offset-3' : ''} ${index % section.meterBeatSteps === 0 ? 'border-l-accent border-l-2' : ''}`}
                        style:opacity={step.enabled ? Math.max(0.35, step.velocity / 127) : 1}
                        onclick={() => section.toggleStep(machine, lane, index)}
                        aria-current={section.previewed(laneSequence, index) ? 'step' : undefined}
                        aria-label={`${lane.name} step ${index + 1}`}>
                    </button>
                {/each}
            </div>
        {/each}
    </div>
</div>
