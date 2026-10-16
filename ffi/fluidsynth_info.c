#include <string.h>

#include "fluid_defsfont.h"

/* This bridge is intentionally specific to FluidSynth's default SF2 loader.
 * It exposes the sample-backed voice ranges which the loader has already
 * computed by intersecting preset and instrument zones. */
int aaw_fluid_preset_note_mask(fluid_preset_t *preset, unsigned char notes[128])
{
    fluid_defpreset_t *defpreset;
    fluid_preset_zone_t *preset_zone;

    if(preset == NULL || notes == NULL)
    {
        return -1;
    }

    defpreset = fluid_preset_get_data(preset);
    if(defpreset == NULL)
    {
        return -1;
    }

    memset(notes, 0, 128);

    for(preset_zone = defpreset->zone; preset_zone != NULL; preset_zone = preset_zone->next)
    {
        fluid_list_t *item;

        for(item = preset_zone->voice_zone; item != NULL; item = item->next)
        {
            fluid_voice_zone_t *voice_zone = item->data;
            int first;
            int last;
            int key;

            if(voice_zone == NULL || voice_zone->range.vello > voice_zone->range.velhi)
            {
                continue;
            }

            first = voice_zone->range.keylo < 0 ? 0 : voice_zone->range.keylo;
            last = voice_zone->range.keyhi > 127 ? 127 : voice_zone->range.keyhi;

            for(key = first; key <= last; ++key)
            {
                notes[key] = 1;
            }
        }
    }

    return 0;
}
