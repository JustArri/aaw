import type { EqualizerBand } from '$lib/generated/models';

export type EqualizerDefinition = { bands: EqualizerBand[] };

export type EqualizerPreset = {
    name: string;
    definition: EqualizerDefinition;
};

export const equalizerPresets: EqualizerPreset[] = [
    { name: 'Flat Reference', definition: { bands: [] } },
    {
        name: 'Natural Hi-Fi',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 110, gain_db: 1.2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 320, gain_db: -0.8, resonance: 0.7 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Analog',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 160, gain_db: 1.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 450, gain_db: 0.7, resonance: 0.65 } },
                { type: 'Peaking', data: { frequency: 3400, gain_db: -1.2, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Soft Listening',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 140, gain_db: 0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 3000, gain_db: -1.5, resonance: 0.75 } },
                { type: 'HighShelf', data: { frequency: 7500, gain_db: -1.3, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Low-Volume Loudness',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 160, gain_db: 3, resonance: 0.707 } },
                { type: 'HighShelf', data: { frequency: 6500, gain_db: 1.5, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Bright Detailed',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: 1, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: 1.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Natural Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 100, gain_db: 0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 330, gain_db: -1.5, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: 0.7, resonance: 0.7 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 0.5, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 140, gain_db: 1.7, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 320, gain_db: -0.7, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2900, gain_db: -1.6, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Bright Piano',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -1.5, resonance: 0.9 } },
                { type: 'Peaking', data: { frequency: 3200, gain_db: 1.5, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 1.2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Natural Acoustic Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 100, gain_db: -1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 270, gain_db: -1.8, resonance: 0.9 } },
                { type: 'Peaking', data: { frequency: 2200, gain_db: 0.9, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 1.2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Acoustic Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 150, gain_db: 1.2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 350, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 3300, gain_db: -1.2, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: -0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Smooth Violin',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 180, gain_db: -0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 450, gain_db: 0.8, resonance: 0.75 } },
                { type: 'Peaking', data: { frequency: 3300, gain_db: -2, resonance: 1 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: -0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Present Violin',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -0.8, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1800, gain_db: 1.2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 4200, gain_db: -0.8, resonance: 1.1 } },
                { type: 'HighShelf', data: { frequency: 9500, gain_db: 0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Cello',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 140, gain_db: 1.6, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 350, gain_db: -0.9, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2700, gain_db: -1.3, resonance: 0.95 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Orchestral Strings',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 130, gain_db: 0.5, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 350, gain_db: -1, resonance: 0.75 } },
                { type: 'Peaking', data: { frequency: 3100, gain_db: -1.2, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 10000, gain_db: 0.5, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Electric Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 150, gain_db: 1.5, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 450, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2400, gain_db: -1.2, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Clarity Cleanup',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 300, gain_db: -2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1800, gain_db: 0.8, resonance: 0.75 } }
            ]
        }
    },
    {
        name: 'Harshness Reduction',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 3500, gain_db: -2.5, resonance: 1.1 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },

    // Extended musical and corrective presets.
    {
        name: 'Soft Felt Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 130, gain_db: 1.1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 2200, gain_db: -2.2, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 6500, gain_db: -1.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Open Classical Piano',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 280, gain_db: -0.9, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2100, gain_db: 0.6, resonance: 0.75 } },
                { type: 'HighShelf', data: { frequency: 9500, gain_db: 1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Intimate Jazz Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 120, gain_db: 1.3, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 420, gain_db: -1.2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2600, gain_db: -0.5, resonance: 0.8 } }
            ]
        }
    },
    {
        name: 'Cinematic Piano',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 95, gain_db: 2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 350, gain_db: -1.3, resonance: 0.75 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 1.1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Soft Piano Hammers',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 3100, gain_db: -2, resonance: 0.95 } },
                { type: 'HighShelf', data: { frequency: 8200, gain_db: -0.9, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Gentle Harpsichord',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 2400, gain_db: -1.6, resonance: 0.9 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: -1.2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Shimmering Harp',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 290, gain_db: -1, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 2400, gain_db: 0.8, resonance: 0.7 } },
                { type: 'HighShelf', data: { frequency: 8800, gain_db: 1.3, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Fingerstyle Acoustic Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 125, gain_db: 0.7, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 240, gain_db: -1.4, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 2900, gain_db: -0.7, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 0.6, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Strummed Acoustic Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 110, gain_db: -1.1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 320, gain_db: -2, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 2900, gain_db: 1, resonance: 0.85 } }
            ]
        }
    },
    {
        name: 'Soft Nylon Guitar',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 330, gain_db: -1, resonance: 0.75 } },
                { type: 'Peaking', data: { frequency: 2400, gain_db: -1.1, resonance: 0.95 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Clear Nylon Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 110, gain_db: -0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 350, gain_db: -1.4, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1900, gain_db: 1, resonance: 0.8 } }
            ]
        }
    },
    {
        name: 'Sparkling Steel-String Guitar',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 280, gain_db: -1.5, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 3100, gain_db: 0.7, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 9500, gain_db: 1.6, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Clean Electric Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 120, gain_db: -1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 320, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2200, gain_db: 1, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Jazz Electric Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 180, gain_db: 1.1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 1800, gain_db: -1, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 6000, gain_db: -2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Defined Bass Guitar',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 90, gain_db: 1.1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 260, gain_db: -1.8, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 1200, gain_db: 1.2, resonance: 0.9 } }
            ]
        }
    },
    {
        name: 'Woody Upright Bass',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 95, gain_db: 1.1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 280, gain_db: -0.8, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1400, gain_db: -0.9, resonance: 0.9 } }
            ]
        }
    },
    {
        name: 'Warm Violin',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 500, gain_db: 0.9, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: -1.4, resonance: 0.9 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: -0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Airy Violin',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 3100, gain_db: -0.8, resonance: 1 } },
                { type: 'HighShelf', data: { frequency: 9500, gain_db: 1.2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Mellow Viola',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 180, gain_db: 0.7, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 440, gain_db: -0.8, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2900, gain_db: -1.7, resonance: 0.9 } }
            ]
        }
    },
    {
        name: 'Defined Viola',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -1.1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1650, gain_db: 1, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 0.4, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Present Solo Cello',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 300, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1500, gain_db: 1.2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 3700, gain_db: -0.7, resonance: 1 } }
            ]
        }
    },
    {
        name: 'Orchestral Double Bass',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 90, gain_db: 1.4, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 260, gain_db: -1.2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 950, gain_db: 0.6, resonance: 0.8 } }
            ]
        }
    },
    {
        name: 'Warm Cinematic Strings',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 150, gain_db: 1.2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 340, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 3100, gain_db: -1.7, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: -0.5, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Bright String Ensemble',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 350, gain_db: -1.2, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2100, gain_db: 0.9, resonance: 0.7 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Airy Flute',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 330, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2700, gain_db: -0.5, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 9500, gain_db: 1.3, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Soft Flute',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 2600, gain_db: -1.4, resonance: 1 } },
                { type: 'HighShelf', data: { frequency: 8200, gain_db: -0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Woody Clarinet',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 320, gain_db: 0.8, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 1500, gain_db: -1, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8000, gain_db: -0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Smooth Oboe',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 1400, gain_db: -1.3, resonance: 1 } },
                { type: 'Peaking', data: { frequency: 3000, gain_db: -1.2, resonance: 0.95 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -0.5, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Bassoon',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 160, gain_db: 1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 400, gain_db: -0.9, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2500, gain_db: -1, resonance: 0.9 } }
            ]
        }
    },
    {
        name: 'Mellow Saxophone',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 180, gain_db: 0.9, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 850, gain_db: -0.7, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: -1.4, resonance: 0.9 } }
            ]
        }
    },
    {
        name: 'Present Saxophone',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 340, gain_db: -1.1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 1900, gain_db: 1.4, resonance: 0.9 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 0.6, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Rounded Brass',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 450, gain_db: -0.8, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2600, gain_db: -1.8, resonance: 0.9 } },
                { type: 'HighShelf', data: { frequency: 8700, gain_db: -0.6, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Brass Fanfare',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 330, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2200, gain_db: 1.2, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Choir',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 180, gain_db: 0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 330, gain_db: -1.4, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2900, gain_db: -0.8, resonance: 0.85 } }
            ]
        }
    },
    {
        name: 'Airy Choir',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 330, gain_db: -1.5, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2700, gain_db: 0.5, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 1.2, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Rounded Marimba',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 150, gain_db: 0.8, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 700, gain_db: -1, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 3500, gain_db: -1, resonance: 0.8 } }
            ]
        }
    },
    {
        name: 'Glassy Vibraphone',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 420, gain_db: -1.1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: 0.8, resonance: 0.85 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: 1.1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Soft Kalimba',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 1100, gain_db: -0.7, resonance: 0.85 } },
                { type: 'Peaking', data: { frequency: 4000, gain_db: -1.7, resonance: 1 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: -0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Organ',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 130, gain_db: 1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 450, gain_db: -1, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 7500, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Bell-Like Electric Piano',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 340, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 2700, gain_db: 1.2, resonance: 0.9 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: 0.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Warm Synth Pad',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 170, gain_db: 1, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 2800, gain_db: -1.6, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 8200, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Smooth Synth Lead',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 330, gain_db: -1, resonance: 0.8 } },
                { type: 'Peaking', data: { frequency: 3200, gain_db: -1.8, resonance: 1 } },
                { type: 'HighShelf', data: { frequency: 8500, gain_db: -0.7, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Soft Lo-Fi',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 160, gain_db: 1.2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 3000, gain_db: -1.4, resonance: 0.8 } },
                { type: 'HighShelf', data: { frequency: 6500, gain_db: -2.8, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Midrange Focus',
        definition: {
            bands: [
                { type: 'LowShelf', data: { frequency: 140, gain_db: -1.2, resonance: 0.707 } },
                { type: 'Peaking', data: { frequency: 1700, gain_db: 1.1, resonance: 0.75 } },
                { type: 'HighShelf', data: { frequency: 9000, gain_db: -1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Boxiness Reduction',
        definition: {
            bands: [{ type: 'Peaking', data: { frequency: 480, gain_db: -2.3, resonance: 1 } }]
        }
    },
    {
        name: 'Nasal Reduction',
        definition: {
            bands: [{ type: 'Peaking', data: { frequency: 1050, gain_db: -2, resonance: 1.2 } }]
        }
    },
    {
        name: 'Presence Taming',
        definition: {
            bands: [{ type: 'Peaking', data: { frequency: 2700, gain_db: -1.8, resonance: 1 } }]
        }
    },
    {
        name: 'Gentle Air',
        definition: {
            bands: [
                { type: 'HighShelf', data: { frequency: 9800, gain_db: 1.1, resonance: 0.707 } }
            ]
        }
    },
    {
        name: 'Sub-Bass Control',
        definition: {
            bands: [{ type: 'LowShelf', data: { frequency: 75, gain_db: -2, resonance: 0.707 } }]
        }
    },
    {
        name: 'Low-Mid Cleanup',
        definition: {
            bands: [
                { type: 'Peaking', data: { frequency: 250, gain_db: -1.7, resonance: 0.75 } },
                { type: 'Peaking', data: { frequency: 450, gain_db: -0.8, resonance: 0.85 } }
            ]
        }
    }
];
