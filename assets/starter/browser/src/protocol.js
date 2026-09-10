export const CONTROL_MAX_BYTES = 16_384;
export const STATE_MAX_BYTES = 8_192;

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
const tokenPattern = /^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$/u;
const controlTypes = new Set([
  "hello",
  "ready",
  "command",
  "applied",
  "rejected",
  "snapshot_request",
  "snapshot",
  "revoked",
  "bye",
  "error",
]);
const stateTypes = new Set(["state"]);

function fail(message) {
  throw new TypeError(message);
}

function isPlainRecord(value) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function exactKeys(value, expected, label) {
  if (!isPlainRecord(value)) fail(`${label} must be an object`);
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (actual.length !== wanted.length || actual.some((key, index) => key !== wanted[index])) {
    fail(`${label} contains unknown or missing fields`);
  }
}

function token(value, label, minimum = 1) {
  if (typeof value !== "string" || value.length < minimum || !tokenPattern.test(value)) {
    fail(`${label} is not a bounded protocol token`);
  }
  return value;
}

function safeInteger(value, label, maximum = Number.MAX_SAFE_INTEGER) {
  if (!Number.isSafeInteger(value) || value < 0 || value > maximum) {
    fail(`${label} must be a non-negative safe integer`);
  }
  return value;
}

function jsonValue(value, label = "value", depth = 0) {
  if (depth > 8) fail(`${label} is too deeply nested`);
  if (value === null || typeof value === "string" || typeof value === "boolean") return;
  if (typeof value === "number") {
    if (!Number.isFinite(value)) fail(`${label} contains a non-finite number`);
    return;
  }
  if (Array.isArray(value)) {
    if (value.length > 256) fail(`${label} has too many items`);
    value.forEach((entry, index) => jsonValue(entry, `${label}[${index}]`, depth + 1));
    return;
  }
  if (!isPlainRecord(value)) fail(`${label} is not JSON-compatible`);
  const entries = Object.entries(value);
  if (entries.length > 128) fail(`${label} has too many fields`);
  for (const [key, entry] of entries) {
    if (!key || key.length > 96 || ["__proto__", "prototype", "constructor"].includes(key)) {
      fail(`${label} has an unsafe field name`);
    }
    jsonValue(entry, `${label}.${key}`, depth + 1);
  }
}

function bytes(value) {
  if (typeof value === "string") return encoder.encode(value);
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (ArrayBuffer.isView(value)) return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  fail("wire data must be UTF-8 text or bytes");
}

export function decodeEnvelope(value, lane = "control") {
  try {
    const input = bytes(value);
    const maximum = lane === "state" ? STATE_MAX_BYTES : CONTROL_MAX_BYTES;
    if (input.byteLength === 0 || input.byteLength > maximum) return undefined;
    const envelope = JSON.parse(decoder.decode(input));
    validateEnvelope(envelope, lane);
    return envelope;
  } catch {
    return undefined;
  }
}

export function validateEnvelope(envelope, lane = "control") {
  exactKeys(
    envelope,
    ["protocol", "version", "type", "authorityGeneration", "senderId", "senderEpoch", "sequence", "body"],
    "envelope",
  );
  token(envelope.protocol, "protocol");
  if (envelope.version !== 1) fail("unsupported protocol version");
  token(envelope.authorityGeneration, "authorityGeneration", 8);
  token(envelope.senderId, "senderId", 8);
  safeInteger(envelope.senderEpoch, "senderEpoch", 0xffff_ffff);
  safeInteger(envelope.sequence, "sequence", 0xffff_ffff);
  const allowed = lane === "state" ? stateTypes : controlTypes;
  if (!allowed.has(envelope.type)) fail(`unknown ${lane} message type`);
  jsonValue(envelope.body, "body");
  if (envelope.type === "command") validateCommand(envelope.body);
  if (envelope.type === "applied" || envelope.type === "rejected") validateOutcome(envelope.body);
  return envelope;
}

export function validateCommand(command) {
  exactKeys(
    command,
    ["commandId", "grantId", "principalId", "scope", "expectedRevision", "action"],
    "command",
  );
  token(command.commandId, "commandId", 8);
  token(command.grantId, "grantId", 8);
  token(command.principalId, "principalId", 8);
  token(command.scope, "scope");
  if (command.expectedRevision !== null) safeInteger(command.expectedRevision, "expectedRevision");
  validateProductAction(command.action);
  if (command.scope !== "example.control") fail("action scope does not match the product schema");
  return command;
}

export function validateProductAction(action) {
  if (!isPlainRecord(action)) fail("action must be an object");
  if (action.kind === "set_level") {
    exactKeys(action, ["kind", "value"], "set_level action");
    safeInteger(action.value, "set_level.value", 100);
    return action;
  }
  if (action.kind === "activate" || action.kind === "deactivate") {
    exactKeys(action, ["kind"], `${action.kind} action`);
    return action;
  }
  fail("unknown product action");
}

export function validateProductState(state) {
  exactKeys(state, ["active", "level"], "product state");
  if (typeof state.active !== "boolean") fail("product state.active must be boolean");
  safeInteger(state.level, "product state.level", 100);
  return state;
}

export function validateOutcome(outcome) {
  exactKeys(outcome, ["commandId", "ok", "revision", "result", "error"], "outcome");
  token(outcome.commandId, "commandId", 8);
  if (typeof outcome.ok !== "boolean") fail("outcome.ok must be boolean");
  safeInteger(outcome.revision, "revision");
  if (outcome.ok) {
    if (outcome.error !== null) fail("an applied outcome cannot contain an error");
    validateProductState(outcome.result);
  } else {
    if (outcome.result !== null) fail("a rejected outcome cannot contain a result");
    token(outcome.error, "error");
  }
  return outcome;
}

export function makeCommandEnvelope({
  protocol,
  authorityGeneration,
  senderId,
  senderEpoch,
  sequence,
  command,
}) {
  validateCommand(command);
  const envelope = {
    protocol,
    version: 1,
    type: "command",
    authorityGeneration,
    senderId,
    senderEpoch,
    sequence,
    body: command,
  };
  validateEnvelope(envelope);
  const encoded = JSON.stringify(envelope);
  if (encoder.encode(encoded).byteLength > CONTROL_MAX_BYTES) fail("control message is too large");
  return encoded;
}

export function isNewerSequence(sequence, previous) {
  safeInteger(sequence, "sequence", 0xffff_ffff);
  if (previous === undefined || previous === null) return true;
  safeInteger(previous, "previous", 0xffff_ffff);
  const distance = (sequence - previous) >>> 0;
  return distance > 0 && distance < 0x8000_0000;
}

export class RevisionStore {
  #authorityGeneration;
  #revision = -1;
  #state;

  accept(snapshot) {
    exactKeys(snapshot, ["authorityGeneration", "revision", "state"], "snapshot");
    token(snapshot.authorityGeneration, "authorityGeneration", 8);
    safeInteger(snapshot.revision, "revision");
    validateProductState(snapshot.state);
    if (this.#authorityGeneration && snapshot.authorityGeneration !== this.#authorityGeneration) {
      this.#authorityGeneration = snapshot.authorityGeneration;
      this.#revision = -1;
      this.#state = undefined;
    }
    if (snapshot.revision <= this.#revision) return false;
    this.#authorityGeneration = snapshot.authorityGeneration;
    this.#revision = snapshot.revision;
    this.#state = structuredClone(snapshot.state);
    return true;
  }

  current() {
    if (!this.#authorityGeneration) return undefined;
    return {
      authorityGeneration: this.#authorityGeneration,
      revision: this.#revision,
      state: structuredClone(this.#state),
    };
  }
}
