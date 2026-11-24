import { writable } from "svelte/store";

export const docsDialog = writable(null);
let pending;

export function chooseDocsFile(options) {
  if (pending) finishDocsDialog();
  return new Promise((resolve) => {
    pending = resolve;
    docsDialog.set(options);
  });
}

export function finishDocsDialog(value = null) {
  const resolve = pending;
  pending = null;
  docsDialog.set(null);
  resolve?.(value);
}
