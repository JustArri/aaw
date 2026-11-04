export type LanePreset = {
    value: string;
    label: string;
    description: string;
    lanes: ReadonlyArray<readonly [name: string, note: number]>;
};

export type BeatPreset = {
    value: string;
    label: string;
    description: string;
    voices: ReadonlyArray<{
        notes: readonly number[];
        preferredNote?: number;
        hits: ReadonlyArray<readonly [step: number, velocity: number]>;
    }>;
};

export const drumVoiceNames: Readonly<Record<number, string>> = {
    35: 'Acoustic bass drum',
    36: 'Kick',
    37: 'Side stick',
    38: 'Acoustic snare',
    39: 'Clap',
    40: 'Electronic snare',
    42: 'Closed hi-hat',
    44: 'Pedal hi-hat',
    46: 'Open hi-hat',
    49: 'Crash',
    51: 'Ride',
    56: 'Cowbell',
    60: 'High bongo',
    61: 'Low bongo',
    62: 'Mute high conga',
    63: 'Open high conga',
    64: 'Low conga',
    65: 'High timbale',
    66: 'Low timbale',
    70: 'Maracas',
    75: 'Clave'
};

export const lanePresets: LanePreset[] = [
    {
        value: 'classic',
        label: 'Classic drum kit',
        description: 'Kick, snare, hi-hats, toms, crash and ride',
        lanes: [
            ['Kick', 36],
            ['Snare', 38],
            ['Side stick', 37],
            ['Closed hi-hat', 42],
            ['Pedal hi-hat', 44],
            ['Open hi-hat', 46],
            ['Low tom', 45],
            ['Mid tom', 47],
            ['High tom', 50],
            ['Crash', 49],
            ['Ride', 51]
        ]
    },
    {
        value: 'compact',
        label: 'Compact kit',
        description: 'Kick, snare, clap and two hi-hats',
        lanes: [
            ['Kick', 36],
            ['Snare', 38],
            ['Clap', 39],
            ['Closed hi-hat', 42],
            ['Open hi-hat', 46]
        ]
    },
    {
        value: 'electronic',
        label: 'Electronic kit',
        description: 'Electronic drums with clap, toms and cymbals',
        lanes: [
            ['Kick', 36],
            ['Electronic snare', 40],
            ['Clap', 39],
            ['Closed hi-hat', 42],
            ['Open hi-hat', 46],
            ['Low electronic tom', 41],
            ['Mid electronic tom', 45],
            ['High electronic tom', 48],
            ['Crash', 49],
            ['Ride', 51]
        ]
    },
    {
        value: 'latin',
        label: 'Latin percussion',
        description: 'Bongos, congas, timbales, cowbell, clave and maracas',
        lanes: [
            ['High bongo', 60],
            ['Low bongo', 61],
            ['Mute high conga', 62],
            ['Open high conga', 63],
            ['Low conga', 64],
            ['High timbale', 65],
            ['Low timbale', 66],
            ['Cowbell', 56],
            ['Clave', 75],
            ['Maracas', 70]
        ]
    }
];

export const beatPresets: BeatPreset[] = [
    {
        value: 'rock',
        label: 'Basic rock',
        description: 'Kick and snare backbeat with eighth-note closed hi-hat',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 120],
                    [6, 105],
                    [8, 118]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 118],
                    [12, 122]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 88],
                    [2, 72],
                    [4, 92],
                    [6, 72],
                    [8, 88],
                    [10, 72],
                    [12, 92],
                    [14, 72]
                ]
            }
        ]
    },
    {
        value: 'four-floor',
        label: 'Four on the floor',
        description: 'Quarter-note kick, backbeat and alternating hi-hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 122],
                    [4, 116],
                    [8, 122],
                    [12, 116]
                ]
            },
            {
                notes: [38, 39, 40],
                hits: [
                    [4, 115],
                    [12, 120]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [2, 82],
                    [6, 82],
                    [10, 82]
                ]
            },
            { notes: [46], hits: [[14, 96]] }
        ]
    },
    {
        value: 'hip-hop',
        label: 'Hip-hop',
        description: 'Syncopated kick, snare backbeat and sixteenth-note hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [3, 94],
                    [7, 112],
                    [10, 120],
                    [14, 88]
                ]
            },
            {
                notes: [38, 39, 40],
                hits: [
                    [4, 116],
                    [12, 122]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 82],
                    [1, 62],
                    [2, 75],
                    [3, 62],
                    [4, 86],
                    [5, 62],
                    [6, 76],
                    [7, 66],
                    [8, 82],
                    [9, 62],
                    [10, 78],
                    [11, 62],
                    [12, 88],
                    [13, 64],
                    [14, 78],
                    [15, 70]
                ]
            }
        ]
    },
    {
        value: 'reggae',
        label: 'One-drop reggae',
        description: 'One-drop kick and side-stick with offbeat hi-hat',
        voices: [
            { notes: [35, 36], hits: [[8, 116]] },
            { notes: [37, 38], hits: [[8, 112]] },
            {
                notes: [42, 44],
                hits: [
                    [2, 78],
                    [6, 84],
                    [10, 78],
                    [14, 88]
                ]
            }
        ]
    },
    {
        value: 'breakbeat',
        label: 'Breakbeat',
        description: 'Broken kick pattern, snare backbeat and driving hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [6, 106],
                    [10, 120],
                    [15, 90]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 120],
                    [12, 124],
                    [15, 55]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 84],
                    [2, 70],
                    [4, 88],
                    [6, 72],
                    [8, 84],
                    [10, 74],
                    [12, 90],
                    [14, 76]
                ]
            },
            { notes: [46], hits: [[7, 90]] }
        ]
    },
    {
        value: 'disco',
        label: 'Disco',
        description: 'Steady kick, backbeat, open offbeat hats and ride',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 122],
                    [4, 118],
                    [8, 122],
                    [12, 118]
                ]
            },
            {
                notes: [38, 39, 40],
                hits: [
                    [4, 118],
                    [12, 122]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 72],
                    [4, 78],
                    [8, 72],
                    [12, 80]
                ]
            },
            {
                notes: [46],
                hits: [
                    [2, 92],
                    [6, 94],
                    [10, 92],
                    [14, 98]
                ]
            },
            {
                notes: [51, 53, 59],
                hits: [
                    [0, 68],
                    [4, 70],
                    [8, 68],
                    [12, 72]
                ]
            }
        ]
    },
    {
        value: 'house',
        label: 'House',
        description: 'Four-on-the-floor kick, clap and offbeat open hi-hat',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [4, 120],
                    [8, 124],
                    [12, 120]
                ]
            },
            {
                notes: [38, 39, 40],
                preferredNote: 39,
                hits: [
                    [4, 114],
                    [12, 118]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 66],
                    [4, 72],
                    [8, 66],
                    [12, 74]
                ]
            },
            {
                notes: [46],
                hits: [
                    [2, 102],
                    [6, 98],
                    [10, 102],
                    [14, 106]
                ]
            }
        ]
    },
    {
        value: 'techno',
        label: 'Techno',
        description: 'Heavy quarter-note kick, clap and relentless closed hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 127],
                    [4, 124],
                    [8, 127],
                    [12, 124]
                ]
            },
            {
                notes: [38, 39, 40],
                preferredNote: 39,
                hits: [
                    [4, 105],
                    [12, 110]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 72],
                    [2, 86],
                    [4, 74],
                    [6, 90],
                    [8, 72],
                    [10, 88],
                    [12, 76],
                    [14, 94]
                ]
            },
            { notes: [46], hits: [[14, 96]] }
        ]
    },
    {
        value: 'electro',
        label: 'Electro',
        description: 'Syncopated electronic kick, snare and metallic hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [3, 96],
                    [8, 118],
                    [10, 108],
                    [15, 92]
                ]
            },
            {
                notes: [39, 40],
                preferredNote: 40,
                hits: [
                    [4, 120],
                    [12, 124]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 82],
                    [2, 68],
                    [4, 88],
                    [6, 70],
                    [8, 80],
                    [10, 72],
                    [12, 90],
                    [14, 76]
                ]
            },
            {
                notes: [46],
                hits: [
                    [7, 92],
                    [15, 98]
                ]
            }
        ]
    },
    {
        value: 'trap',
        label: 'Trap half-time',
        description: 'Half-time clap, sparse kick and rapid hi-hat subdivisions',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [7, 100],
                    [10, 116],
                    [14, 92]
                ]
            },
            { notes: [38, 39, 40], preferredNote: 39, hits: [[8, 124]] },
            {
                notes: [42, 44],
                hits: [
                    [0, 82],
                    [1, 64],
                    [2, 76],
                    [3, 66],
                    [4, 80],
                    [5, 64],
                    [6, 76],
                    [7, 68],
                    [8, 86],
                    [9, 66],
                    [10, 80],
                    [11, 68],
                    [12, 82],
                    [13, 70],
                    [14, 86],
                    [15, 76]
                ]
            },
            {
                notes: [46],
                hits: [
                    [6, 88],
                    [14, 92]
                ]
            }
        ]
    },
    {
        value: 'boom-bap',
        label: 'Boom bap',
        description: 'Punchy sampled kick and snare with loose eighth-note hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [5, 94],
                    [7, 110],
                    [10, 120],
                    [15, 86]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 122],
                    [12, 126]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 78],
                    [2, 66],
                    [4, 84],
                    [6, 68],
                    [8, 76],
                    [10, 70],
                    [12, 86],
                    [14, 72]
                ]
            }
        ]
    },
    {
        value: 'funk',
        label: 'Funk',
        description: 'Syncopated kick, ghost snare and busy hi-hat accents',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [3, 96],
                    [7, 112],
                    [10, 118],
                    [14, 98]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 120],
                    [7, 48],
                    [11, 52],
                    [12, 124],
                    [15, 55]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 92],
                    [1, 58],
                    [2, 72],
                    [3, 60],
                    [4, 94],
                    [5, 62],
                    [6, 76],
                    [7, 64],
                    [8, 90],
                    [9, 60],
                    [10, 78],
                    [11, 62],
                    [12, 96],
                    [13, 64],
                    [14, 80],
                    [15, 66]
                ]
            },
            {
                notes: [46],
                hits: [
                    [6, 88],
                    [14, 94]
                ]
            }
        ]
    },
    {
        value: 'punk',
        label: 'Punk rock',
        description: 'Fast alternating kick and snare with constant eighth-note hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [4, 116],
                    [8, 124],
                    [10, 108],
                    [12, 116]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [2, 118],
                    [6, 122],
                    [10, 118],
                    [14, 124]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 94],
                    [2, 86],
                    [4, 94],
                    [6, 88],
                    [8, 96],
                    [10, 88],
                    [12, 96],
                    [14, 90]
                ]
            },
            { notes: [49, 52, 55, 57], hits: [[0, 120]] }
        ]
    },
    {
        value: 'metal',
        label: 'Metal drive',
        description: 'Double-kick drive, hard snare and crash-led cymbals',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 126],
                    [1, 108],
                    [2, 116],
                    [3, 108],
                    [6, 114],
                    [7, 106],
                    [8, 126],
                    [9, 108],
                    [10, 116],
                    [11, 108],
                    [14, 116],
                    [15, 110]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 126],
                    [12, 127]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [2, 86],
                    [6, 90],
                    [10, 88],
                    [14, 92]
                ]
            },
            {
                notes: [49, 52, 55, 57],
                hits: [
                    [0, 124],
                    [8, 116]
                ]
            }
        ]
    },
    {
        value: 'drum-bass',
        label: 'Drum and bass',
        description: 'Two-step break with syncopated kick and fast hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [7, 104],
                    [10, 120]
                ]
            },
            {
                notes: [38, 40],
                hits: [
                    [4, 124],
                    [12, 126]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 84],
                    [1, 60],
                    [2, 72],
                    [3, 62],
                    [4, 88],
                    [5, 62],
                    [6, 74],
                    [7, 66],
                    [8, 82],
                    [9, 62],
                    [10, 76],
                    [11, 64],
                    [12, 90],
                    [13, 64],
                    [14, 78],
                    [15, 68]
                ]
            },
            {
                notes: [46],
                hits: [
                    [6, 88],
                    [14, 94]
                ]
            }
        ]
    },
    {
        value: 'uk-garage',
        label: 'UK garage',
        description: 'Shuffled two-step kick, clap and syncopated hats',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 122],
                    [6, 102],
                    [11, 114]
                ]
            },
            {
                notes: [38, 39, 40],
                preferredNote: 39,
                hits: [
                    [4, 118],
                    [12, 122]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 78],
                    [3, 70],
                    [6, 82],
                    [8, 76],
                    [11, 74],
                    [14, 86]
                ]
            },
            {
                notes: [46],
                hits: [
                    [7, 92],
                    [15, 96]
                ]
            }
        ]
    },
    {
        value: 'reggaeton',
        label: 'Reggaeton dembow',
        description: 'Dembow kick and snare pattern with closed hi-hat support',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 124],
                    [7, 112],
                    [8, 118]
                ]
            },
            {
                notes: [38, 39, 40],
                hits: [
                    [4, 120],
                    [10, 110],
                    [12, 124]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 74],
                    [2, 68],
                    [4, 80],
                    [6, 68],
                    [8, 74],
                    [10, 70],
                    [12, 82],
                    [14, 70]
                ]
            }
        ]
    },
    {
        value: 'bossa',
        label: 'Bossa nova',
        description: 'Soft kick, side-stick clave feel and closed hi-hat pulse',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 96],
                    [6, 78],
                    [8, 92],
                    [14, 76]
                ]
            },
            {
                notes: [37, 38],
                hits: [
                    [0, 78],
                    [3, 72],
                    [6, 80],
                    [10, 76],
                    [13, 82]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 66],
                    [2, 62],
                    [4, 68],
                    [6, 62],
                    [8, 66],
                    [10, 62],
                    [12, 70],
                    [14, 64]
                ]
            }
        ]
    },
    {
        value: 'samba',
        label: 'Samba percussion',
        description: 'Conga, timbale, cowbell and maracas ensemble',
        voices: [
            {
                notes: [63, 64],
                hits: [
                    [0, 104],
                    [3, 82],
                    [6, 96],
                    [8, 106],
                    [11, 84],
                    [14, 98]
                ]
            },
            {
                notes: [65, 66],
                hits: [
                    [2, 90],
                    [5, 82],
                    [10, 92],
                    [13, 84]
                ]
            },
            {
                notes: [56],
                hits: [
                    [0, 92],
                    [4, 88],
                    [8, 94],
                    [12, 90]
                ]
            },
            {
                notes: [70, 82],
                hits: [
                    [0, 70],
                    [2, 66],
                    [4, 72],
                    [6, 66],
                    [8, 70],
                    [10, 68],
                    [12, 74],
                    [14, 68]
                ]
            }
        ]
    },
    {
        value: 'afrobeat',
        label: 'Afrobeat',
        description: 'Layered kick, cross-stick, hats and cowbell syncopation',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 118],
                    [3, 92],
                    [8, 112],
                    [11, 96],
                    [14, 104]
                ]
            },
            {
                notes: [37, 38],
                hits: [
                    [4, 110],
                    [7, 76],
                    [12, 114],
                    [15, 78]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [0, 78],
                    [2, 68],
                    [4, 82],
                    [6, 70],
                    [8, 78],
                    [10, 72],
                    [12, 84],
                    [14, 72]
                ]
            },
            {
                notes: [56],
                hits: [
                    [1, 84],
                    [5, 88],
                    [9, 84],
                    [13, 90]
                ]
            }
        ]
    },
    {
        value: 'minimal',
        label: 'Minimal pulse',
        description: 'Sparse kick, rim and hi-hat pattern with plenty of space',
        voices: [
            {
                notes: [35, 36],
                hits: [
                    [0, 118],
                    [10, 102]
                ]
            },
            {
                notes: [37, 38, 40],
                preferredNote: 37,
                hits: [
                    [6, 94],
                    [14, 100]
                ]
            },
            {
                notes: [42, 44],
                hits: [
                    [2, 68],
                    [6, 74],
                    [10, 70],
                    [14, 78]
                ]
            }
        ]
    }
];
