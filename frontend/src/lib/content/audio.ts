export type LatencyMode = 'Automatic' | 'Low' | 'Stable' | 'CustomBufferSize';
export type SampleRateMode = 'Automatic' | 'Fixed';

export const latencyModes: { value: LatencyMode; label: string }[] = [
    { value: 'Automatic', label: 'Automatic' },
    { value: 'Low', label: 'Low latency' },
    { value: 'Stable', label: 'Stable' },
    { value: 'CustomBufferSize', label: 'Custom' }
];

export const sampleRateModes: { value: SampleRateMode; label: string }[] = [
    { value: 'Automatic', label: 'Automatic' },
    { value: 'Fixed', label: 'Fixed' }
];
