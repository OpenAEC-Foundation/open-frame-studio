import { open, realpath } from "node:fs/promises";
import { basename, extname, isAbsolute, relative, resolve, sep } from "node:path";
import { constants } from "node:fs";
import { FrameToolError } from "../../ui/src/lib/frameTools.js";

const MAX_PROJECT_BYTES = 32 * 1024 * 1024;

// The operator chooses the document. Requests cannot set a filename or select a different folder.
export function createFrameFileContext(filePath, { root } = {}) {
  const path = filePath && resolve(filePath);
  return async () => {
    if (!path) throw new FrameToolError("geen_project");
    if (extname(path).toLowerCase() !== ".ofs") throw new FrameToolError("ongeldig_bestand");
    const file = await open(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0) | (constants.O_NONBLOCK ?? 0));
    try {
      if (root) {
        const target = process.platform === "linux" ? `/proc/self/fd/${file.fd}` : path;
        const suffix = relative(await realpath(root), await realpath(target));
        if (!suffix || suffix === ".." || suffix.startsWith(".." + sep) || isAbsolute(suffix)) {
          throw new FrameToolError("ongeldig_bestand");
        }
      }
      const stat = await file.stat();
      if (!stat.isFile() || !stat.size || stat.size > MAX_PROJECT_BYTES) throw new FrameToolError("ongeldig_bestand");
      // Read at most the measured size even if an editor replaces or extends the file.
      const bytes = Buffer.alloc(stat.size);
      let offset = 0;
      while (offset < bytes.length) {
        const { bytesRead } = await file.read(bytes, offset, bytes.length - offset, offset);
        if (!bytesRead) throw new FrameToolError("project_gewijzigd");
        offset += bytesRead;
      }
      const now = await file.stat();
      if (now.size !== stat.size || now.mtimeMs !== stat.mtimeMs) throw new FrameToolError("project_gewijzigd");
      const project = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
      const name = basename(path);
      return { project, hostFileRef: { id: name, name, version: stat.mtime.toISOString(), readOnly: true } };
    } finally {
      await file.close();
    }
  };
}
