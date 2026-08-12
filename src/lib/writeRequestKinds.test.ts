import { describe, expect, it } from "vitest";

import contract from "../../shared/write-request-kinds.json";
import { WRITE_REQUEST_KINDS } from "./writeRequestKinds";

describe("parity do contrato WriteRequest (Rust ↔ TS)", () => {
  it("lista TS bate com shared/write-request-kinds.json", () => {
    // O lado Rust valida o mesmo JSON contra o enum `WriteRequest`
    // (src-tauri/src/domain/operation.rs) — juntos, os dois testes garantem
    // que o contrato não deriva silenciosamente.
    expect([...WRITE_REQUEST_KINDS]).toEqual(contract);
  });
});
