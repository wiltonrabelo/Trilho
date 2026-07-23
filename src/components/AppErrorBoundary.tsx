import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = { children: ReactNode };
type State = { error: Error | null };

/** Evita tela branca total se o React quebrar em runtime. */
export class AppErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("Trilho UI crash:", error, info.componentStack);
  }

  private reload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="flex h-full min-h-[100vh] flex-col items-center justify-center gap-3 bg-bg px-6 text-center text-text">
        <p className="text-sm font-semibold">A interface encontrou um erro.</p>
        <p className="max-w-md text-xs text-muted">
          {this.state.error.message || "Falha inesperada no renderer."}
        </p>
        <button
          type="button"
          onClick={this.reload}
          className="rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90"
        >
          Recarregar
        </button>
      </div>
    );
  }
}
