<script lang="ts">
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import { onDestroy, onMount, tick } from 'svelte';
    import * as api from '$lib/generated/api';
    import type {
        DrumLane,
        DrumLaneSequence,
        DrumMachineController,
        DrumStep
    } from '$lib/generated/models';
    import Button from '$lib/components/Button.svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import ProjectTransport from '$lib/components/ProjectTransport.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import { errorToUserMessage } from '$lib/error';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import { audioOutputStream } from '$lib/state/audio.svelte';
    import ControllerEditor from '../instruments/+page.svelte';

    type Machine = { id: number; data: DrumMachineController; savedName: string };
    type LanePreset = {
        value: string;
        label: string;
        description: string;
        lanes: ReadonlyArray<readonly [name: string, note: number]>;
    };
    type BeatPreset = {
        value: string;
        label: string;
        description: string;
        voices: ReadonlyArray<{
            notes: readonly number[];
            preferredNote?: number;
            hits: ReadonlyArray<readonly [step: number, velocity: number]>;
        }>;
    };
    const drumVoiceNames: Readonly<Record<number, string>> = {
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
    type SelectedStep = { machineId: number; laneId: number; index: number };
    type SelectedVoice = {
        machineId: number;
        laneId: number;
        initialName: string;
        initialNote: number;
        initialChannel: number;
        initialVelocity: number;
    };
    const lanePresets: LanePreset[] = [
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
    const beatPresets: BeatPreset[] = [
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
    let requestedInstrument = $derived(page.url.searchParams.get('synth'));
    let instrumentId = $derived(
        requestedInstrument === null ? Number.NaN : Number(requestedInstrument)
    );
    let machines = $state<Machine[]>([]);
    let loading = $state(true);
    let adding = $state(false);
    let addingMidiController = $state(false);
    let controllerEditorRevision = $state(0);
    let saving = $state(new Set<number>());
    let dirtySaves = $state(new Set<number>());
    let removing = $state(new Set<number>());
    let selectedLanePresets = $state<Record<number, string>>({});
    let selectedBeatPresets = $state<Record<number, string>>({});
    let selectedStep = $state<SelectedStep | null>(null);
    let selectedVoice = $state<SelectedVoice | null>(null);
    let voiceEditor: HTMLDivElement | undefined = $state();
    let voiceEditorHighlighted = $state(false);
    let voiceHighlightTimeout: ReturnType<typeof setTimeout> | undefined;
    let previewFrame: number | undefined;
    let positionSyncInterval: ReturnType<typeof setInterval> | undefined;
    let positionSyncing = false;
    let positionAnchorTick = 0;
    let positionAnchorTime = 0;
    let positionSynced = false;
    let wasPlaying = false;
    let previewPlaying = $state(false);
    let previewTick = $state(0);
    let meterBeatSteps = $derived(16 / (activeProject()?.transport.beat_unit ?? 4));
    let previewQuantum = $derived(
        Math.max(
            1,
            Math.min(
                ...machines.flatMap(({ data }) =>
                    data.sequences.map(({ step_resolution_ticks }) => step_resolution_ticks)
                ),
                960
            )
        )
    );

    onMount(() => {
        void load();
        positionAnchorTime = performance.now();
        updatePreview();
        positionSyncInterval = setInterval(syncTransportPosition, 500);
    });
    onDestroy(() => {
        clearTimeout(voiceHighlightTimeout);
        clearInterval(positionSyncInterval);
        if (previewFrame !== undefined) cancelAnimationFrame(previewFrame);
    });

    async function syncTransportPosition() {
        if (!activeProject()?.transport.playing || positionSyncing) return;
        positionSyncing = true;
        const requestStarted = performance.now();
        try {
            const response = await api.transportPosition();
            const responseReceived = performance.now();
            if (response.status === 200) {
                positionAnchorTick = response.data;
                positionAnchorTime = (requestStarted + responseReceived) / 2;
                positionSynced = true;
            }
        } catch {
            // Retry on the next interval without showing an unsynchronized playhead.
        } finally {
            positionSyncing = false;
        }
    }

    function updatePreview() {
        const transport = activeProject()?.transport;
        const now = performance.now();
        if (transport?.playing) {
            if (!wasPlaying) {
                positionAnchorTick = 0;
                positionAnchorTime = now;
                positionSynced = false;
                void syncTransportPosition();
            }
            const elapsedMinutes = (now - positionAnchorTime) / 60_000;
            const output = audioOutputStream();
            const outputLatencyTicks = output
                ? (output.buffer_size / output.sample_rate / 60) * transport.tempo_bpm * 960
                : 0;
            const estimatedTick = Math.max(
                0,
                positionAnchorTick + elapsedMinutes * transport.tempo_bpm * 960 - outputLatencyTicks
            );
            const quantizedTick = Math.floor(estimatedTick / previewQuantum) * previewQuantum;
            if (previewTick !== quantizedTick) previewTick = quantizedTick;
            previewPlaying = positionSynced;
        } else {
            previewPlaying = false;
            previewTick = 0;
            positionSynced = false;
        }
        wasPlaying = transport?.playing ?? false;
        previewFrame = requestAnimationFrame(updatePreview);
    }

    function previewed(sequence: DrumLaneSequence | undefined, index: number) {
        if (!previewPlaying || !sequence || sequence.length_steps === 0) return false;
        return (
            Math.floor(previewTick / sequence.step_resolution_ticks) % sequence.length_steps ===
            index
        );
    }

    async function load() {
        if (!Number.isInteger(instrumentId) || instrumentId < 0) {
            loading = false;
            return;
        }
        loading = true;
        try {
            const controllers = await api.controllerList(instrumentId);
            machines = controllers.data.flatMap(([id, controller]) =>
                controller.type === 'DrumMachine'
                    ? [{ id, data: controller.data, savedName: controller.data.name }]
                    : []
            );
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Load drum machines' });
        } finally {
            loading = false;
        }
    }

    async function addMachine() {
        adding = true;
        try {
            await api.drumMachineAppend(instrumentId);
            await load();
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Add drum machine' });
        } finally {
            adding = false;
        }
    }

    async function addMidiController() {
        addingMidiController = true;
        try {
            await api.controllerAppend(instrumentId);
            controllerEditorRevision += 1;
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Add MIDI input controller' });
        } finally {
            addingMidiController = false;
        }
    }

    async function saveMachine(machine: Machine): Promise<boolean> {
        if (saving.has(machine.id)) {
            dirtySaves = new Set(dirtySaves).add(machine.id);
            return true;
        }
        saving = new Set(saving).add(machine.id);
        try {
            do {
                const nextDirty = new Set(dirtySaves);
                nextDirty.delete(machine.id);
                dirtySaves = nextDirty;
                const savedName = machine.data.name;
                const response = await api.controllerReplace(instrumentId, machine.id, {
                    type: 'DrumMachine',
                    data: machine.data
                });
                if (response.status !== 204) {
                    throw new Error(`Update failed (${response.status})`);
                }
                machine.savedName = savedName;
            } while (dirtySaves.has(machine.id));
            return true;
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update drum machine' });
            await load();
            return false;
        } finally {
            const next = new Set(saving);
            next.delete(machine.id);
            saving = next;
        }
    }

    async function removeMachine(machine: Machine) {
        removing = new Set(removing).add(machine.id);
        try {
            await api.controllerRemove(instrumentId, machine.id);
            machines = machines.filter(({ id }) => id !== machine.id);
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Remove drum machine' });
        } finally {
            const next = new Set(removing);
            next.delete(machine.id);
            removing = next;
        }
    }

    async function addLane(machine: Machine) {
        if (machine.data.lanes.length >= 64) return;
        const usedIds = new Set(machine.data.lanes.map(({ id }) => id));
        let id = 0;
        while (usedIds.has(id)) id += 1;

        const resolution = machine.data.sequences[0]?.step_resolution_ticks ?? 240;
        const transport = activeProject()?.transport;
        const barTicks = transport
            ? ((960 * 4) / transport.beat_unit) * transport.beats_per_bar
            : 3840;
        const lengthSteps = Math.max(1, Math.floor(barTicks / resolution));
        const lane: DrumLane = {
            id,
            name: `Lane ${machine.data.lanes.length + 1}`,
            output_note: 36,
            output_channel: 9,
            velocity: 127,
            gate_mode: 'OneShot',
            choke_group: null,
            muted: false,
            solo: false
        };
        machine.data.lanes.push(lane);
        machine.data.sequences.push({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: Array.from({ length: lengthSteps }, () => ({
                enabled: false,
                velocity: 100,
                micro_offset_ticks: 0,
                gate_length_ticks: Math.max(1, Math.floor(resolution / 2))
            }))
        });
        if (await saveMachine(machine)) await selectVoice(machine, lane);
    }

    async function removeLane(machine: Machine, lane: DrumLane) {
        machine.data.lanes = machine.data.lanes.filter(({ id }) => id !== lane.id);
        machine.data.sequences = machine.data.sequences.filter(
            ({ lane_id }) => lane_id !== lane.id
        );
        if (selectedVoice?.machineId === machine.id && selectedVoice.laneId === lane.id) {
            selectedVoice = null;
        }
        if (selectedStep?.machineId === machine.id && selectedStep.laneId === lane.id) {
            selectedStep = null;
        }
        await saveMachine(machine);
    }

    async function removeAllLanes(machine: Machine) {
        machine.data.lanes = [];
        machine.data.sequences = [];
        if (selectedVoice?.machineId === machine.id) selectedVoice = null;
        if (selectedStep?.machineId === machine.id) selectedStep = null;
        await saveMachine(machine);
    }

    async function applyLanePreset(machine: Machine) {
        const preset = lanePresets.find(({ value }) => value === selectedLanePresets[machine.id]);
        if (!preset) return;

        const resolution = machine.data.sequences[0]?.step_resolution_ticks ?? 240;
        const transport = activeProject()?.transport;
        const barTicks = transport
            ? ((960 * 4) / transport.beat_unit) * transport.beats_per_bar
            : 3840;
        const lengthSteps = Math.max(1, Math.floor(barTicks / resolution));
        machine.data.lanes = preset.lanes.map(([name, output_note], id) => ({
            id,
            name,
            output_note,
            output_channel: 9,
            velocity: 127,
            gate_mode: 'OneShot',
            choke_group: null,
            muted: false,
            solo: false
        }));
        machine.data.sequences = machine.data.lanes.map(({ id }) => ({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: Array.from({ length: lengthSteps }, () => ({
                enabled: false,
                velocity: 100,
                micro_offset_ticks: 0,
                gate_length_ticks: Math.max(1, Math.floor(resolution / 2))
            }))
        }));
        if (selectedVoice?.machineId === machine.id) selectedVoice = null;
        if (selectedStep?.machineId === machine.id) selectedStep = null;
        if (await saveMachine(machine)) {
            selectedLanePresets = { ...selectedLanePresets, [machine.id]: '' };
        }
    }

    async function applyBeatPreset(machine: Machine) {
        const preset = beatPresets.find(({ value }) => value === selectedBeatPresets[machine.id]);
        if (!preset) return;

        const resolution = machine.data.sequences[0]?.step_resolution_ticks ?? 240;
        const transport = activeProject()?.transport;
        const barTicks = transport
            ? ((960 * 4) / transport.beat_unit) * transport.beats_per_bar
            : 3840;
        const lengthSteps = Math.max(1, Math.floor(barTicks / resolution));
        machine.data.lanes = preset.voices.map((voice, id) => {
            const output_note =
                voice.preferredNote ?? (voice.notes.includes(36) ? 36 : voice.notes[0]);
            return {
                id,
                name: drumVoiceNames[output_note] ?? `Note ${output_note}`,
                output_note,
                output_channel: 9,
                velocity: 127,
                gate_mode: 'OneShot',
                choke_group: null,
                muted: false,
                solo: false
            };
        });
        machine.data.sequences = machine.data.lanes.map(({ id }) => ({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: Array.from({ length: lengthSteps }, () => ({
                enabled: false,
                velocity: 100,
                micro_offset_ticks: 0,
                gate_length_ticks: Math.max(1, Math.floor(resolution / 2))
            }))
        }));

        const presetCycleTicks = 960 * 4;
        const presetStepTicks = 960 / 4;
        for (const [voiceIndex, voice] of preset.voices.entries()) {
            const laneSequence = machine.data.sequences[voiceIndex];
            for (
                let cycleStartTick = 0;
                cycleStartTick < barTicks;
                cycleStartTick += presetCycleTicks
            ) {
                for (const [presetStep, velocity] of voice.hits) {
                    const hitTick = cycleStartTick + presetStep * presetStepTicks;
                    if (hitTick >= barTicks) {
                        continue;
                    }

                    const index = Math.round(hitTick / resolution);
                    const step = laneSequence.steps[index];
                    if (step && index < laneSequence.length_steps) {
                        step.velocity = step.enabled ? Math.max(step.velocity, velocity) : velocity;
                        step.enabled = true;
                    }
                }
            }
        }
        selectedStep = null;
        if (selectedVoice?.machineId === machine.id) selectedVoice = null;
        if (await saveMachine(machine)) {
            selectedBeatPresets = { ...selectedBeatPresets, [machine.id]: '' };
            selectedLanePresets = { ...selectedLanePresets, [machine.id]: '' };
        }
    }

    function sequence(machine: Machine, lane: DrumLane) {
        return machine.data.sequences.find(({ lane_id }) => lane_id === lane.id);
    }
    function selected(machine: Machine, lane: DrumLane, index: number) {
        return (
            selectedStep?.machineId === machine.id &&
            selectedStep.laneId === lane.id &&
            selectedStep.index === index
        );
    }
    function selectedStepValue(machine: Machine): DrumStep | undefined {
        if (selectedStep?.machineId !== machine.id) return;
        const lane = machine.data.lanes.find(({ id }) => id === selectedStep?.laneId);
        return lane ? sequence(machine, lane)?.steps[selectedStep.index] : undefined;
    }
    function selectedLane(machine: Machine): DrumLane | undefined {
        if (selectedStep?.machineId !== machine.id) return;
        return machine.data.lanes.find(({ id }) => id === selectedStep?.laneId);
    }
    function selectedVoiceLane(machine: Machine): DrumLane | undefined {
        if (selectedVoice?.machineId !== machine.id) return;
        return machine.data.lanes.find(({ id }) => id === selectedVoice?.laneId);
    }
    async function selectVoice(machine: Machine, lane: DrumLane) {
        clearTimeout(voiceHighlightTimeout);
        voiceEditorHighlighted = false;
        selectedVoice = {
            machineId: machine.id,
            laneId: lane.id,
            initialName: lane.name,
            initialNote: lane.output_note,
            initialChannel: lane.output_channel + 1,
            initialVelocity: lane.velocity
        };
        await tick();
        voiceEditor?.scrollIntoView({
            behavior: 'smooth',
            block: 'nearest',
            inline: 'nearest'
        });
        voiceEditor?.querySelector<HTMLInputElement>('input')?.focus({ preventScroll: true });
        voiceEditorHighlighted = true;
        voiceHighlightTimeout = setTimeout(() => {
            voiceEditorHighlighted = false;
        }, 900);
    }
    async function toggleStep(machine: Machine, lane: DrumLane, index: number) {
        const step = sequence(machine, lane)?.steps[index];
        if (!step) return;
        step.enabled = !step.enabled;
        selectedStep = { machineId: machine.id, laneId: lane.id, index };
        await saveMachine(machine);
    }
    function laneHasSteps(machine: Machine, lane: DrumLane) {
        return sequence(machine, lane)?.steps.some(({ enabled }) => enabled) ?? false;
    }
    function machineHasSteps(machine: Machine) {
        return machine.data.sequences.some((laneSequence) =>
            laneSequence.steps.some(({ enabled }) => enabled)
        );
    }
    async function clearLane(machine: Machine, lane: DrumLane) {
        const laneSequence = sequence(machine, lane);
        if (!laneSequence) return;
        for (const step of laneSequence.steps) step.enabled = false;
        await saveMachine(machine);
    }
    async function clearMachine(machine: Machine) {
        for (const laneSequence of machine.data.sequences) {
            for (const step of laneSequence.steps) step.enabled = false;
        }
        await saveMachine(machine);
    }
</script>

<div class="grid min-w-0 gap-5 overflow-x-clip">
    <header class="flex min-w-0 items-center gap-3">
        <a
            class="border-border bg-surface-button text-foreground-muted hover:text-accent grid size-9 shrink-0 place-items-center rounded-lg border transition-colors"
            href={`/synths?synth=${instrumentId}`}
            aria-label="Back to instrument">
            <Icon icon="lucide:arrow-left" width="1rem" />
        </a>
        <div class="min-w-0 flex-1">
            <h2 class="text-foreground m-0 truncate text-base font-semibold">Controllers</h2>
            <p class="text-foreground-muted m-0 text-sm">MIDI input and sample-clock sequencers</p>
        </div>
    </header>

    {#if Number.isInteger(instrumentId) && instrumentId >= 0}
        <ProjectTransport onchange={load} />

        <section class="grid min-w-0 gap-4">
            <div>
                <h3 class="text-foreground m-0 text-sm font-semibold">Drum machines</h3>
                <p class="text-foreground-muted m-0 text-xs">One-bar sequencers</p>
            </div>
            {#if loading}<p class="text-foreground-muted m-0 text-sm">Loading drum machines…</p>
            {:else if machines.length === 0}<div
                    class="border-border text-foreground-muted rounded-xl border border-dashed p-5 text-center text-sm">
                    Add a drum machine to sequence this instrument.
                </div>{/if}

            {#each machines as machine (machine.id)}
                <article
                    class="border-border bg-surface-panel grid min-w-0 gap-3 rounded-xl border p-3">
                    <div class="flex flex-wrap items-end gap-2">
                        <div class="grid min-w-48 flex-1 gap-1">
                            <label
                                class="text-foreground-muted text-xs"
                                for={`drum-machine-${machine.id}-name`}>
                                Name
                            </label>
                            <AutoSaveTextInput
                                id={`drum-machine-${machine.id}-name`}
                                bind:value={machine.data.name}
                                pending={machine.data.name !== machine.savedName}
                                saving={saving.has(machine.id)}
                                oncommit={() => saveMachine(machine)} />
                        </div>
                        <Button
                            size="sm"
                            variant="accent-outline"
                            disabled={machine.data.lanes.length >= 64 || saving.has(machine.id)}
                            onclick={() => addLane(machine)}>
                            <Icon icon="lucide:plus" width="0.875rem" /> Lane
                        </Button>
                        <TimedConfirmButton
                            label="Delete all"
                            confirmLabel="Confirm delete"
                            busyLabel="Deleting"
                            busy={saving.has(machine.id)}
                            disabled={machine.data.lanes.length === 0}
                            onconfirm={() => removeAllLanes(machine)} />
                        <TimedConfirmButton
                            label="Clear all"
                            confirmLabel="Confirm clear"
                            busyLabel="Clearing"
                            disabled={saving.has(machine.id) || !machineHasSteps(machine)}
                            onconfirm={() => clearMachine(machine)} />
                        <Button
                            size="sm"
                            variant={machine.data.enabled ? 'accent-outline' : 'secondary'}
                            onclick={() => {
                                machine.data.enabled = !machine.data.enabled;
                                return saveMachine(machine);
                            }}>
                            {machine.data.enabled ? 'Enabled' : 'Disabled'}
                        </Button>
                        <TimedConfirmButton
                            label="Remove"
                            confirmLabel="Confirm remove"
                            busyLabel="Removing"
                            busy={removing.has(machine.id)}
                            onconfirm={() => removeMachine(machine)} />
                    </div>

                    <div class="grid gap-3 min-[37.5rem]:grid-cols-2">
                        <div class="flex min-w-0 items-end gap-2">
                            <label class="grid min-w-0 flex-1 gap-1">
                                <span class="text-foreground-muted text-xs">Lane preset</span>
                                <SelectDropdown
                                    value={selectedLanePresets[machine.id] ?? ''}
                                    options={lanePresets.map(({ value, label, description }) => ({
                                        value,
                                        label,
                                        description
                                    }))}
                                    placeholder="Choose a drum layout"
                                    disabled={saving.has(machine.id)}
                                    ariaLabel="Drum-machine lane preset"
                                    onchange={(value) => {
                                        selectedLanePresets = {
                                            ...selectedLanePresets,
                                            [machine.id]: value
                                        };
                                    }} />
                            </label>
                            <TimedConfirmButton
                                label="Apply"
                                confirmLabel="Replace lanes?"
                                busyLabel="Applying"
                                busy={saving.has(machine.id)}
                                disabled={!selectedLanePresets[machine.id]}
                                onconfirm={() => applyLanePreset(machine)} />
                        </div>
                        <div class="flex min-w-0 items-end gap-2">
                            <label class="grid min-w-0 flex-1 gap-1">
                                <span class="text-foreground-muted text-xs">Beat preset</span>
                                <SelectDropdown
                                    value={selectedBeatPresets[machine.id] ?? ''}
                                    options={beatPresets.map(({ value, label, description }) => ({
                                        value,
                                        label,
                                        description
                                    }))}
                                    placeholder="Choose a predefined beat"
                                    disabled={saving.has(machine.id)}
                                    searchable
                                    searchPlaceholder="Search beat presets"
                                    ariaLabel="Drum-machine beat preset"
                                    onchange={(value) => {
                                        selectedBeatPresets = {
                                            ...selectedBeatPresets,
                                            [machine.id]: value
                                        };
                                    }} />
                            </label>
                            <TimedConfirmButton
                                label="Apply"
                                confirmLabel="Replace lanes and beat?"
                                busyLabel="Applying"
                                busy={saving.has(machine.id)}
                                disabled={!selectedBeatPresets[machine.id]}
                                onconfirm={() => applyBeatPreset(machine)} />
                        </div>
                    </div>

                    <div
                        class="w-full max-w-full min-w-0 overflow-x-auto overscroll-x-contain pb-1">
                        <div class="grid gap-1.5">
                            {#each machine.data.lanes as lane (lane.id)}
                                {@const laneSequence = sequence(machine, lane)}
                                <div
                                    class="grid items-center gap-1"
                                    style:grid-template-columns={`15rem repeat(${laneSequence?.length_steps ?? 0}, minmax(1.75rem, 1fr))`}
                                    style:min-width={`${15 + (laneSequence?.length_steps ?? 0) * 2}rem`}>
                                    <div class="flex min-w-0 items-center gap-1">
                                        <button
                                            class="text-foreground hover:text-accent min-w-0 flex-1 cursor-pointer truncate text-left text-xs"
                                            onclick={() => selectVoice(machine, lane)}
                                            title="Edit voice routing">
                                            {lane.name}
                                        </button>
                                        <button
                                            class={`border-border text-foreground-muted hover:border-accent/60 hover:text-accent grid size-6 cursor-pointer place-items-center rounded border ${selectedVoice?.machineId === machine.id && selectedVoice.laneId === lane.id ? 'border-accent text-accent' : ''}`}
                                            onclick={() => selectVoice(machine, lane)}
                                            aria-label={`Edit ${lane.name}`}>
                                            <Icon icon="lucide:pencil" width="0.75rem" />
                                        </button>
                                        <button
                                            class={`size-6 cursor-pointer rounded border text-[0.625rem] transition-colors ${lane.muted ? 'border-danger bg-danger/25 text-danger-foreground ring-danger/40 ring-1' : 'border-border text-foreground-muted hover:border-danger/60 hover:text-danger-foreground'}`}
                                            onclick={() => {
                                                lane.muted = !lane.muted;
                                                return saveMachine(machine);
                                            }}
                                            aria-pressed={lane.muted}
                                            aria-label={`Mute ${lane.name}`}>
                                            M
                                        </button>
                                        <button
                                            class={`size-6 cursor-pointer rounded border text-[0.625rem] transition-colors ${lane.solo ? 'border-accent bg-accent/25 text-accent ring-accent/40 ring-1' : 'border-border text-foreground-muted hover:border-accent/60 hover:text-accent'}`}
                                            onclick={() => {
                                                lane.solo = !lane.solo;
                                                return saveMachine(machine);
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
                                            disabled={saving.has(machine.id) ||
                                                !laneHasSteps(machine, lane)}
                                            onconfirm={() => clearLane(machine, lane)} />
                                    </div>
                                    {#each laneSequence?.steps.slice(0, laneSequence.length_steps) ?? [] as step, index}
                                        <button
                                            class={`border-border hover:border-accent/60 aspect-square cursor-pointer rounded border transition-colors ${step.enabled ? 'bg-accent' : 'bg-surface-input'} ${selected(machine, lane, index) ? 'ring-accent/40 ring-2' : ''} ${previewed(laneSequence, index) ? 'outline-accent outline-2 -outline-offset-3' : ''} ${index % meterBeatSteps === 0 ? 'border-l-accent border-l-2' : ''}`}
                                            style:opacity={step.enabled
                                                ? Math.max(0.35, step.velocity / 127)
                                                : 1}
                                            onclick={() => toggleStep(machine, lane, index)}
                                            aria-current={previewed(laneSequence, index)
                                                ? 'step'
                                                : undefined}
                                            aria-label={`${lane.name} step ${index + 1}`}>
                                        </button>
                                    {/each}
                                </div>
                            {/each}
                        </div>
                    </div>

                    {#if selectedVoiceLane(machine)}
                        {@const lane = selectedVoiceLane(machine)!}
                        <div
                            bind:this={voiceEditor}
                            class={`border-border grid gap-3 rounded-lg border-t pt-3 transition-[background-color,box-shadow] min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-3 ${voiceEditorHighlighted ? 'bg-accent/10 ring-accent/50 animate-[pulse_900ms_ease-out_1] ring-2' : ''}`}>
                            <div class="grid content-start gap-1">
                                <label
                                    class="text-foreground-muted text-sm"
                                    for={`drum-machine-${machine.id}-lane-${lane.id}-name`}>
                                    Lane name
                                </label>
                                <AutoSaveTextInput
                                    id={`drum-machine-${machine.id}-lane-${lane.id}-name`}
                                    bind:value={lane.name}
                                    pending={lane.name !== selectedVoice?.initialName}
                                    saving={saving.has(machine.id)}
                                    oncommit={async () => {
                                        if (
                                            (await saveMachine(machine)) &&
                                            selectedVoice?.machineId === machine.id &&
                                            selectedVoice.laneId === lane.id
                                        ) {
                                            selectedVoice.initialName = lane.name;
                                        }
                                    }} />
                            </div>
                            <div>
                                <NumericControl
                                    compact
                                    label={`${lane.name} MIDI note`}
                                    value={lane.output_note}
                                    unit="note"
                                    smallStep={1}
                                    largeStep={12}
                                    resetValue={selectedVoice?.initialNote ?? lane.output_note}
                                    min={0}
                                    max={127}
                                    onchange={async (value) => {
                                        lane.output_note = value;
                                        await saveMachine(machine);
                                        return lane.output_note;
                                    }} />
                            </div>
                            <NumericControl
                                compact
                                label={`${lane.name} MIDI channel`}
                                value={lane.output_channel + 1}
                                unit="channel"
                                smallStep={1}
                                largeStep={4}
                                resetValue={selectedVoice?.initialChannel ??
                                    lane.output_channel + 1}
                                min={1}
                                max={16}
                                onchange={async (value) => {
                                    lane.output_channel = value - 1;
                                    await saveMachine(machine);
                                    return lane.output_channel + 1;
                                }} />
                            <div>
                                <NumericControl
                                    compact
                                    label={`${lane.name} velocity`}
                                    value={lane.velocity}
                                    unit="MIDI"
                                    smallStep={1}
                                    largeStep={10}
                                    resetValue={selectedVoice?.initialVelocity ?? lane.velocity}
                                    min={1}
                                    max={127}
                                    onchange={async (value) => {
                                        lane.velocity = value;
                                        await saveMachine(machine);
                                        return lane.velocity;
                                    }} />
                            </div>
                            <div
                                class="flex items-end justify-end min-[37.5rem]:col-span-2 min-[56.25rem]:col-span-3">
                                <TimedConfirmButton
                                    label="Remove lane"
                                    confirmLabel="Confirm remove"
                                    busyLabel="Removing"
                                    busy={saving.has(machine.id)}
                                    onconfirm={() => removeLane(machine, lane)} />
                            </div>
                        </div>
                    {/if}

                    {#if selectedStepValue(machine) && selectedStep?.machineId === machine.id}
                        {@const step = selectedStepValue(machine)!}
                        {@const lane = selectedLane(machine)!}
                        <div class="border-border flex flex-wrap items-end gap-3 border-t pt-3">
                            <span class="text-foreground-muted mr-auto text-xs">
                                {lane.name} · Step {selectedStep.index + 1}
                            </span>
                            <div class="grid w-full gap-3 min-[37.5rem]:grid-cols-2">
                                <NumericControl
                                    compact
                                    label="Step velocity"
                                    value={step.velocity}
                                    unit="MIDI"
                                    smallStep={1}
                                    largeStep={10}
                                    resetValue={100}
                                    min={1}
                                    max={127}
                                    onchange={async (value) => {
                                        step.velocity = value;
                                        await saveMachine(machine);
                                        return step.velocity;
                                    }} />
                                <NumericControl
                                    compact
                                    label="Gate length"
                                    value={step.gate_length_ticks}
                                    unit="ticks"
                                    smallStep={10}
                                    largeStep={120}
                                    resetValue={120}
                                    min={1}
                                    max={960}
                                    onchange={async (value) => {
                                        step.gate_length_ticks = value;
                                        await saveMachine(machine);
                                        return step.gate_length_ticks;
                                    }} />
                            </div>
                        </div>
                    {/if}
                    {#if saving.has(machine.id)}<p
                            class="text-foreground-muted m-0 text-right text-xs">
                            Saving…
                        </p>{/if}
                </article>
            {/each}
            <Button
                class="justify-self-end"
                variant="accent-outline"
                busy={adding}
                onclick={addMachine}>
                <Icon icon="lucide:plus" width="1rem" /> Add
            </Button>
        </section>

        <section class="grid gap-3">
            <div>
                <h3 class="text-foreground m-0 text-sm font-semibold">MIDI input controllers</h3>
                <p class="text-foreground-muted m-0 text-xs">Live MIDI routing and transforms</p>
            </div>
            {#key controllerEditorRevision}
                <ControllerEditor embedded {instrumentId} showAddButton={false} />
            {/key}
            <Button
                class="justify-self-end"
                variant="accent-outline"
                busy={addingMidiController}
                onclick={addMidiController}>
                <Icon icon="lucide:plus" width="1rem" /> Add
            </Button>
        </section>
    {:else}<p class="text-danger m-0 text-sm">Select an instrument first.</p>{/if}
</div>
