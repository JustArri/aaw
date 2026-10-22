import activity from '@iconify-icons/lucide/activity';
import archive from '@iconify-icons/lucide/archive';
import audio from '@iconify-icons/lucide/audio-lines';
import chevron from '@iconify-icons/lucide/chevron-right';
import folder from '@iconify-icons/lucide/folder';
import grid from '@iconify-icons/lucide/grid-2x2';
import more from '@iconify-icons/lucide/ellipsis';
import midi from '@iconify-icons/lucide/piano';
import plus from '@iconify-icons/lucide/plus';
import power from '@iconify-icons/lucide/power';
import save from '@iconify-icons/lucide/save';
import settings from '@iconify-icons/lucide/settings';
import sliders from '@iconify-icons/lucide/sliders-horizontal';
import x from '@iconify-icons/lucide/x';

export const icons = {
    assets: archive,
    audio,
    chevron,
    folder,
    grid,
    live: activity,
    midi,
    more,
    plus,
    power,
    save,
    settings,
    sliders,
    x
};

export type IconName = keyof typeof icons;
