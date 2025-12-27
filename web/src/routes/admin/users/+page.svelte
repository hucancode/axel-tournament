<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { adminService } from "$lib/services/admin";
    import { Button, LinkButton } from "$lib/components";
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
            <LinkButton variant="secondary" href="/admin" label="Back to Dashboard" />
        </div>
    </div>
    {#if error}
        <div
            class="card bg-red-100 border-l-4 border-red-600 mb-4"
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
            <div class="overflow-x-auto">
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
                                        <Button
                                            variant="success"
                                            label="Unban"
                                            onclick={() => handleUnban(user)}
                                        />
                                        {#if user.ban_reason}
                                            <p
                                                class="text-sm text-gray-500 mt-1"
                                            >
                                                Reason: {user.ban_reason}
                                            </p>
                                        {/if}
                                    {:else if user.role !== "admin"}
                                        <Button
                                            variant="danger"
                                            label="Ban"
                                            onclick={() => openBanDialog(user)}
                                        />
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
                    <Button
                        variant="secondary"
                        label="Previous"
                        disabled={currentPage === 1}
                        onclick={() => {
                            currentPage--;
                            loadUsers();
                        }}
                    />
                    <span class="text-gray-700">
                        Page {currentPage} of {totalPages}
                    </span>
                    <Button
                        variant="secondary"
                        label="Next"
                        disabled={currentPage === totalPages}
                        onclick={() => {
                            currentPage++;
                            loadUsers();
                        }}
                    />
                </div>
            {/if}
        </div>
    {/if}
</div>

<!-- Ban Dialog Modal -->
{#if showBanDialog && selectedUser}
    <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]"
        role="button"
        tabindex="0"
        aria-label="Close ban dialog"
        onclick={(e) => {
            if (e.target === e.currentTarget) closeBanDialog();
        }}
        onkeydown={(e) => {
            if (
                e.target === e.currentTarget &&
                (e.key === "Escape" || e.key === "Enter" || e.key === " ")
            ) {
                e.preventDefault();
                closeBanDialog();
            }
        }}
    >
        <div
            class="card max-w-[500px] w-[90%] max-h-[90vh] overflow-y-auto"
        >
            <h2 class="font-bold text-xl mb-4">
                Ban User: {selectedUser.username}
            </h2>
            {#if banError}
                <div
                    class="bg-red-100 border-l-4 border-red-600 p-4 mb-4"
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
                    <Button
                        type="submit"
                        variant="danger"
                        disabled={banLoading || !banReason.trim()}
                        label={banLoading ? "Banning..." : "Ban User"}
                    />
                    <Button
                        type="button"
                        variant="secondary"
                        disabled={banLoading}
                        label="Cancel"
                        onclick={closeBanDialog}
                    />
                </div>
            </form>
        </div>
    </div>
{/if}
