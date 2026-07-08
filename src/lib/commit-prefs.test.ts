import { afterEach, describe, expect, it } from "vitest";

import {
  getPrefillCommitFileList,
  setPrefillCommitFileList,
} from "@/lib/commit-prefs";

describe("commit-prefs", () => {
  afterEach(() => {
    localStorage.removeItem("trilho.commit.prefillFileList");
  });

  it("pré-preenche por padrão", () => {
    expect(getPrefillCommitFileList()).toBe(true);
  });

  it("persiste opt-out", () => {
    setPrefillCommitFileList(false);
    expect(getPrefillCommitFileList()).toBe(false);
    setPrefillCommitFileList(true);
    expect(getPrefillCommitFileList()).toBe(true);
  });
});
