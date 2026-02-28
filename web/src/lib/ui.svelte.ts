// Scry UI State - Svelte 5 Runes
export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
    id: string;
    message: string;
    description?: string;
    type: ToastType;
}

class UIState {
    toasts = $state<Toast[]>([]);

    notify(message: string, description?: string, type: ToastType = 'info') {
        const id = Math.random().toString(36).substring(2);
        this.toasts.push({ id, message, description, type });
        
        // Automatisch nach 5 Sekunden entfernen
        setTimeout(() => {
            this.toasts = this.toasts.filter(t => t.id !== id);
        }, 5000);
    }
}

export const ui = new UIState();
