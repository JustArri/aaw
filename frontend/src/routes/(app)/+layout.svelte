<script lang="ts">
    import { page } from '$app/state';
    import Navigation, { type NavigationEntry } from '$lib/components/Navigation.svelte';
    import NotificationCenter from '$lib/components/NotificationCenter.svelte';
    import TopBar from '$lib/components/TopBar.svelte';
    import { autoUpdateActiveProject } from '$lib/state/active-project.svelte';
    import { autoUpdateAudio } from '$lib/state/audio.svelte';
    import { autoUpdateGatewayState } from '$lib/state/gateway.svelte';
    import { autoloadMidiEventLogSize } from '$lib/state/midi-events.svelte';
    import { autoSubscribeServerNotifications } from '$lib/state/server-notifications.svelte';
    import { autoloadTheme } from '$lib/state/theme.svelte';
    import '../../app.css';

    let { children } = $props();

    const navigation: NavigationEntry[] = [
        { type: 'title', label: 'Active project' },
        { type: 'item', label: 'Live', href: '/', icon: 'lucide:activity' },
        {
            type: 'item',
            label: 'Instruments',
            href: '/instruments',
            icon: 'lucide:sliders-horizontal'
        },
        { type: 'item', label: 'Synths', href: '/synths', icon: 'lucide:grid-2x2' },
        { type: 'item', label: 'User Presets', href: '/user-presets', icon: 'lucide:list-music' },
        { type: 'title', label: 'Engine' },
        { type: 'item', label: 'MIDI Inputs', href: '/midi-inputs', icon: 'lucide:cable' },
        { type: 'item', label: 'MIDI Events', href: '/midi-events', icon: 'lucide:piano' },
        { type: 'item', label: 'Audio', href: '/audio', icon: 'lucide:audio-lines' },
        { type: 'title', label: 'Files' },
        { type: 'item', label: 'Projects', href: '/projects', icon: 'lucide:folders' },
        { type: 'item', label: 'Assets', href: '/assets', icon: 'lucide:archive' }
    ];

    const routeLabels: Record<string, string> = {
        '/': 'Live',
        '/live': 'Live',
        '/instruments': 'Instruments',
        '/user-presets': 'User Presets',
        '/synths': 'Synths',
        '/midi-inputs': 'MIDI Inputs',
        '/midi-events': 'MIDI Events',
        '/audio': 'Audio',
        '/projects': 'Projects',
        '/assets': 'Assets',
        '/settings': 'Settings'
    };

    let activeLabel = $derived(routeLabels[page.url.pathname] ?? '');

    autoloadTheme();
    autoloadMidiEventLogSize();
    autoUpdateGatewayState();
    autoSubscribeServerNotifications();
    autoUpdateAudio();
    autoUpdateActiveProject();
</script>

<svelte:head>
    <title>AAW Control</title>
    <meta name="description" content="Control surface for the AAW real-time audio engine" />
</svelte:head>

<div
    class="bg-app-background text-foreground [&_button:focus-visible]:outline-accent [&_input:focus-visible]:outline-accent min-h-screen min-w-[20rem] font-sans antialiased min-[56.25rem]:grid min-[56.25rem]:grid-cols-[13.875rem_minmax(0,1fr)] [&_button:focus-visible]:outline-2 [&_button:focus-visible]:outline-offset-2 [&_input:focus-visible]:outline-2 [&_input:focus-visible]:outline-offset-2">
    <Navigation items={navigation} {activeLabel} />

    <section class="min-w-0">
        <TopBar {activeLabel} />

        <main
            class="px-3 pt-3 pb-23.5 min-[37.5rem]:px-5 min-[56.25rem]:mx-auto min-[56.25rem]:w-full min-[56.25rem]:max-w-360 min-[56.25rem]:px-7.5 min-[56.25rem]:pt-4 min-[56.25rem]:pb-10">
            {@render children()}
        </main>
    </section>
</div>

<NotificationCenter />
