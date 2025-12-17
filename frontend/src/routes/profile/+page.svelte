<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { authService } from "$lib/services/auth";
    import { submissionService } from "$lib/services/submissions";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import type { Submission } from "$lib/types";
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

<div class="page">
    <div class="container" style="max-width: 800px;">
        <div class="page-header">
            <h1 class="page-title">My Profile</h1>
        </div>
        <div class="card mb-4">
            <h2 class="text-xl font-semibold mb-4">User Information</h2>
            <div
                class="grid"
                style="grid-template-columns: 1fr 2fr; gap: 1rem;"
            >
                <div class="font-semibold">Username:</div>
                <div>{user?.username}</div>
                <div class="font-semibold">Email:</div>
                <div>{user?.email}</div>
                <div class="font-semibold">Role:</div>
                <div>
                    <span
                        class="badge"
                        class:badge-running={user?.role === "admin"}
                        class:badge-scheduled={user?.role === "player"}
                    >
                        {user?.role}
                    </span>
                </div>
                <div class="font-semibold">Member Since:</div>
                <div>
                    {user?.created_at
                        ? new Date(user.created_at).toLocaleDateString()
                        : "N/A"}
                </div>
            </div>
        </div>
        <div class="card mb-4">
            <h2 class="text-xl font-semibold mb-4">Update Location</h2>
            {#if error}
                <div
                    class="card"
                    style="background: #fee2e2; margin-bottom: 1rem;"
                >
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}
            {#if success}
                <div
                    class="card"
                    style="background: #d1fae5; margin-bottom: 1rem;"
                >
                    <p class="text-green-600">{success}</p>
                </div>
            {/if}
            <div class="form-group">
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
                <p class="text-sm text-gray-500" style="margin-top: 0.25rem;">
                    2-letter ISO country code
                </p>
            </div>
            <button
                onclick={updateLocation}
                class="btn btn-primary"
                disabled={loading}
            >
                {loading ? "Updating..." : "Update Location"}
            </button>
        </div>
        <div class="card">
            <h2 class="text-xl font-semibold mb-4">My Submissions</h2>
            {#if submissions.length === 0}
                <p class="text-center text-gray-500">No submissions yet</p>
            {:else}
                <table>
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
                                        class="text-primary-600"
                                    >
                                        View Tournament
                                    </a>
                                </td>
                                <td>{submission.language}</td>
                                <td>
                                    <span
                                        class="badge badge-{submission.status}"
                                    >
                                        {submission.status}
                                    </span>
                                </td>
                                <td
                                    >{new Date(
                                        submission.created_at,
                                    ).toLocaleDateString()}</td
                                >
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    </div>
</div>
