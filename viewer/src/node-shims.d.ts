// Minimal ambient declarations for the tiny slice of the Node.js runtime the
// viewer touches. QuorumForge's viewer is deliberately dependency-light: it
// pulls in neither `@types/node` nor any runtime packages. These declarations
// cover exactly what `cli.ts` uses and nothing more.

declare const process: {
  argv: string[];
  exitCode?: number;
  exit(code?: number): never;
  stdin: {
    on(event: "data", listener: (chunk: string | Uint8Array) => void): void;
    on(event: "end", listener: () => void): void;
    setEncoding(encoding: string): void;
    isTTY?: boolean;
  };
  stdout: { write(text: string): boolean };
  stderr: { write(text: string): boolean };
};

declare module "node:fs" {
  export function readFileSync(path: string, encoding: "utf8"): string;
  export function writeFileSync(path: string, data: string): void;
  export function existsSync(path: string): boolean;
}

declare module "node:process" {
  const p: typeof process;
  export default p;
}

declare module "node:assert/strict" {
  interface AssertStrict {
    (value: unknown, message?: string): asserts value;
    equal(actual: unknown, expected: unknown, message?: string): void;
    ok(value: unknown, message?: string): asserts value;
    throws(fn: () => unknown, expected?: RegExp | Error): void;
  }
  const assert: AssertStrict;
  export default assert;
}
