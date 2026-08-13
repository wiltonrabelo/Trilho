//! Configuração comum de subprocessos.

use std::process::Command;

/// Impede a janela de console que o Windows abre para cada processo de console
/// iniciado por um app de janela: sem isso, todo `git` chamado pela UI pisca um
/// prompt na tela — dezenas deles só para abrir um repositório.
///
/// Não use com programas de interface gráfica: com a flag o `explorer` cai na
/// Área de Trabalho e o `git-bash` abre sem janela nenhuma.
pub fn sem_janela_de_console(comando: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        comando.creation_flags(CREATE_NO_WINDOW);
    }
    comando
}

#[cfg(test)]
mod tests {
    /// Guarda-corpo: um spawn novo de `git`/`ssh` sem o helper volta a piscar
    /// prompts na tela do usuário, e isso não aparece em nenhum outro teste.
    /// `explorer` e `git-bash` ficam de fora de propósito — são GUI.
    #[test]
    fn spawns_de_console_escondem_a_janela() {
        let fontes = [
            ("git_cli.rs", include_str!("git_cli.rs")),
            ("credential.rs", include_str!("credential.rs")),
            ("upstream.rs", include_str!("upstream.rs")),
            ("llm_credentials.rs", include_str!("llm_credentials.rs")),
            ("ssh_keys.rs", include_str!("ssh_keys.rs")),
            ("worktree_file.rs", include_str!("worktree_file.rs")),
        ];
        for (arquivo, fonte) in fontes {
            // Testes spawnam git para montar fixtures e já rodam em console.
            let producao = fonte.split("#[cfg(test)]").next().unwrap_or(fonte);
            let linhas: Vec<&str> = producao.lines().collect();
            for (indice, linha) in linhas.iter().enumerate() {
                let spawn = ["\"git\"", "\"ssh\"", "\"cmd\"", "\"where\"", "\"taskkill\""]
                    .iter()
                    .any(|bin| linha.contains(&format!("Command::new({bin})")));
                if !spawn {
                    continue;
                }
                // O helper pode envolver a chamada, abrir a linha ou vir logo
                // depois, conforme o comando seja montado em uma ou mais etapas.
                let inicio = indice.saturating_sub(1);
                let fim = (indice + 1).min(linhas.len() - 1);
                let coberto = linhas[inicio..=fim]
                    .iter()
                    .any(|l| l.contains("sem_janela_de_console"));
                assert!(
                    coberto,
                    "{arquivo}:{}: spawn de console sem sem_janela_de_console — {}",
                    indice + 1,
                    linha.trim()
                );
            }
        }
    }
}
