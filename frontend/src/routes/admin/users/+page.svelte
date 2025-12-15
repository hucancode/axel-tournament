<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { adminService } from "$lib/services/admin";
    import type { User } from "$lib/types";
    let users = $state<User[]>([]);
    let loading = $state(true);
    let error = $state("");
    let currentPage = $state(1);
    let totalPages = $state(1);
    const pageSize = 50;
    // Ban dialog state
    let showBanDialog = $state(false);
    let selectedUser = $state<User | null>(null);
    let banReason = $state("");
    let banLoading = $state(false);
    let banError = $state("");
    const auth = $derived($authStore);
    onMount(async () => {
        // Check authentication and admin role
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        if (auth.user?.role !== "admin") {
            goto("/");
            return;
        }
        await loadUsers();
    });
    async function loadUsers() {
        loading = true;
        error = "";
        try {
            const allUsers = await adminService.listUsers(
                currentPage,
                pageSize,
            );
            users = allUsers;
            // Note: Backend should ideally return total count for pagination
            // For now, we'll show all users on one page
            totalPages = 1;
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load users";
        } finally {
            loading = false;
        }
    }
    function openBanDialog(user: User) {
        selectedUser = user;
        banReason = "";
        banError = "";
        showBanDialog = true;
    }
    function closeBanDialog() {
        showBanDialog = false;
        selectedUser = null;
        banReason = "";
        banError = "";
    }
    async function handleBan() {
        if (!selectedUser || !banReason.trim()) {
            banError = "Please provide a reason for banning";
            return;
        }
        banLoading = true;
        banError = "";
        try {
            await adminService.banUser(selectedUser.id, banReason);
            await loadUsers(); // Reload users list
            closeBanDialog();
        } catch (err) {
            banError =
                err instanceof Error ? err.message : "Failed to ban user";
        } finally {
            banLoading = false;
        }
    }
    async function handleUnban(user: User) {
        if (!confirm(`Are you sure you want to unban ${user.username}?`)) {
            return;
        }
        try {
            await adminService.unbanUser(user.id);
            await loadUsers(); // Reload users list
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to unban user";
        }
    }
    function formatDate(dateStr: string): string {
        return new Date(dateStr).toLocaleDateString("en-US", {
            year: "numeric",
            month: "short",
            day: "numeric",
        });
    }
</script>

<div class="container page">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">User Management</h1>
                <p class="text-gray-500">
                    Manage user accounts and permissions
                </p>
            </div>
            <a href="/admin" class="btn btn-secondary">Back to Dashboard</a>
        </div>
    </div>
    {#if error}
        <div
            class="card"
            style="background-color: #fee2e2; border-left: 4px solid var(--red-600); margin-bottom: 1rem;"
        >
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <div class="card text-center">
            <p class="text-gray-500">Loading users...</p>
        </div>
    {:else if users.length === 0}
        <div class="card text-center">
            <p class="text-gray-500">No users found</p>
        </div>
    {:else}
        <div class="card">
            <div style="overflow-x: auto;">
                <table>
                    <thead>
                        <tr>
                            <th>Username</th>
                            <th>Email</th>
                            <th>Role</th>
                            <th>Location</th>
                            <th>Status</th>
                            <th>Joined</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each users as user}
                            <tr>
                                <td>
                                    <span class="font-semibold"
                                        >{user.username}</span
                                    >
                                </td>
                                <td>{user.email}</td>
                                <td>
                                    <span
                                        class="badge {user.role === 'admin'
                                            ? 'badge-running'
                                            : 'badge-scheduled'}"
                                    >
                                        {user.role}
                                    </span>
                                </td>
                                <td>{user.location || "N/A"}</td>
                                <td>
                                    {#if user.is_banned}
                                        <span class="badge badge-failed"
                                            >Banned</span
                                        >
                                    {:else}
                                        <span class="badge badge-accepted"
                                            >Active</span
                                        >
                                    {/if}
                                </td>
                                <td class="text-sm text-gray-500">
                                    {formatDate(user.created_at)}
                                </td>
                                <td>
                                    {#if user.is_banned}
                                        <button
                                            class="btn btn-success btn-sm"
                                            style="padding: 0.25rem 0.75rem; font-size: 0.875rem;"
                                            onclick={() => handleUnban(user)}
                                        >
                                            Unban
                                        </button>
                                        {#if user.ban_reason}
                                            <p
                                                class="text-sm text-gray-500"
                                                style="margin-top: 0.25rem;"
                                            >
                                                Reason: {user.ban_reason}
                                            </p>
                                        {/if}
                                    {:else if user.role !== "admin"}
                                        <button
                                            class="btn btn-danger btn-sm"
                                            style="padding: 0.25rem 0.75rem; font-size: 0.875rem;"
                                            onclick={() => openBanDialog(user)}
                                        >
                                            Ban
                                        </button>
                                    {:else}
                                        <span class="text-sm text-gray-500"
                                            >Admin</span
                                        >
                                    {/if}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            <!-- Pagination (for future implementation) -->
            {#if totalPages > 1}
                <div class="flex justify-between items-center mt-4">
                    <button
                        class="btn btn-secondary"
                        disabled={currentPage === 1}
                        onclick={() => {
                            currentPage--;
                            loadUsers();
                        }}
                    >
                        Previous
                    </button>
                    <span class="text-gray-700">
                        Page {currentPage} of {totalPages}
                    </span>
                    <button
                        class="btn btn-secondary"
                        disabled={currentPage === totalPages}
                        onclick={() => {
                            currentPage++;
                            loadUsers();
                        }}
                    >
                        Next
                    </button>
                </div>
            {/if}
        </div>
    {/if}
</div>

<!-- Ban Dialog Modal -->
{#if showBanDialog && selectedUser}
    <div
        style="
			position: fixed;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			background: rgba(0, 0, 0, 0.5);
			display: flex;
			align-items: center;
			justify-content: center;
			z-index: 1000;
		"
        onclick={(e) => {
            if (e.target === e.currentTarget) closeBanDialog();
        }}
    >
        <div
            class="card"
            style="
				max-width: 500px;
				width: 90%;
				max-height: 90vh;
				overflow-y: auto;
			"
        >
            <h2 class="font-bold text-xl mb-4">
                Ban User: {selectedUser.username}
            </h2>
            {#if banError}
                <div
                    style="background-color: #fee2e2; border-left: 4px solid var(--red-600); padding: 1rem; margin-bottom: 1rem;"
                >
                    <p class="text-red-600">{banError}</p>
                </div>
            {/if}
            <form
                onsubmit={(e) => {
                    e.preventDefault();
                    handleBan();
                }}
            >
                <div class="form-group">
                    <label for="banReason">Reason for ban</label>
                    <textarea
                        id="banReason"
                        class="textarea"
                        bind:value={banReason}
                        disabled={banLoading}
                        rows="4"
                        placeholder="Enter the reason for banning this user..."
                        required
                    ></textarea>
                </div>
                <div class="flex gap-2">
                    <button
                        type="submit"
                        class="btn btn-danger"
                        disabled={banLoading || !banReason.trim()}
                    >
                        {banLoading ? "Banning..." : "Ban User"}
                    </button>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        disabled={banLoading}
                        onclick={closeBanDialog}
                    >
                        Cancel
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
