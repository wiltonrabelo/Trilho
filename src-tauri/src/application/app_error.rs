//! Erro tipado da fronteira IPC — o frontend decide o fluxo pelo `code`,
//! nunca por heurística sobre o texto da mensagem.

use serde::Serialize;

/// Código estável do erro — contrato com o frontend (`src/lib/errors.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AppErrorCode {
    /// Token A-02 ausente/consumido/expirado ou estado do repo mudou desde o
    /// preview. O frontend deve pedir um novo preview antes de tentar de novo.
    PreviewRequired,
    /// Falha na operação — mensagem já amigável (MVP §4), sem stderr cru.
    OperationFailed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
}

impl AppError {
    pub fn preview_required(message: impl Into<String>) -> Self {
        Self {
            code: AppErrorCode::PreviewRequired,
            message: message.into(),
        }
    }

    pub fn operation_failed(message: impl Into<String>) -> Self {
        Self {
            code: AppErrorCode::OperationFailed,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<super::GitError> for AppError {
    fn from(err: super::GitError) -> Self {
        AppError::operation_failed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializa_code_em_camel_case() {
        let err = AppError::preview_required("token consumido");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "previewRequired");
        assert_eq!(json["message"], "token consumido");
    }
}
