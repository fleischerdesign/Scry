import type { User } from "./types/User";

// Scry Auth State - Svelte 5 Runes Edition
class AuthState {
    apiKey = $state<string | null>(localStorage.getItem("scry_api_key"));
    user = $state<User | null>(null);
    isAuthenticated = $derived(this.user !== null && this.apiKey !== null);

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

    login(apiKey: string, user: User) {
        this.apiKey = apiKey;
        this.user = user;
        localStorage.setItem("scry_api_key", apiKey);
        localStorage.setItem("scry_user", JSON.stringify(user));
    }

    logout() {
        this.apiKey = null;
        this.user = null;
        localStorage.removeItem("scry_api_key");
        localStorage.removeItem("scry_user");
    }
}

export const auth = new AuthState();
