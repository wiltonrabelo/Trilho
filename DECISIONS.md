# Decisões de implementação — Trilho

## M0

### shadcn/ui adiado para M1+
- **Decisão:** não instalar shadcn/ui no M0; **continua adiado no M1** (layout Tailwind puro).
- **Motivo:** componentes atuais são simples; shadcn entra quando o layout estabilizar (M1-b ou início M2).

### Auth RF-10 parcial (M1)
- **「Conectar」** reexecuta `fetch` para acionar o GCM; se o credential helper não estiver
  instalado/configurado, o usuário verá erro repetido — mensagem orienta `git fetch` no terminal
  uma vez. Aceitável no MVP; assistente OAuth completo fica pós-MVP.
- **Status:** não iniciado no M0.
- **Ação:** iniciar aquisição do certificado EV **antes/durante M1** (lead time de semanas — caminho crítico do MVP §5).

### Ícone
- **Decisão:** ícone aprovado = locomotiva em movimento (line-art, traços de velocidade).
- **Fonte:** `assets/trilho-icon-1024.png` (crop 1024×1024) → gerado via `npm run tauri icon`.
