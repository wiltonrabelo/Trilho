// Contrato de erro tipado da fronteira IPC — espelha `AppError` do backend
// (src-tauri/src/application/app_error.rs). O fluxo é decidido pelo `code`,
// nunca por heurística sobre o texto da mensagem.

export type AppErrorCode = "previewRequired" | "operationFailed";

export class AppIpcError extends Error {
  readonly code: AppErrorCode;

  constructor(code: AppErrorCode, message: string) {
    super(message);
    this.name = "AppIpcError";
    this.code = code;
  }
}

function isSerializedAppError(
  value: unknown,
): value is { code: AppErrorCode; message: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as { message: unknown }).message === "string" &&
    ((value as { code: unknown }).code === "previewRequired" ||
      (value as { code: unknown }).code === "operationFailed")
  );
}

/** Converte a rejeição crua do invoke Tauri em `AppIpcError`/`Error`. */
export function toAppError(raw: unknown): Error {
  if (raw instanceof Error) return raw;
  if (isSerializedAppError(raw)) return new AppIpcError(raw.code, raw.message);
  return new Error(String(raw));
}

/**
 * Token A-02 consumido/expirado ou estado do repo mudou desde o preview —
 * o chamador deve pedir um novo preview e reapresentar a confirmação.
 */
export function isPreviewRequiredError(error: unknown): boolean {
  return error instanceof AppIpcError && error.code === "previewRequired";
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
