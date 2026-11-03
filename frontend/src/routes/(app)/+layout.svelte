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
        { type: 'item', label: 'Instruments', href: '/synths', icon: 'lucide:grid-2x2' },
        { type: 'item', label: 'User Presets', href: '/user-presets', icon: 'lucide:list-music' },
        {
            type: 'item',
            label: 'MIDI App Controls',
            href: '/project-midi-app-controls',
            icon: 'lucide:sliders-horizontal'
        },
        { type: 'title', label: 'Engine' },
        {
            type: 'item',
            label: 'Performance',
            href: '/engine-performance',
            icon: 'lucide:chart-no-axes-combined'
        },
        { type: 'item', label: 'MIDI Inputs', href: '/midi-inputs', icon: 'lucide:cable' },
        { type: 'item', label: 'Keyboard', href: '/keyboard', icon: 'lucide:keyboard' },
        { type: 'item', label: 'MIDI Events', href: '/midi-events', icon: 'lucide:piano' },
        {
            type: 'item',
            label: 'MIDI App Controls',
            href: '/midi-app-controls',
            icon: 'lucide:sliders-horizontal'
        },
        { type: 'item', label: 'Audio', href: '/audio', icon: 'lucide:audio-lines' },
        { type: 'item', label: 'System', href: '/system', icon: 'lucide:power' },
        { type: 'title', label: 'Files' },
        { type: 'item', label: 'Projects', href: '/projects', icon: 'lucide:folders' },
        {
            type: 'item',
            label: 'MIDI Configurations',
            href: '/engine-midi-configurations',
            icon: 'lucide:file-cog'
        },
        { type: 'item', label: 'Assets', href: '/assets', icon: 'lucide:archive' },
        { type: 'item', label: 'Recordings', href: '/recordings', icon: 'lucide:file-music' }
    ];

    const routeLabels: Record<string, string> = {
        ...Object.fromEntries(
            navigation.flatMap((entry) =>
                entry.type === 'item' ? [[entry.href, entry.label]] : []
            )
        ),
        '/live': 'Live',
        '/instruments': 'Controllers',
        '/controllers': 'Instrument controllers',
        '/settings': 'Settings'
    };
    const navigationAliases: Record<string, string> = {
        '/live': '/',
        '/instruments': '/synths',
        '/controllers': '/synths'
    };

    let activeLabel = $derived(routeLabels[page.url.pathname] ?? '');
    let activeHref = $derived(navigationAliases[page.url.pathname] ?? page.url.pathname);
    let navigationOpen = $state(false);

    function openNavigation() {
        navigationOpen = true;
    }

    function closeNavigation() {
        navigationOpen = false;
    }

    function onWindowKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            closeNavigation();
        }
    }

    autoloadTheme();
    autoloadMidiEventLogSize();
    autoUpdateGatewayState();
    autoSubscribeServerNotifications();
    autoUpdateAudio();
    autoUpdateActiveProject();
</script>

<svelte:window onkeydown={onWindowKeydown} />

<svelte:head>
    <title>AAW Control</title>
    <meta name="description" content="Control surface for the AAW real-time audio engine" />
</svelte:head>

<div
    class="bg-app-background text-foreground grid h-dvh min-h-0 min-w-0 grid-cols-[minmax(0,1fr)] overflow-hidden font-sans antialiased min-[56.25rem]:grid-cols-[13.875rem_minmax(0,1fr)]">
    <Navigation items={navigation} {activeHref} open={navigationOpen} onClose={closeNavigation} />

    <section class="grid min-h-0 min-w-0 grid-rows-[auto_minmax(0,1fr)]">
        <TopBar {activeLabel} onMenuOpen={openNavigation} />

        <main
            class="h-full min-h-0 min-w-0 overflow-x-clip overflow-y-auto px-3 pt-3 pb-6 min-[37.5rem]:px-5 min-[56.25rem]:mx-auto min-[56.25rem]:w-full min-[56.25rem]:max-w-360 min-[56.25rem]:px-7.5 min-[56.25rem]:pt-4 min-[56.25rem]:pb-10">
            {@render children()}
        </main>
    </section>
</div>

<NotificationCenter />
