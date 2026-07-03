import type { AppInfo, CommitDto, RepoInfo, RepoStatusDto } from "@/types";

export const MOCK_APP_INFO: AppInfo = {
  name: "Trilho",
  version: "0.1.0",
};

export const MOCK_REPO: RepoInfo = {
  path: "C:\\Projetos\\Trilho",
  branch: "master",
  upstream: null,
  isDetached: false,
  hasCommits: true,
};

export const MOCK_COMMITS: CommitDto[] = [
  {
    id: "9f3a1c2e5b7d0a4f6e8c1b2d3a4f5e6c7d8b9a0f",
    shortId: "9f3a1c2",
    summary: "feat: estrutura inicial do Trilho (M0)",
    authorName: "Você",
    authoredAt: "2026-07-02T14:10:00-03:00",
    isLocalOnly: true,
  },
  {
    id: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e",
    shortId: "1b2c3d4",
    summary: "chore: configuração de tema claro/escuro",
    authorName: "Você",
    authoredAt: "2026-07-02T11:05:00-03:00",
    isLocalOnly: false,
  },
];

export const MOCK_STATUS: RepoStatusDto = {
  staged: [],
  unstaged: [
    { path: "src/App.tsx", kind: "modified", staged: false },
  ],
  untracked: [],
};
