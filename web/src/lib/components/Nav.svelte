<script lang="ts">
    import { Button, LinkButton } from "$lib/components";
    import type { User } from "$lib/types";

    interface Props {
        currentPath: string;
        isAuthenticated: boolean;
        user?: User | null;
        onLogout?: () => void;
    }

    let { currentPath, isAuthenticated, user, onLogout }: Props = $props();
</script>

<nav>
    <div class="nav-container">
        <div class="nav-links">
            <a href="/" class="brand">Axel Tournament</a>
            <a
                href="/tournaments"
                class:active={currentPath.startsWith('/tournaments')}
            >
                Tournaments
            </a>
            <a
                href="/games"
                class:active={currentPath.startsWith('/games')}
            >
                Games
            </a>
            <a
                href="/rooms"
                class:active={currentPath.startsWith('/rooms')}
            >
                Rooms
            </a>
            <a
                href="/leaderboard"
                class:active={currentPath === '/leaderboard'}
            >
                Leaderboard
            </a>
        </div>
        <div class="nav-actions">
            {#if isAuthenticated}
                <a href="/profile" class:active={currentPath === '/profile'}>{user?.username}</a>
                <Button onclick={onLogout} label="Logout" variant="ghost" />
            {:else}
                <a href="/login" class:active={currentPath === '/login'}>Login</a>
                <LinkButton href="/register" variant="primary" label="Sign Up" />
            {/if}
        </div>
    </div>
</nav>

<style>
    nav {
        border-bottom: 1px solid var(--color-blueprint-line-light);
        background-color: var(--color-blueprint-paper);
        padding: 1rem 2rem;
    }

    .nav-container {
        max-width: 75rem;
        margin-inline: auto;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .nav-links,
    .nav-actions {
        display: flex;
        gap: 1.5rem;
        align-items: center;
    }

    a {
        text-decoration: none;
        font-weight: 500;
        color: inherit;
        transition: color var(--transition-fast), opacity var(--transition-fast);
    }

    a:hover {
        color: var(--color-primary);
    }

    a.active {
        color: var(--color-primary);
        font-weight: 600;
    }

    a:focus-visible {
        outline: 2px solid var(--color-primary);
        outline-offset: 2px;
    }

    .brand {
        font-size: 1.25rem;
        font-weight: 700;
        color: var(--color-primary);
    }

    .brand:hover {
        opacity: 0.8;
    }
</style>
