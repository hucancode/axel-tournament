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
    <div class="nav-content">
        <div class="nav-links">
            <a href="/" class="logo">Axel Tournament</a>
            <a
                href="/tournaments"
                class:active={currentPath.startsWith("/tournaments")}
            >
                Tournaments
            </a>
            <a
                href="/games"
                class:active={currentPath.startsWith("/games")}
            >
                Games
            </a>
            <a
                href="/leaderboard"
                class:active={currentPath === "/leaderboard"}
            >
                Leaderboard
            </a>
        </div>
        <div class="nav-links">
            {#if isAuthenticated}
                {#if user?.role === "admin"}
                    <a href="/admin" class:active={currentPath.startsWith("/admin")}>Admin</a>
                {/if}
                {#if user?.role === "gamesetter" || user?.role === "admin"}
                    <a href="/game-setter" class:active={currentPath.startsWith("/game-setter")}>Game Setter</a>
                {/if}
                <a href="/profile" class:active={currentPath === "/profile"}>{user?.username}</a>
                <Button onclick={onLogout} label="Logout" variant="ghost" />
            {:else}
                <a href="/login" class:active={currentPath === "/login"}>Login</a>
                <LinkButton href="/register" variant="primary" label="Sign Up" />
            {/if}
        </div>
    </div>
</nav>
