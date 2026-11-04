export const midiNoteNames = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];

export const midiChannels = Array.from({ length: 16 }, (_, value) => value);
export const midiSources = Array.from({ length: 16 }, (_, value) => value);
export const midiChannelColors = midiChannels.map(
    (channel) => `var(--midi-channel-${channel + 1})`
);
