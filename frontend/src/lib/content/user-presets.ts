export const userPresetActionTypeOptions = [
    {
        value: 'disable-all',
        label: 'Disable all controllers',
        description: 'Disable every controller before applying later actions.'
    },
    {
        value: 'set-controller',
        label: 'Set controller state',
        description: 'Enable or disable one instrument controller.'
    },
    {
        value: 'set-program',
        label: 'Set SoundFont program',
        description: 'Select a bank and program on a SoundFont instrument.'
    },
    {
        value: 'set-clap-parameter',
        label: 'Set CLAP parameter',
        description: 'Set one writable parameter on a CLAP plugin instrument.'
    }
];

export const enabledOptions = [
    { value: 'true', label: 'Enabled' },
    { value: 'false', label: 'Disabled' }
];
