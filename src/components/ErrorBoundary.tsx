import { Component, type ErrorInfo, type ReactNode } from "react";
import { BrandMark } from "./BrandMark";
import { Banner } from "./ui";

type Props = { children: ReactNode };
type State = { error: Error | null };

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("Unexpected initialization error", error, info.componentStack);
  }

  render(): ReactNode {
    if (this.state.error) {
      return (
        <main className="camera-page" data-testid="camera-state" data-state="init-error">
          <header>
            <BrandMark size={32} />
            <h1>Unexpected initialization error</h1>
          </header>
          <Banner>{this.state.error.message}</Banner>
        </main>
      );
    }
    return this.props.children;
  }
}
