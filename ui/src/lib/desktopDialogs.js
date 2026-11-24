let workspace;

export function setDialogWorkspace(path) {
  workspace = path;
}

export function workspaceDialogOptions(options, directory = workspace) {
  if (!directory) return options;
  const filename = options.defaultPath?.split(/[\\/]/).pop();
  const defaultPath = filename && ![".", ".."].includes(filename) ? directory + "/" + filename : directory;
  return { ...options, defaultPath };
}

export async function desktopDialogs() {
  const { open, save } = await import("@tauri-apps/plugin-dialog");
  return {
    open: (options) => open(workspaceDialogOptions(options)),
    save: (options) => save(workspaceDialogOptions(options)),
  };
}
