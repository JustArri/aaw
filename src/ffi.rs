//! Raw, generated native synthesizer bindings.

/// Raw FluidSynth C bindings.
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod fluidsynth {
    include!(concat!(env!("OUT_DIR"), "/fluidsynth_bindings.rs"));
}

/// Raw sfizz C bindings.
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod sfizz {
    include!(concat!(env!("OUT_DIR"), "/sfizz_bindings.rs"));
}
