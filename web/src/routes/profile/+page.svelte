<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { authService } from "$services/auth";
    import { submissionService } from "$services/submissions";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import type { Submission } from "$lib/models";
    let user = $state($authStore.user);
    let submissions = $state<Submission[]>([]);
    let location = $state(user?.location || "");
    let loading = $state(false);
    let error = $state("");
    let success = $state("");
    onMount(async () => {
        if (!$authStore.isAuthenticated) {
            goto("/login");
            return;
        }
        try {
            submissions = await submissionService.list();
        } catch (err) {
            console.error("Failed to load submissions:", err);
        }
    });
    async function updateLocation() {
        error = "";
        success = "";
        loading = true;
        try {
            const updated = await authService.updateLocation(location);
            authStore.setAuth(updated, $authStore.token!);
            success = "Location updated successfully";
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to update location";
        } finally {
            loading = false;
        }
    }
</script>

<style>
    .user-info-section, .location-section, .submissions-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
        margin-bottom: var(--spacing-4);
    }

    .user-info-grid {
        display: grid;
        grid-template-columns: 1fr 2fr;
        gap: var(--spacing-4);
    }

    .user-info-grid dt {
        font-weight: 600;
    }

    .error-message {
        border: 1px solid var(--color-error);
        padding: var(--spacing-6);
        background-color: var(--color-bg-popup);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .success-message {
        padding: var(--spacing-6);
        background-color: var(--color-bg-popup);
        margin-bottom: var(--spacing-4);
        color: var(--color-success);
    }

    .location-form {
        margin-bottom: var(--spacing-4);
    }

    .location-form label {
        display: block;
        margin-bottom: var(--spacing-2);
        font-weight: 500;
        color: var(--color-fg);
    }

    .form-help {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-top: var(--spacing-1);
    }

    .empty-state {
        text-align: center;
        color: var(--color-fg-muted);
    }

    .submissions-table th {
        border-bottom: 1px solid var(--color-border);
    }

    .submissions-table td {
        border-bottom: 1px solid var(--color-border-light);
    }

    .tournament-link {
        color: var(--color-primary);
        text-decoration: none;
    }

    .submit-date {
        color: var(--color-fg-muted);
    }
</style>

<main>
    <div class="container">
        <section class="user-info-section">
            <h2>User Information</h2>
            <dl class="user-info-grid">
                <dt>Username:</dt>
                <dd>{user?.username}</dd>
                <dt>Email:</dt>
                <dd>{user?.email}</dd>
                <dt>Role:</dt>
                <dd>
                    <span
                        class="badge"
                        class:badge-running={user?.role === "admin"}
                        class:badge-scheduled={user?.role === "player"}
                    >
                        {user?.role}
                    </span>
                </dd>
                <dt>Member Since:</dt>
                <dd>
                    {user?.created_at
                        ? new Date(user.created_at).toLocaleDateString()
                        : "N/A"}
                </dd>
            </dl>
        </section>

        <section class="location-section">
            <h2>Update Location</h2>
            {#if error}
                <div class="error-message">
                    <p>{error}</p>
                </div>
            {/if}
            {#if success}
                <div class="success-message">
                    <p>{success}</p>
                </div>
            {/if}
            <div class="location-form">
                <label for="location">Country Code</label>
                <input
                    type="text"
                    id="location"
                    class="input"
                    bind:value={location}
                    placeholder="US, UK, FR, etc."
                    maxlength="2"
                    disabled={loading}
                />
                <p class="form-help">2-letter ISO country code</p>
            </div>
            <button
                onclick={updateLocation}
                data-variant="primary"
                disabled={loading}
            >
                {loading ? "Updating..." : "Update Location"}
            </button>
        </section>

        <section class="submissions-section">
            <h2>My Submissions</h2>
            {#if submissions.length === 0}
                <p class="empty-state">No submissions yet</p>
            {:else}
                <table class="submissions-table">
                    <thead>
                        <tr>
                            <th>Tournament</th>
                            <th>Language</th>
                            <th>Status</th>
                            <th>Submitted</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each submissions as submission}
                            <tr>
                                <td>
                                    <a
                                        href="/tournaments/{submission.tournament_id}"
                                        class="tournament-link"
                                    >
                                        View Tournament
                                    </a>
                                </td>
                                <td>{submission.language}</td>
                                <td>
                                    <span class="badge badge-{submission.status}">
                                        {submission.status}
                                    </span>
                                </td>
                                <td class="submit-date">
                                    {new Date(submission.created_at).toLocaleDateString()}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </section>
    </div>
</main>
