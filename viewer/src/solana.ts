// Solana integration for the SSP operations console.
//
// Reads the on-chain registry, regime state and reserve vault for a pool and
// decodes them into plain objects the console can render. The PDA seeds mirror
// programs/ssp-registry/src/state.rs exactly; if one side moves, the other must
// move with it.

import { Connection, PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
);

export type Regime = "expand" | "watch" | "hedge" | "backstop";

const REGIMES: Regime[] = ["expand", "watch", "hedge", "backstop"];

export function poolPda(baseMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), baseMint.toBuffer()],
    PROGRAM_ID
  );
}

export function regimePda(pool: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("regime"), pool.toBuffer()],
    PROGRAM_ID
  );
}

export function reservePda(baseMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("reserve"), baseMint.toBuffer()],
    PROGRAM_ID
  );
}

export function attestationPda(
  pool: PublicKey,
  agent: PublicKey,
  epoch: bigint
): [PublicKey, number] {
  const epochLe = Buffer.alloc(8);
  epochLe.writeBigUInt64LE(epoch);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("attest"), pool.toBuffer(), agent.toBuffer(), epochLe],
    PROGRAM_ID
  );
}

export interface OnChainRegime {
  epoch: bigint;
  outcome: Regime;
  polarityBps: number;
  stressBps: number;
  stabilityIndexBps: number;
  settledAt: bigint;
}

export interface OnChainReserve {
  free: bigint;
  deployed: bigint;
}

// Account layouts, discriminator first. Offsets are positional because Borsh
// serialises sequentially with no padding.
const REGIME_EPOCH = 8;
const REGIME_OUTCOME = 16;
const REGIME_POLARITY = 17;
const REGIME_STRESS = 21;
const REGIME_INDEX = 23;
const REGIME_SETTLED_AT = 25;

const RESERVE_TOTAL = 8;
const RESERVE_DEPLOYED = 16;

export function decodeRegime(data: Buffer): OnChainRegime {
  const outcome = REGIMES[data[REGIME_OUTCOME]];
  if (!outcome) {
    throw new Error(`unknown regime discriminant ${data[REGIME_OUTCOME]}`);
  }
  return {
    epoch: data.readBigUInt64LE(REGIME_EPOCH),
    outcome,
    polarityBps: data.readInt32LE(REGIME_POLARITY),
    stressBps: data.readUInt16LE(REGIME_STRESS),
    stabilityIndexBps: data.readUInt16LE(REGIME_INDEX),
    settledAt: data.readBigUInt64LE(REGIME_SETTLED_AT),
  };
}

export function decodeReserve(data: Buffer): OnChainReserve {
  const total = data.readBigUInt64LE(RESERVE_TOTAL);
  const deployed = data.readBigUInt64LE(RESERVE_DEPLOYED);
  return { free: total - deployed, deployed };
}

export async function fetchPoolSnapshot(
  connection: Connection,
  baseMint: PublicKey
): Promise<{ regime: OnChainRegime; reserve: OnChainReserve }> {
  const [pool] = poolPda(baseMint);
  const [regimeAddress] = regimePda(pool);
  const [reserveAddress] = reservePda(baseMint);

  const [regimeInfo, reserveInfo] = await connection.getMultipleAccountsInfo([
    regimeAddress,
    reserveAddress,
  ]);
  if (!regimeInfo) throw new Error("regime account not found");
  if (!reserveInfo) throw new Error("reserve vault not found");

  return {
    regime: decodeRegime(Buffer.from(regimeInfo.data)),
    reserve: decodeReserve(Buffer.from(reserveInfo.data)),
  };
}

export function bpsToPercent(bps: number): string {
  return `${(bps / 100).toFixed(2)}%`;
}
