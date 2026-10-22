import type { Effects, Project } from '$lib/generated/models';
import * as api from '$lib/generated/api';
import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import { untrack } from 'svelte';
import { gatewayOnline, reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let projectState = $state(null as Project | null);
let fetchingProject = false;
let fetchingProjectName = false;
let fetchingProjectStorage = false;
let settingProjectName = false;
let creatingProject = false;

export function activeProject() {
    return projectState;
}

export function autoUpdateActiveProject() {
    $effect(() => {
        if (!gatewayOnline()) {
            return;
        }
        void untrack(fetchActiveProject);
    });
}

export function resetActiveProject() {
    projectState = null;
}

export async function createNewActiveProject() {
    if (creatingProject) {
        return;
    }
    creatingProject = true;

    let rsp;

    try {
        rsp = await api.newActiveProject();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        creatingProject = false;
    }

    switch (rsp.status) {
        case 204:
            await fetchActiveProject();
            break;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}

export async function fetchActiveProject() {
    if (fetchingProject) {
        return;
    }
    fetchingProject = true;

    let rsp;

    try {
        rsp = await api.activeProject();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingProject = false;
    }

    const status = rsp.status;

    switch (status) {
        case 200:
            projectState = rsp.data;
            break;
        default:
            projectState = null;
            reportGatewayError(status);
            throw gatewayErrorByStatus(status);
    }
}

export async function fetchActiveProjectStorage() {
    if (fetchingProjectStorage) {
        return;
    }
    fetchingProjectStorage = true;

    let rsp;

    try {
        rsp = await api.activeProjectStorage();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingProjectStorage = false;
    }

    if (projectState) {
        projectState.storage = rsp.data;
    }
}

export function updateActiveProjectRevision(revision: number) {
    if (projectState) {
        projectState.revision = revision;
    }
}

export function updateActiveProjectStorage(storage: Project['storage']) {
    if (projectState) {
        projectState.storage = storage;
    }
}

export function updateActiveProjectName(name: string) {
    if (projectState) {
        projectState.name = name;
    }
}

export function updateActiveProjectEffects(effects: Effects) {
    if (projectState) {
        projectState.synth_graph.effects = effects;
    }
}

export async function fetchActiveProjectName() {
    if (fetchingProjectName) {
        return;
    }
    fetchingProjectName = true;

    let rsp;

    try {
        rsp = await api.activeProjectName();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingProjectName = false;
    }

    const status = rsp.status;

    switch (status) {
        case 200:
            if (projectState) {
                projectState.name = rsp.data;
            }
            break;
        default:
            reportGatewayError(status);
            throw gatewayErrorByStatus(status);
    }
}

export async function setActiveProjectName(name: string) {
    if (settingProjectName) {
        return;
    }
    settingProjectName = true;

    let rsp;

    try {
        rsp = await api.setActiveProjectName(name);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        settingProjectName = false;
    }

    const status = rsp.status;

    switch (status) {
        case 204:
            if (projectState) {
                projectState.name = name;
            }
            break;
        default:
            reportGatewayError(status);
            throw gatewayErrorByStatus(status, rsp.data);
    }
}
