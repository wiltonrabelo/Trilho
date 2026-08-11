//! A-02 — autorização de uso único ligando preview RF-08 → execute.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::domain::{CloneRequest, WriteRequest};

/// Validade do token após o preview (confirmação humana típica).
const TTL: Duration = Duration::from_secs(5 * 60);
/// Limite de autorizações pendentes (evita crescimento ilimitado).
const MAX_PENDING: usize = 64;

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct OpAuthEntry<T: Clone> {
    pub repo_path: String,
    pub request: T,
    pub commands: Vec<String>,
    expires_at: Instant,
}

pub struct OpAuthStore<T: Clone> {
    inner: Mutex<HashMap<String, OpAuthEntry<T>>>,
}

pub type WriteAuthStore = OpAuthStore<WriteRequest>;
pub type CloneAuthStore = OpAuthStore<CloneRequest>;

impl<T: Clone> Default for OpAuthStore<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl<T: Clone> OpAuthStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emite token de uso único vinculado ao preview. Não emitir se `blocked`.
    pub fn issue(
        &self,
        repo_path: &str,
        request: &T,
        commands: &[String],
    ) -> Result<String, String> {
        let token = mint_token();
        let entry = OpAuthEntry {
            repo_path: repo_path.to_string(),
            request: request.clone(),
            commands: commands.to_vec(),
            expires_at: Instant::now() + TTL,
        };
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Estado de autorização indisponível.".to_string())?;
        purge_expired(&mut guard);
        while guard.len() >= MAX_PENDING {
            // Remove o mais antigo por expiração; se empatar, qualquer um.
            let victim = guard
                .iter()
                .min_by_key(|(_, e)| e.expires_at)
                .map(|(k, _)| k.clone());
            if let Some(k) = victim {
                guard.remove(&k);
            } else {
                break;
            }
        }
        guard.insert(token.clone(), entry);
        Ok(token)
    }

    /// Consome atomicamente. Falha se inexistente, expirado ou já usado.
    pub fn take(&self, token: &str) -> Result<OpAuthEntry<T>, String> {
        let token = token.trim();
        if token.is_empty() {
            return Err(
                "Confirmação inválida: falta autorização do preview (RF-08).".into(),
            );
        }
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Estado de autorização indisponível.".to_string())?;
        purge_expired(&mut guard);
        let entry = guard.remove(token).ok_or_else(|| {
            "Confirmação inválida ou já usada. Peça o preview novamente.".to_string()
        })?;
        if Instant::now() > entry.expires_at {
            return Err("Confirmação expirada. Peça o preview novamente.".into());
        }
        Ok(entry)
    }

    /// Reinsere um token **apenas** se nenhum efeito colateral Git ocorreu.
    /// Após `execute_*` (mesmo com erro), **não** restaurar — risco de retry
    /// sobre operação parcial. O FE deve pedir novo preview.
    #[allow(dead_code)]
    pub fn restore(&self, token: &str, entry: OpAuthEntry<T>) {
        let token = token.trim();
        if token.is_empty() {
            return;
        }
        if let Ok(mut guard) = self.inner.lock() {
            purge_expired(&mut guard);
            if Instant::now() <= entry.expires_at {
                guard.insert(token.to_string(), entry);
            }
        }
    }

    /// Descarta um token (cancelamento do diálogo).
    #[allow(dead_code)]
    pub fn revoke(&self, token: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(token.trim());
        }
    }
}

fn purge_expired<T: Clone>(map: &mut HashMap<String, OpAuthEntry<T>>) {
    let now = Instant::now();
    map.retain(|_, e| e.expires_at > now);
}

fn mint_token() -> String {
    // 128 bits de entropia via CSPRNG do SO + contador.
    let mut bytes = [0u8; 16];
    if fill_random(&mut bytes).is_err() {
        // Fallback fraco só se o SO falhar — ainda único por processo.
        let n = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        bytes = [
            (n >> 56) as u8,
            (n >> 48) as u8,
            (n >> 40) as u8,
            (n >> 32) as u8,
            (n >> 24) as u8,
            (n >> 16) as u8,
            (n >> 8) as u8,
            n as u8,
            (nanos >> 56) as u8,
            (nanos >> 48) as u8,
            (nanos >> 40) as u8,
            (nanos >> 32) as u8,
            (nanos >> 24) as u8,
            (nanos >> 16) as u8,
            (nanos >> 8) as u8,
            nanos as u8,
        ];
    } else {
        let _ = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fill_random(buf: &mut [u8]) -> Result<(), ()> {
    // Sem dependência direta: usa `std` + Windows BCrypt / Unix getrandom via
    // leitura de fonte do SO com API estável do Rust 1.79+ não disponível;
    // usamos o crate transitivo getrandom se linkado, senão File.
    #[cfg(windows)]
    {
        use std::ptr;
        #[link(name = "bcrypt")]
        extern "system" {
            fn BCryptGenRandom(
                h_algorithm: *mut core::ffi::c_void,
                pb_buffer: *mut u8,
                cb_buffer: u32,
                dw_flags: u32,
            ) -> i32;
        }
        const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;
        let status = unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
    #[cfg(not(windows))]
    {
        use std::fs::File;
        use std::io::Read;
        File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(buf))
            .map_err(|_| ())
    }
}

/// Paths equivalentes para o mesmo working tree (normaliza barras / trim).
pub fn same_repo_path(a: &str, b: &str) -> bool {
    fn norm(p: &str) -> String {
        let p = p.trim().replace('/', "\\");
        let p = p.trim_end_matches('\\');
        // Comparação case-insensitive no Windows.
        #[cfg(windows)]
        {
            p.to_ascii_lowercase()
        }
        #[cfg(not(windows))]
        {
            p.to_string()
        }
    }
    norm(a) == norm(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_req() -> WriteRequest {
        WriteRequest::FetchRemote
    }

    #[test]
    fn issue_take_ok_once() {
        let store = WriteAuthStore::new();
        let cmds = vec!["git".into(), "fetch".into()];
        let token = store
            .issue("C:/repo", &sample_req(), &cmds)
            .expect("issue");
        let entry = store.take(&token).expect("take");
        assert!(same_repo_path(&entry.repo_path, "C:/repo"));
        assert_eq!(entry.commands, cmds);
        assert!(store.take(&token).is_err(), "replay deve falhar");
    }

    #[test]
    fn restore_permite_retry_apos_falha() {
        let store = WriteAuthStore::new();
        let cmds = vec!["git".into(), "push".into()];
        let token = store
            .issue("C:/repo", &sample_req(), &cmds)
            .expect("issue");
        let entry = store.take(&token).expect("take");
        store.restore(&token, entry);
        let again = store.take(&token).expect("retry");
        assert_eq!(again.commands, cmds);
    }

    #[test]
    fn take_sem_token_falha() {
        let store = WriteAuthStore::new();
        assert!(store.take("").is_err());
        assert!(store.take("inexistente").is_err());
    }

    #[test]
    fn revoke_impede_take() {
        let store = WriteAuthStore::new();
        let token = store
            .issue("C:/repo", &sample_req(), &["git".into()])
            .unwrap();
        store.revoke(&token);
        assert!(store.take(&token).is_err());
    }

    #[test]
    fn same_repo_path_normaliza() {
        assert!(same_repo_path(r"C:\Repo\A", "C:/Repo/A"));
        assert!(!same_repo_path(r"C:\Repo\A", r"C:\Repo\B"));
    }

    /// B-03 — vínculo preview→execute: request/argv ficam no store; execute sem token falha.
    #[test]
    fn bind_preview_execute_rejeita_sem_token_e_exige_argv() {
        let store = WriteAuthStore::new();
        let cmds = vec!["git".into(), "push".into()];
        let token = store
            .issue("C:/repo", &WriteRequest::Push, &cmds)
            .unwrap();
        let entry = store.take(&token).unwrap();
        assert_eq!(entry.commands, cmds);
        assert!(matches!(entry.request, WriteRequest::Push));
        // Sem preview prévio: não há como “adivinhar” o token.
        assert!(store.take("qualquer-coisa").is_err());
        // Clone usa store separado — tokens não se misturam.
        let clone_store = CloneAuthStore::new();
        let creq = CloneRequest {
            url: "https://github.com/a/b.git".into(),
            parent_dir: "C:/tmp".into(),
            folder_name: "b".into(),
            branch: None,
            depth: None,
        };
        let ctoken = clone_store
            .issue("C:/tmp", &creq, &["git".into(), "clone".into()])
            .unwrap();
        assert!(store.take(&ctoken).is_err());
        assert!(clone_store.take(&ctoken).is_ok());
    }
}
