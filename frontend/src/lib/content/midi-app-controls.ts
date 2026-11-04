export type EngineMidiControlType = 'note-toggle' | 'note-enable' | 'note-disable' | 'cc-switch';

export const engineMidiControlTypeOptions = [
    {
        value: 'note-toggle',
        label: 'Note toggles transport',
        description: 'Each matching Note On toggles Play/Stop.'
    },
    {
        value: 'note-enable',
        label: 'Note starts transport',
        description: 'Each matching Note On starts playback.'
    },
    {
        value: 'note-disable',
        label: 'Note stops transport',
        description: 'Each matching Note On stops playback.'
    },
    {
        value: 'cc-switch',
        label: 'CC switches transport',
        description: 'Values at or above the threshold play; lower values stop.'
    }
];

export type ProjectMidiControlType =
    | 'note-controller-toggle'
    | 'note-controller-enable'
    | 'note-controller-disable'
    | 'cc-controller-switch'
    | 'note-preset'
    | 'cc-preset';

export const projectMidiControlTypeOptions: {
    value: ProjectMidiControlType;
    label: string;
    description: string;
    availability: 'controller' | 'preset';
}[] = [
    {
        value: 'note-controller-toggle',
        label: 'Note toggles controller',
        description: 'Toggle an instrument controller on each matching Note On.',
        availability: 'controller'
    },
    {
        value: 'note-controller-enable',
        label: 'Note enables controller',
        description: 'Enable an instrument controller on each matching Note On.',
        availability: 'controller'
    },
    {
        value: 'note-controller-disable',
        label: 'Note disables controller',
        description: 'Disable an instrument controller on each matching Note On.',
        availability: 'controller'
    },
    {
        value: 'cc-controller-switch',
        label: 'CC switches controller',
        description: 'Enable or disable a controller according to a CC threshold.',
        availability: 'controller'
    },
    {
        value: 'note-preset',
        label: 'Note selects user preset',
        description: 'Activate a user preset on each matching Note On.',
        availability: 'preset'
    },
    {
        value: 'cc-preset',
        label: 'CC selects user preset',
        description: 'Activate a user preset when the matching CC is nonzero.',
        availability: 'preset'
    }
];
