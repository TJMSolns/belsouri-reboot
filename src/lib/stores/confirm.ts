import { writable } from 'svelte/store';

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
  /** If set, user must type this text to enable the confirm button */
  requiredInput?: string;
}

interface ConfirmState {
  open: boolean;
  options: ConfirmOptions;
  resolve: ((value: string | boolean) => void) | null;
}

const DEFAULT_OPTIONS: ConfirmOptions = {
  title: 'Confirm',
  message: '',
  confirmLabel: 'Confirm',
  cancelLabel: 'Cancel',
  destructive: false,
};

const _state = writable<ConfirmState>({
  open: false,
  options: DEFAULT_OPTIONS,
  resolve: null,
});

export const confirmState = { subscribe: _state.subscribe };

/** Show a confirm dialog. Returns `true` if confirmed, `false` if cancelled. */
export function confirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise(resolve => {
    _state.set({ open: true, options: { ...DEFAULT_OPTIONS, ...options }, resolve: (v) => resolve(Boolean(v)) });
  });
}

/** Show a prompt dialog. Returns the entered string or `null` if cancelled. */
export function prompt(options: Omit<ConfirmOptions, 'destructive' | 'requiredInput'> & { placeholder?: string }): Promise<string | null> {
  return new Promise(resolve => {
    _state.set({
      open: true,
      options: { ...DEFAULT_OPTIONS, ...options, requiredInput: undefined },
      resolve: (v) => resolve(v === false ? null : String(v)),
    });
  });
}

export function _resolve(value: string | boolean) {
  _state.update(s => {
    s.resolve?.(value);
    return { open: false, options: DEFAULT_OPTIONS, resolve: null };
  });
}
