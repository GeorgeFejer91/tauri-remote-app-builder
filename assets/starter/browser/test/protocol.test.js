import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import { decodeBinaryFrame, encodeBinaryFrame } from "../src/binary-frame.js";
import {
  RevisionStore,
  decodeEnvelope,
  isNewerSequence,
  makeCommandEnvelope,
  validateCommand,
  validateProductAction,
} from "../src/protocol.js";

const fixtures = JSON.parse(
  fs.readFileSync(new URL("../../contracts/command-fixtures.json", import.meta.url), "utf8"),
);

test("validates the shared command fixture and rejects extra authority", () => {
  const { authorityGeneration: _generation, ...command } = fixtures.validCommand;
  assert.equal(validateCommand(command), command);
  assert.throws(
    () => validateCommand({ ...command, nativePath: "/forbidden" }),
    /unknown or missing/,
  );
});

test("rejects unknown or open-ended product actions", () => {
  assert.throws(() => validateProductAction({ kind: "invoke", payload: {} }), /unknown product action/);
  assert.throws(
    () => validateProductAction({ kind: "set_level", value: 37, nativePath: "/forbidden" }),
    /unknown or missing/,
  );
  assert.throws(() => validateProductAction({ kind: "set_level", value: 101 }), /non-negative/);
});

test("round trips a bounded control envelope", () => {
  const { authorityGeneration, ...command } = fixtures.validCommand;
  const encoded = makeCommandEnvelope({
    protocol: "example.remote",
    authorityGeneration,
    senderId: "browser_demo_01",
    senderEpoch: 3,
    sequence: 41,
    command,
  });
  assert.deepEqual(decodeEnvelope(encoded), JSON.parse(encoded));
  assert.equal(decodeEnvelope("{"), undefined);
});

test("uses serial arithmetic across a 32-bit wrap", () => {
  assert.equal(isNewerSequence(0, 0xffff_ffff), true);
  assert.equal(isNewerSequence(0xffff_ffff, 0), false);
  assert.equal(isNewerSequence(10, 10), false);
});

test("accepts only newer revisions and resets on authority generation", () => {
  const store = new RevisionStore();
  assert.equal(store.accept(fixtures.validSnapshot), true);
  assert.equal(store.accept(fixtures.validSnapshot), false);
  assert.equal(
    store.accept({ ...fixtures.validSnapshot, authorityGeneration: "authority_demo_02", revision: 0 }),
    true,
  );
  assert.equal(store.current().revision, 0);
  assert.throws(
    () => store.accept({ ...fixtures.validSnapshot, state: { active: false, level: 37, extra: true } }),
    /unknown or missing/,
  );
});

test("round trips strict binary metadata and payload", () => {
  const payload = new TextEncoder().encode("test");
  const frame = encodeBinaryFrame(fixtures.validBinaryMetadata, payload, 65_536);
  const decoded = decodeBinaryFrame(frame, 65_536);
  assert.deepEqual(decoded.metadata, fixtures.validBinaryMetadata);
  assert.deepEqual(decoded.payload, payload);
});

test("rejects binary metadata extension and negotiated overflow", () => {
  const payload = new TextEncoder().encode("test");
  assert.throws(
    () => encodeBinaryFrame({ ...fixtures.validBinaryMetadata, nativePath: "/forbidden" }, payload, 65_536),
    /unknown or missing/,
  );
  assert.throws(
    () => encodeBinaryFrame(fixtures.validBinaryMetadata, payload, 16),
    /negotiated maximum/,
  );
});
