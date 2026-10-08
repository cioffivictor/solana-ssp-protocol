import { decodeRegime, decodeReserve, bpsToPercent } from "./solana.js";

function assert(cond: boolean, msg: string): void {
  if (!cond) throw new Error(`assertion failed: ${msg}`);
}

// RegimeState: discriminator 8 bytes, then epoch, outcome, polarity, stress,
// index, settled_at, bump.
const regime = Buffer.alloc(34);
regime.writeBigUInt64LE(7n, 8);
regime[16] = 3;
regime.writeInt32LE(-2400, 17);
regime.writeUInt16LE(3800, 21);
regime.writeUInt16LE(2400, 23);
regime.writeBigUInt64LE(1_000_000n, 25);

const decoded = decodeRegime(regime);
assert(decoded.epoch === 7n, "epoch round-trips");
assert(decoded.outcome === "backstop", "outcome maps to token");
assert(decoded.polarityBps === -2400, "polarity is signed");
assert(decoded.stressBps === 3800, "stress round-trips");
assert(decoded.settledAt === 1_000_000n, "slot round-trips");

const reserve = Buffer.alloc(25);
reserve.writeBigUInt64LE(50_000n, 8);
reserve.writeBigUInt64LE(12_500n, 16);
const r = decodeReserve(reserve);
assert(r.free === 37_500n, "free balance is reserve minus deployed");

assert(bpsToPercent(2400) === "24.00%", "bps renders as percent");

console.log("solana.ts: all assertions passed");
