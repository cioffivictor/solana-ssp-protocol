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
