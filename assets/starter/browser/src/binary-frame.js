const MAGIC = new TextEncoder().encode("RBK1");
const HEADER_BYTES = 8;
const MAX_METADATA_BYTES = 4096;
const MAX_PAYLOAD_BYTES = 64 * 1024;
const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
const idPattern = /^[A-Za-z0-9][A-Za-z0-9_.:-]{7,127}$/u;
const metadataKeys = [
  "type",
  "authorityGeneration",
  "grantId",
  "principalId",
  "sequence",
  "transferId",
  "fileId",
  "offset",
  "payloadLength",
].sort();

function validateMetadata(metadata) {
  if (!metadata || typeof metadata !== "object" || Array.isArray(metadata)) {
    throw new TypeError("binary metadata must be an object");
  }
  const keys = Object.keys(metadata).sort();
  if (keys.length !== metadataKeys.length || keys.some((key, index) => key !== metadataKeys[index])) {
    throw new TypeError("binary metadata contains unknown or missing fields");
  }
  if (metadata.type !== "example.bulk.chunk.v1") throw new TypeError("unsupported binary frame type");
  for (const key of ["authorityGeneration", "grantId", "principalId", "transferId", "fileId"]) {
    if (typeof metadata[key] !== "string" || !idPattern.test(metadata[key])) {
      throw new TypeError(`invalid ${key}`);
    }
  }
  for (const key of ["sequence", "offset", "payloadLength"]) {
    if (!Number.isSafeInteger(metadata[key]) || metadata[key] < 0) throw new TypeError(`invalid ${key}`);
  }
  if (metadata.payloadLength < 1 || metadata.payloadLength > MAX_PAYLOAD_BYTES) {
    throw new TypeError("invalid payloadLength");
  }
  return metadata;
}

export function encodeBinaryFrame(metadata, payload, negotiatedMaximum) {
  const body = payload instanceof Uint8Array ? payload : new Uint8Array(payload);
  if (body.byteLength < 1 || body.byteLength > MAX_PAYLOAD_BYTES) throw new TypeError("invalid payload size");
  const normalized = { ...metadata, payloadLength: body.byteLength };
  validateMetadata(normalized);
  const metadataBytes = encoder.encode(JSON.stringify(normalized));
  if (metadataBytes.byteLength < 2 || metadataBytes.byteLength > MAX_METADATA_BYTES) {
    throw new TypeError("invalid metadata size");
  }
  const total = HEADER_BYTES + metadataBytes.byteLength + body.byteLength;
  if (!Number.isSafeInteger(negotiatedMaximum) || total > negotiatedMaximum) {
    throw new TypeError("frame exceeds negotiated maximum");
  }
  const frame = new Uint8Array(total);
  frame.set(MAGIC, 0);
  new DataView(frame.buffer).setUint32(4, metadataBytes.byteLength);
  frame.set(metadataBytes, HEADER_BYTES);
  frame.set(body, HEADER_BYTES + metadataBytes.byteLength);
  return frame;
}

export function decodeBinaryFrame(input, negotiatedMaximum) {
  const frame = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (frame.byteLength < HEADER_BYTES + 3 || frame.byteLength > negotiatedMaximum) {
    throw new TypeError("invalid binary frame size");
  }
  if (!MAGIC.every((byte, index) => frame[index] === byte)) throw new TypeError("invalid binary frame magic");
  const metadataLength = new DataView(frame.buffer, frame.byteOffset, frame.byteLength).getUint32(4);
  if (metadataLength < 2 || metadataLength > MAX_METADATA_BYTES) throw new TypeError("invalid metadata size");
  const payloadOffset = HEADER_BYTES + metadataLength;
  if (payloadOffset >= frame.byteLength) throw new TypeError("missing binary payload");
  const metadata = validateMetadata(JSON.parse(decoder.decode(frame.subarray(HEADER_BYTES, payloadOffset))));
  const payload = frame.slice(payloadOffset);
  if (payload.byteLength !== metadata.payloadLength || payload.byteLength > MAX_PAYLOAD_BYTES) {
    throw new TypeError("binary payload length mismatch");
  }
  return { metadata, payload };
}

export async function waitForCapacity(channel, {
  highWatermark = 1024 * 1024,
  lowWatermark = 512 * 1024,
  timeoutMs = 15_000,
} = {}) {
  if (channel.readyState !== "open") throw new Error("bulk channel is not open");
  channel.bufferedAmountLowThreshold = lowWatermark;
  if (channel.bufferedAmount <= highWatermark) return;
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => finish(new Error("bulk backpressure did not clear")), timeoutMs);
    const onLow = () => finish();
    const onClosed = () => finish(new Error("bulk channel closed during backpressure"));
    const finish = (error) => {
      clearTimeout(timer);
      channel.removeEventListener("bufferedamountlow", onLow);
      channel.removeEventListener("close", onClosed);
      channel.removeEventListener("error", onClosed);
      if (error) reject(error);
      else resolve();
    };
    channel.addEventListener("bufferedamountlow", onLow, { once: true });
    channel.addEventListener("close", onClosed, { once: true });
    channel.addEventListener("error", onClosed, { once: true });
    if (channel.bufferedAmount <= lowWatermark) finish();
  });
}
