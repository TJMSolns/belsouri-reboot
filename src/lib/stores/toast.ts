import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;  // ms, 0 = persistent
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  function add(type: ToastType, message: string, duration = 4000) {
    const id = crypto.randomUUID();
    update(toasts => [...toasts, { id, type, message, duration }]);
    if (duration > 0) {
      setTimeout(() => dismiss(id), duration);
    }
    return id;
  }

  function dismiss(id: string) {
    update(toasts => toasts.filter(t => t.id !== id));
  }

  return {
    subscribe,
    success: (msg: string, duration?: number) => add('success', msg, duration),
    error:   (msg: string, duration?: number) => add('error',   msg, duration ?? 6000),
    info:    (msg: string, duration?: number) => add('info',    msg, duration),
    warning: (msg: string, duration?: number) => add('warning', msg, duration),
    dismiss,
  };
}

export const toast = createToastStore();
