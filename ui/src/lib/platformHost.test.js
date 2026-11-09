import test from "node:test";
import assert from "node:assert/strict";
import {
  decodeOfsBytes,
  ensureOfsName,
  getOpenAecHost,
  listHostProjects,
  openHostProjectIfCurrent,
  readHostProject,
  writeHostProject,
} from "./platformHost.js";

const bytes = (text) => new TextEncoder().encode(text);

test("opens UTF-8 OFS bytes and preserves the host version and read-only flag", async () => {
  const json = '{"projectInfo":{"name":"René"}}';
  const host = {
    open: async (id, version) => {
      assert.equal(id, "doc-1");
      assert.equal(version, 3);
      const data = bytes(json);
      return {
        id, name: "renovatie.ofs", version: 3, readOnly: true,
        bytes: data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength),
      };
    },
    save: async () => { throw new Error("unexpected save"); },
  };
  assert.equal(getOpenAecHost({ openaecHost: host }), host);
  assert.equal(getOpenAecHost({ openaecHost: {} }), null);
  assert.deepEqual(await readHostProject(host, "doc-1", 3), {
    json,
    ref: { id: "doc-1", name: "renovatie.ofs", version: 3, readOnly: true },
  });
});

test("save sends expectedVersion and follows a host-created conflict copy", async () => {
  let received;
  const host = {
    save: async (content, options) => {
      received = { json: decodeOfsBytes(content), options };
      return { id: "conflict-copy", name: "project (conflict).ofs", version: 1 };
    },
  };
  const ref = await writeHostProject(host, '{"formatVersion":"1.3"}', {
    current: { id: "doc-1", name: "project.ofs", version: "v7", readOnly: false },
  });
  assert.deepEqual(received, {
    json: '{"formatVersion":"1.3"}',
    options: { id: "doc-1", expectedVersion: "v7" },
  });
  assert.deepEqual(ref, {
    id: "conflict-copy", name: "project (conflict).ofs", version: 1, readOnly: false,
  });
});

test("read-only documents require Save As and new names have an OFS extension", async () => {
  let options;
  const host = {
    save: async (_content, value) => {
      options = value;
      return { id: "new", name: value.name, version: 1 };
    },
  };
  const current = { id: "old", name: "old.ofs", version: 2, readOnly: true };
  await assert.rejects(writeHostProject(host, "{}", { current }), /alleen-lezen/);
  await assert.rejects(writeHostProject(host, "{}", {
    current: { id: "old", name: "old.ofs", readOnly: false },
  }), /geen versie/);
  const saved = await writeHostProject(host, "{}", { current, name: "kopie" });
  assert.deepEqual(options, { name: "kopie.ofs" });
  assert.equal(saved.name, "kopie.ofs");
  assert.equal(ensureOfsName("Kozijn.OFS"), "Kozijn.OFS");
});

test("only OFS files are listed and malformed host bytes are rejected", async () => {
  const files = await listHostProjects({ list: async () => [
    { id: "1", name: "a.ofs" }, { id: "2", name: "notitie.txt" }, { name: "bad.ofs" },
  ] });
  assert.deepEqual(files, [{ id: "1", name: "a.ofs" }]);
  assert.throws(() => decodeOfsBytes("not bytes"), /geen leesbare/);
  assert.throws(() => decodeOfsBytes(Uint8Array.of(0xff)), /encoded data/);
});

test("late Docs open never applies after a local edit or project switch", async () => {
  for (const change of [{ epoch: 0, revision: 1 }, { epoch: 1, revision: 0 }]) {
    let finishOpen;
    const response = new Promise((resolve) => { finishOpen = resolve; });
    let state = { epoch: 0, revision: 0 };
    const applied = [];
    const pending = openHostProjectIfCurrent(
      { open: () => response }, "doc-late", undefined,
      {
        captureState: () => ({ ...state }),
        isStateCurrent: (token) => token.epoch === state.epoch && token.revision === state.revision,
        apply: (file) => { applied.push(file); },
      },
    );
    state = change;
    finishOpen({ id: "doc-late", name: "late.ofs", version: 1, bytes: bytes("{}") });
    assert.equal(await pending, false);
    assert.deepEqual(applied, []);
  }
});

test("Docs open applies when the local project stayed unchanged", async () => {
  const applied = [];
  const opened = await openHostProjectIfCurrent(
    { open: async () => ({ id: "doc", name: "current.ofs", version: 2, bytes: bytes("{\"ok\":true}") }) },
    "doc", undefined,
    { captureState: () => 42, isStateCurrent: (token) => token === 42, apply: (file) => { applied.push(file); } },
  );
  assert.equal(opened, true);
  assert.equal(applied[0].json, '{"ok":true}');
});

test("late Docs open also ignores a core mutation before the dirty flag updates", async () => {
  let finishOpen;
  const response = new Promise((resolve) => { finishOpen = resolve; });
  let signalOpen;
  const openStarted = new Promise((resolve) => { signalOpen = resolve; });
  let coreSnapshot = "before";
  let applied = false;
  const pending = openHostProjectIfCurrent(
    { open: () => { signalOpen(); return response; } }, "doc-late", undefined,
    {
      captureState: async () => coreSnapshot,
      isStateCurrent: async (snapshot) => snapshot === coreSnapshot,
      apply: () => { applied = true; },
    },
  );
  await openStarted;
  coreSnapshot = "after mutation";
  finishOpen({ id: "doc-late", name: "late.ofs", version: 1, bytes: bytes("{}") });
  assert.equal(await pending, false);
  assert.equal(applied, false);
});
