# Decisões de implementação — Trilho

## M0

### shadcn/ui adiado para M1+
- **Decisão:** não instalar shadcn/ui no M0.
- **Motivo:** componentes atuais (ThemeToggle, lista de commits) são simples e Tailwind puro basta.
- **Quando:** instalar no início do M1, junto com repo picker e layout de 3 colunas (PLANO §8).

### Code signing (certificado EV)
- **Status:** não iniciado no M0.
- **Ação:** iniciar aquisição do certificado EV **antes/durante M1** (lead time de semanas — caminho crítico do MVP §5).

### Ícone
- **Decisão:** ícone aprovado = locomotiva em movimento (line-art, traços de velocidade).
- **Fonte:** `assets/trilho-icon-1024.png` (crop 1024×1024) → gerado via `npm run tauri icon`.
