use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    assert_submodule("lib/fluidsynth/CMakeLists.txt");
    assert_submodule("lib/fluidsynth/gcem/CMakeLists.txt");
    assert_submodule("lib/sfizz/CMakeLists.txt");
    assert_submodule("lib/sfizz/external/abseil-cpp/CMakeLists.txt");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ffi/fluidsynth.h");
    println!("cargo:rerun-if-changed=ffi/sfizz.h");
    println!("cargo:rerun-if-changed=lib/fluidsynth/CMakeLists.txt");
    println!("cargo:rerun-if-changed=lib/sfizz/CMakeLists.txt");

    let fluid = build_fluidsynth();
    let sfizz = build_sfizz();

    build_fluidsynth_info(&fluid);
    link_static_library(&fluid, "fluidsynth");
    link_sfizz_static_libraries(&sfizz);
    link_cpp_runtime();
    generate_bindings();
}

fn build_fluidsynth_info(fluid: &Path) {
    cc::Build::new()
        .file("ffi/fluidsynth_info.c")
        .include("lib/fluidsynth/include")
        .include("lib/fluidsynth/include/fluidsynth")
        .include("lib/fluidsynth/src")
        .include("lib/fluidsynth/src/sfloader")
        .include("lib/fluidsynth/src/synth")
        .include("lib/fluidsynth/src/utils")
        .include(fluid.join("build"))
        .include(fluid.join("build/include"))
        .warnings(false)
        .compile("aaw_fluidsynth_info");
}

fn assert_submodule(file: &str) {
    if !Path::new(file).is_file() {
        panic!(
            "missing {file}; initialize native dependencies with \
             `git submodule update --init --recursive`"
        );
    }
}

fn common_config(source: &str, output_name: &str) -> cmake::Config {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"))
        .join("native")
        .join(output_name);
    let library_dir = output.join("lib");
    let mut config = cmake::Config::new(source);
    config
        .out_dir(&output)
        .define("CMAKE_ARCHIVE_OUTPUT_DIRECTORY", &library_dir)
        .define("CMAKE_LIBRARY_OUTPUT_DIRECTORY", &library_dir)
        .define("CMAKE_RUNTIME_OUTPUT_DIRECTORY", &library_dir)
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
    config
}

fn build_fluidsynth() -> PathBuf {
    let mut config = common_config("lib/fluidsynth", "fluidsynth");
    config
        .build_target("libfluidsynth")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("enable-alsa", "OFF")
        .define("enable-aufile", "OFF")
        .define("enable-coreaudio", "OFF")
        .define("enable-coremidi", "OFF")
        .define("enable-dbus", "OFF")
        .define("enable-dsound", "OFF")
        .define("enable-framework", "OFF")
        .define("enable-ipv6", "OFF")
        .define("enable-jack", "OFF")
        .define("enable-ladspa", "OFF")
        .define("enable-libsndfile", "OFF")
        .define("enable-midishare", "OFF")
        .define("enable-native-dls", "OFF")
        .define("enable-network", "OFF")
        .define("enable-oboe", "OFF")
        .define("enable-openmp", "OFF")
        .define("enable-opensles", "OFF")
        .define("enable-oss", "OFF")
        .define("enable-pipewire", "OFF")
        .define("enable-pulseaudio", "OFF")
        .define("enable-readline", "OFF")
        .define("enable-sdl3", "OFF")
        .define("enable-signalsmith", "OFF")
        .define("enable-systemd", "OFF")
        .define("enable-wasapi", "OFF")
        .define("enable-waveout", "OFF")
        .define("enable-winmidi", "OFF");
    config.build()
}

fn build_sfizz() -> PathBuf {
    let mut config = common_config("lib/sfizz", "sfizz");
    config
        .build_target("sfizz_static")
        .define("BUILD_TESTING", "OFF")
        .define("ENABLE_LTO", "OFF")
        .define("SFIZZ_BENCHMARKS", "OFF")
        .define("SFIZZ_DEMOS", "OFF")
        .define("SFIZZ_DEVTOOLS", "OFF")
        .define("SFIZZ_GIT_SUBMODULE_CHECK", "OFF")
        .define("SFIZZ_JACK", "OFF")
        .define("SFIZZ_RENDER", "OFF")
        .define("SFIZZ_SHARED", "OFF")
        .define("SFIZZ_TESTS", "OFF")
        .define("SFIZZ_USE_SNDFILE", "OFF");
    // sfizz's pinned Abseil revision needs this explicit include with GCC 15.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        config.cxxflag("-include").cxxflag("cstdint");
    }
    // sfizz intentionally overrides CMake's output directory to build/library.
    config.build().join("build/library")
}

fn link_static_library(prefix: &Path, name: &str) {
    let library_dir = prefix.join("lib");
    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=static={name}");
}

fn link_sfizz_static_libraries(prefix: &Path) {
    let library_dir = prefix.join("lib");
    println!("cargo:rustc-link-search=native={}", library_dir.display());

    let mut libraries = fs::read_dir(&library_dir)
        .unwrap_or_else(|error| {
            panic!("failed to read sfizz library directory {}: {error}", library_dir.display())
        })
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| static_library_name(&entry.path()).map(str::to_owned))
        .collect::<Vec<_>>();
    libraries.sort_unstable();

    assert!(libraries.iter().any(|library| library == "sfizz"), "sfizz static archive is missing");

    // libsfizz.a is only the public API layer. Its implementation and bundled
    // dependencies are separate archives, which must all be available to the
    // final linker. Normal static linkage still allows unused objects to be
    // discarded from the executable.
    for library in libraries {
        println!("cargo:rustc-link-lib=static={library}");
    }
}

fn static_library_name(path: &Path) -> Option<&str> {
    let file_name = path.file_name()?.to_str()?;
    file_name.strip_prefix("lib")?.strip_suffix(".a")
}

fn link_cpp_runtime() {
    match env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => {}
        _ if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") => {
            println!("cargo:rustc-link-lib=dylib=c++");
        }
        _ => {
            println!("cargo:rustc-link-lib=dylib=stdc++");
            println!("cargo:rustc-link-lib=dylib=atomic");
            println!("cargo:rustc-link-lib=dylib=rt");
            println!("cargo:rustc-link-lib=dylib=m");
        }
    }
}

fn generate_bindings() {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));

    bindgen::Builder::default()
        .header("ffi/fluidsynth.h")
        .clang_arg("-Ilib/fluidsynth/include")
        .allowlist_type("fluid_settings_t")
        .allowlist_type("fluid_synth_t")
        .allowlist_type("fluid_sfloader_t")
        .allowlist_type("fluid_sfont_t")
        .allowlist_type("fluid_preset_t")
        .allowlist_function("new_fluid_settings")
        .allowlist_function("delete_fluid_settings")
        .allowlist_function("new_fluid_synth")
        .allowlist_function("delete_fluid_synth")
        .allowlist_function("delete_fluid_sfloader")
        .allowlist_function("new_fluid_defsfloader")
        .allowlist_function("fluid_sfloader_set_callbacks")
        .allowlist_function("fluid_settings_setnum")
        .allowlist_function("fluid_settings_setint")
        .allowlist_function("fluid_synth_add_sfloader")
        .allowlist_function("fluid_synth_sfload")
        .allowlist_function("fluid_synth_get_sfont_by_id")
        .allowlist_function("fluid_sfont_get_name")
        .allowlist_function("fluid_sfont_iteration_start")
        .allowlist_function("fluid_sfont_iteration_next")
        .allowlist_function("fluid_preset_get_name")
        .allowlist_function("fluid_preset_get_banknum")
        .allowlist_function("fluid_preset_get_num")
        .allowlist_function("aaw_fluid_preset_note_mask")
        .allowlist_function("fluid_synth_noteon")
        .allowlist_function("fluid_synth_noteoff")
        .allowlist_function("fluid_synth_cc")
        .allowlist_function("fluid_synth_pitch_bend")
        .allowlist_function("fluid_synth_bank_select")
        .allowlist_function("fluid_synth_program_select")
        .allowlist_function("fluid_synth_program_change")
        .allowlist_function("fluid_synth_channel_pressure")
        .allowlist_function("fluid_synth_key_pressure")
        .allowlist_function("fluid_synth_all_sounds_off")
        .allowlist_function("fluid_synth_set_sample_rate")
        .allowlist_function("fluid_synth_write_float")
        .generate_comments(false)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate FluidSynth bindings")
        .write_to_file(output.join("fluidsynth_bindings.rs"))
        .expect("failed to write FluidSynth bindings");

    bindgen::Builder::default()
        .header("ffi/sfizz.h")
        .clang_arg("-Ilib/sfizz/src")
        .allowlist_type("sfizz_arg_t")
        .allowlist_type("sfizz_client_t")
        .allowlist_type("sfizz_receive_t")
        .allowlist_type("sfizz_synth_t")
        .allowlist_function("sfizz_create_synth")
        .allowlist_function("sfizz_create_client")
        .allowlist_function("sfizz_delete_client")
        .allowlist_function("sfizz_free")
        .allowlist_function("sfizz_get_num_regions")
        .allowlist_function("sfizz_load_file")
        .allowlist_function("sfizz_load_string")
        .allowlist_function("sfizz_set_receive_callback")
        .allowlist_function("sfizz_set_samples_per_block")
        .allowlist_function("sfizz_set_sample_rate")
        .allowlist_function("sfizz_send_note_on")
        .allowlist_function("sfizz_send_note_off")
        .allowlist_function("sfizz_send_cc")
        .allowlist_function("sfizz_send_pitch_wheel")
        .allowlist_function("sfizz_send_program_change")
        .allowlist_function("sfizz_send_channel_aftertouch")
        .allowlist_function("sfizz_send_poly_aftertouch")
        .allowlist_function("sfizz_send_message")
        .allowlist_function("sfizz_all_sound_off")
        .allowlist_function("sfizz_render_block")
        .generate_comments(false)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate sfizz bindings")
        .write_to_file(output.join("sfizz_bindings.rs"))
        .expect("failed to write sfizz bindings");
}
