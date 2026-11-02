#define FLUIDSYNTH_API
#define FLUID_DEPRECATED
#include <stddef.h>
#include <fluidsynth/types.h>
#include <fluidsynth/settings.h>
#include <fluidsynth/sfont.h>
#include <fluidsynth/synth.h>

int aaw_fluid_preset_note_mask(fluid_preset_t *preset, unsigned char notes[128]);
