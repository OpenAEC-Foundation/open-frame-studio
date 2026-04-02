/**
 * Browser file dialogs — replacements for Tauri's native dialogs.
 */

/**
 * Open a file picker and return the file content as text.
 * @param {Object} opts - { filters: [{ extensions: ['ofs'] }] }
 * @returns {Promise<{ path: string, content: string } | null>}
 */
export function openFile(opts = {}) {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    if (opts.filters?.[0]?.extensions) {
      input.accept = opts.filters[0].extensions.map((e) => `.${e}`).join(",");
    }
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      const content = await file.text();
      resolve({ path: file.name, content });
    };
    input.click();
  });
}

/**
 * Download content as a file.
 * @param {string} filename
 * @param {string|Blob|Uint8Array} content
 * @param {string} mimeType
 */
export function saveFile(filename, content, mimeType = "application/octet-stream") {
  const blob = content instanceof Blob
    ? content
    : new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
