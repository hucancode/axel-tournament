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

<nav class="border-b border-blueprint-line-light bg-blueprint-paper px-8 py-4">
    <div class="max-w-300 mx-auto flex justify-between items-center">
        <div class="flex gap-6 items-center">
            <a href="/" class="text-xl font-bold text-primary no-underline hover:opacity-80 transition-opacity">Axel Tournament</a>
            <a
                href="/tournaments"
                class="no-underline font-medium transition-colors hover:text-primary {currentPath.startsWith('/tournaments') ? 'text-primary font-semibold' : ''}"
            >
                Tournaments
            </a>
            <a
                href="/games"
                class="no-underline font-medium transition-colors hover:text-primary {currentPath.startsWith('/games') ? 'text-primary font-semibold' : ''}"
            >
                Games
            </a>
            <a
                href="/leaderboard"
                class="no-underline font-medium transition-colors hover:text-primary {currentPath === '/leaderboard' ? 'text-primary font-semibold' : ''}"
            >
                Leaderboard
            </a>
        </div>
        <div class="flex gap-6 items-center">
            {#if isAuthenticated}
                {#if user?.role === "admin"}
                    <a href="/admin" class="no-underline font-medium transition-colors hover:text-primary {currentPath.startsWith('/admin') ? 'text-primary font-semibold' : ''}">Admin</a>
                {/if}
                {#if user?.role === "gamesetter" || user?.role === "admin"}
                    <a href="/game-setter" class="no-underline font-medium transition-colors hover:text-primary {currentPath.startsWith('/game-setter') ? 'text-primary font-semibold' : ''}">Game Setter</a>
                {/if}
                <a href="/profile" class="no-underline font-medium transition-colors hover:text-primary {currentPath === '/profile' ? 'text-primary font-semibold' : ''}">{user?.username}</a>
                <Button onclick={onLogout} label="Logout" variant="ghost" />
            {:else}
                <a href="/login" class="no-underline font-medium transition-colors hover:text-primary {currentPath === '/login' ? 'text-primary font-semibold' : ''}">Login</a>
                <LinkButton href="/register" variant="primary" label="Sign Up" />
            {/if}
        </div>
    </div>
</nav>
