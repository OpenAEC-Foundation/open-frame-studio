import { writable } from "svelte/store";

let nextId = 0;

export const toasts = writable([]);

function add(message, type = "info", duration = 4000) {
  const id = nextId++;
  toasts.update((t) => [...t, { id, message, type }]);
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
}

export function dismiss(id) {
  toasts.update((t) => t.filter((x) => x.id !== id));
}

export const toast = {
  success: (msg) => add(msg, "success"),
  error: (msg) => add(msg, "error", 6000),
  info: (msg) => add(msg, "info"),
  warning: (msg) => add(msg, "warning", 5000),
};
