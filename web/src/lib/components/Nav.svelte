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
    <div>
        <div>
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
        <div>
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

<style>
    nav {
        border-bottom: 1px solid var(--blueprint-line-light);
        background: var(--white);
        padding: 1rem 2rem;
    }

    /* Container - there's only one div child of nav */
    nav > div {
        max-width: 1200px;
        margin: 0 auto;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    /* Nav link groups - both divs inside the container */
    nav > div > div {
        display: flex;
        gap: 1.5rem;
        align-items: center;
    }

    a {
        text-decoration: none;
        color: var(--text);
        font-weight: 500;
        transition: color 0.15s ease;
    }

    a:hover {
        color: var(--primary);
    }

    a.active {
        color: var(--primary);
        font-weight: 600;
    }

    a.logo {
        font-size: 1.25rem;
        font-weight: 700;
        color: var(--primary);
    }

    a.logo:hover {
        opacity: 0.8;
    }
</style>
