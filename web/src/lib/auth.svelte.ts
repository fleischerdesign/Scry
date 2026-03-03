import type { User } from "./types/User";

// Scry Auth State - Svelte 5 Runes Edition
class AuthState {
    token = $state<string | null>(localStorage.getItem("scry_token"));
    user = $state<User | null>(null);
    isAuthenticated = $derived(this.user !== null && this.token !== null);

    constructor() {
        const storedUser = localStorage.getItem("scry_user");
        if (storedUser) {
            try {
                this.user = JSON.parse(storedUser);
            } catch {
                this.logout();
            }
        }
    }

    login(token: string, user: User) {
        this.token = token;
        this.user = user;
        localStorage.setItem("scry_token", token);
        localStorage.setItem("scry_user", JSON.stringify(user));
    }

    logout() {
        this.token = null;
        this.user = null;
        localStorage.removeItem("scry_token");
        localStorage.removeItem("scry_user");
    }
}

export const auth = new AuthState();
