import { describe, expect, it, vi } from "vitest";

import { refsMenuItems, type RefsMenuAcoes, type RefsMenuState } from "./refsMenu";

function acoes(over: Partial<RefsMenuAcoes> = {}): RefsMenuAcoes {
  return {
    busy: false,
    remoteBranches: [],
    onSwitchLocal: vi.fn(),
    onSwitchRemote: vi.fn(),
    onStashApply: vi.fn(),
    onStashPop: vi.fn(),
    onStashDrop: vi.fn(),
    onTagSelect: vi.fn(),
    onTagDelete: vi.fn(),
    ...over,
  };
}

const localMenu: RefsMenuState = {
  kind: "local",
  branch: "feature-1",
  x: 0,
  y: 0,
  active: false,
};

describe("refsMenuItems", () => {
  it("sem menu não devolve item", () => {
    expect(refsMenuItems(null, acoes())).toEqual([]);
  });

  it("omite remoções quando os handlers não vêm", () => {
    const ids = refsMenuItems(localMenu, acoes()).map((i) => i.id);
    expect(ids).toEqual(["checkout"]);
  });

  it("desabilita checkout da branch já ativa", () => {
    const menu = { ...localMenu, active: true };
    const [checkout] = refsMenuItems(menu, acoes());
    expect(checkout?.disabled).toBe(true);
    expect(checkout?.primary).toBeFalsy();
  });

  it("busy desabilita tudo que escreve", () => {
    const items = refsMenuItems(
      localMenu,
      acoes({ busy: true, onDeleteLocal: vi.fn() })
    );
    expect(items.every((i) => i.disabled)).toBe(true);
  });

  it("branch local oferece remoção nos remotos onde ela existe", () => {
    const items = refsMenuItems(
      localMenu,
      acoes({
        onDeleteRemote: vi.fn(),
        remoteBranches: [
          { remote: "origin", branch: "feature-1" },
          { remote: "upstream", branch: "feature-1" },
          { remote: "origin", branch: "outra" },
        ],
      })
    );
    expect(items.map((i) => i.id)).toEqual([
      "checkout",
      "delete-remote-origin",
      "delete-remote-upstream",
    ]);
  });

  it("branch sem par remoto cai em origin", () => {
    const items = refsMenuItems(
      localMenu,
      acoes({
        onDeleteRemote: vi.fn(),
        remoteBranches: [{ remote: "origin", branch: "outra" }],
      })
    );
    expect(items.map((i) => i.id)).toContain("delete-remote-origin");
  });

  it("checkout de remota com par local usa o switch local", () => {
    const onSwitchLocal = vi.fn();
    const onSwitchRemote = vi.fn();
    const menu: RefsMenuState = {
      kind: "remote",
      remote: "origin",
      branch: "feature-1",
      x: 0,
      y: 0,
      active: false,
      hasLocal: true,
    };
    const [checkout] = refsMenuItems(menu, acoes({ onSwitchLocal, onSwitchRemote }));
    checkout?.onSelect?.();
    expect(onSwitchLocal).toHaveBeenCalledWith("feature-1");
    expect(onSwitchRemote).not.toHaveBeenCalled();
  });

  it("stash expõe aplicar, pop e excluir", () => {
    const menu: RefsMenuState = {
      kind: "stash",
      index: 2,
      reference: "stash@{2}",
      message: "wip",
      x: 0,
      y: 0,
    };
    const onStashPop = vi.fn();
    const items = refsMenuItems(menu, acoes({ onStashPop }));
    expect(items.map((i) => i.id)).toEqual(["apply", "pop", "drop"]);
    items[1]?.onSelect?.();
    expect(onStashPop).toHaveBeenCalledWith(2);
  });

  it("tag navega para o commit e permite excluir", () => {
    const menu: RefsMenuState = {
      kind: "tag",
      name: "v1.0",
      commitId: "abc123",
      x: 0,
      y: 0,
    };
    const onTagSelect = vi.fn();
    const items = refsMenuItems(menu, acoes({ onTagSelect }));
    expect(items.map((i) => i.id)).toEqual(["goto", "delete-tag"]);
    items[0]?.onSelect?.();
    expect(onTagSelect).toHaveBeenCalledWith("abc123");
  });
});
