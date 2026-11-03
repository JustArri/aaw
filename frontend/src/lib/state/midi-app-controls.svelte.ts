import * as api from '$lib/generated/api';
import type {
    CtrMidiEngineControl,
    CtrMidiEngineControls,
    CtrMidiProjectControl,
    CtrMidiProjectControls,
    StoredMidiEngineControls,
    StoredMidiProjectControls
} from '$lib/generated/models';
import { gatewayNotResponding } from '$lib/error';
import { updateActiveProjectMidiAppControls } from './active-project.svelte';
import { reportGatewayOffline } from './gateway.svelte';

let engineControlsState = $state<CtrMidiEngineControls>({ controls: [] });
let fetching = false;
let saving = false;
let fetchingProjectControls = false;
let savingProjectControls = false;

export function engineMidiAppControls() {
    return engineControlsState;
}

export function updateEngineMidiAppControls(controls: CtrMidiEngineControls) {
    engineControlsState = controls;
}

function normalizeControl(
    control: StoredMidiEngineControls['controls'][number]
): CtrMidiEngineControl {
    if ('CCTransportSwitch' in control.kind) {
        return {
            ...control,
            kind: {
                CCTransportSwitch: {
                    controller: control.kind.CCTransportSwitch.controller,
                    threshold: control.kind.CCTransportSwitch.threshold ?? 64
                }
            }
        };
    }
    return control as CtrMidiEngineControl;
}

function normalizeProjectControl(
    control: StoredMidiProjectControls['controls'][number]
): CtrMidiProjectControl {
    if ('CCInstrumentControllerSwitch' in control.kind) {
        return {
            ...control,
            kind: {
                CCInstrumentControllerSwitch: {
                    ...control.kind.CCInstrumentControllerSwitch,
                    threshold: control.kind.CCInstrumentControllerSwitch.threshold ?? 64
                }
            }
        };
    }
    return control as CtrMidiProjectControl;
}

export async function fetchEngineMidiAppControls() {
    if (fetching) {
        return engineControlsState;
    }
    fetching = true;
    let response;
    try {
        response = await api.engineMidiAppControls();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }

    engineControlsState = { controls: response.data.controls.map(normalizeControl) };
    return engineControlsState;
}

export async function saveEngineMidiAppControls(controls: CtrMidiEngineControls) {
    if (saving) {
        return false;
    }
    saving = true;
    const snapshot = $state.snapshot(controls);
    const stored: StoredMidiEngineControls = {
        version: 1,
        controls: snapshot.controls
    };
    try {
        await api.setEngineMidiAppControls(stored);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        saving = false;
    }

    engineControlsState = snapshot;
    return true;
}

export async function fetchProjectMidiAppControls() {
    if (fetchingProjectControls) {
        return;
    }
    fetchingProjectControls = true;
    let response;
    try {
        response = await api.projectMidiAppControls();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingProjectControls = false;
    }

    const controls: CtrMidiProjectControls = {
        controls: response.data.controls.map(normalizeProjectControl)
    };
    updateActiveProjectMidiAppControls(controls);
    return controls;
}

export async function saveProjectMidiAppControls(controls: CtrMidiProjectControls) {
    if (savingProjectControls) {
        return false;
    }
    savingProjectControls = true;
    const snapshot = $state.snapshot(controls);
    const stored: StoredMidiProjectControls = {
        version: 1,
        controls: snapshot.controls
    };
    try {
        await api.setProjectMidiAppControls(stored);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        savingProjectControls = false;
    }

    updateActiveProjectMidiAppControls(snapshot);
    return true;
}
