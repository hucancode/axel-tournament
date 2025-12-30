<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { authService } from "$lib/services/auth";
    import { submissionService } from "$lib/services/submissions";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import type { Submission } from "$lib/types";
    import { Button } from "$lib/components";
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

<section class="container max-w-3xl">
    <div class="p-6 bg-hatch mb-4">
        <h2 class="text-xl font-semibold mb-4">User Information</h2>
        <div class="grid grid-cols-[1fr_2fr] gap-4">
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
    <div class="p-6 bg-hatch mb-4">
        <h2 class="text-xl font-semibold mb-4">Update Location</h2>
        {#if error}
            <div class="border p-6 bg-hatch bg-red-100 mb-4">
                <p class="text-red-600">{error}</p>
            </div>
        {/if}
        {#if success}
            <div
                class="p-6 bg-hatch bg-green-100 mb-4"
            >
                <p class="text-green-600">{success}</p>
            </div>
        {/if}
        <div class="mb-4">
            <label for="location" class="block mb-2 font-medium text-gray-dark"
                >Country Code</label
            >
            <input
                type="text"
                id="location"
                class="input"
                bind:value={location}
                placeholder="US, UK, FR, etc."
                maxlength="2"
                disabled={loading}
            />
            <p class="text-sm text-gray-500 mt-1">2-letter ISO country code</p>
        </div>
        <Button
            onclick={updateLocation}
            variant="primary"
            label={loading ? "Updating..." : "Update Location"}
            disabled={loading}
        />
    </div>
    <div class="p-6 bg-hatch">
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
                                <span class="badge badge-{submission.status}">
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
</section>
