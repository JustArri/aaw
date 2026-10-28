import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { ProjectInfo, StoredProject } from '$lib/generated/models';
import { unreachable } from '$lib/util';
import { fetchActiveProject } from './active-project.svelte';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let projectsState = $state([] as ProjectInfo[]);
let fetching = false;
let refreshing = false;
let saving = false;
let loading = false;
let deleting = false;
let exporting = false;

export function projects() {
    return projectsState;
}

export function resetProjects() {
    projectsState = [];
}

export async function fetchProjects() {
    if (fetching) {
        return;
    }
    fetching = true;

    let rspProjects;

    try {
        rspProjects = await api.listProjects();
    } catch (error) {
        fetching = false;
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    switch (rspProjects.status) {
        case 200:
            projectsState = rspProjects.data;
            break;
        default:
            fetching = false;
            resetProjects();
            reportGatewayError(rspProjects.status);
            throw gatewayErrorByStatus(rspProjects.status, rspProjects.data);
    }

    fetching = false;
}

export async function refreshProjects() {
    if (refreshing) {
        return;
    }
    refreshing = true;

    let rsp;

    try {
        rsp = await api.refreshProjects();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        refreshing = false;
    }

    switch (rsp.status) {
        case 204:
            await fetchProjects();
            break;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}

export async function saveProject(project: StoredProject) {
    if (saving) {
        return;
    }
    saving = true;

    let rsp;

    try {
        rsp = await api.writeNewProject(project);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        saving = false;
    }

    switch (rsp.status) {
        case 200:
            await fetchProjects();
            return rsp.data;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}

export async function loadProject(projectId: string) {
    if (loading) {
        return;
    }
    loading = true;

    let rsp;

    try {
        rsp = await api.readProject(projectId);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        loading = false;
    }

    switch (rsp.status) {
        case 200:
            return rsp.data;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}

export async function loadActiveProject(projectId: string) {
    if (loading) {
        return;
    }
    loading = true;

    let rsp;

    try {
        rsp = await api.loadActiveProject(projectId);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        loading = false;
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

export async function importActiveProject(project: StoredProject) {
    if (loading) {
        return;
    }
    loading = true;

    let rsp;

    try {
        rsp = await api.importActiveProject(project);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        loading = false;
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

export async function exportActiveProject() {
    if (exporting) {
        return;
    }
    exporting = true;

    let rsp;

    try {
        rsp = await api.exportActiveProject();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        exporting = false;
    }

    const status = rsp.status;

    switch (status) {
        case 200:
            return rsp.data;
        default:
            unreachable(status);
    }
}

export async function saveActiveProject() {
    if (saving) {
        return;
    }
    saving = true;

    let rsp;

    try {
        rsp = await api.saveActiveProject();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        saving = false;
    }

    switch (rsp.status) {
        case 200:
            await fetchProjects();
            await fetchActiveProject();
            return rsp.data;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}

export async function deleteProject(projectId: string) {
    if (deleting) {
        return;
    }
    deleting = true;

    let rsp;

    try {
        rsp = await api.deleteProject(projectId);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        deleting = false;
    }

    switch (rsp.status) {
        case 204:
            projectsState = projectsState.filter((project) => project.id !== projectId);
            break;
        default:
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status, rsp.data);
    }
}
