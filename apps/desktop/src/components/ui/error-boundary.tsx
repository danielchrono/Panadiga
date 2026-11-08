import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';
import { Button } from './button';
import { logger } from '@/utils/logger';

interface Props {
  children: ReactNode;
  fallbackTitle?: string;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public override state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    logger.error('ErrorBoundary', 'Uncaught rendering exception', { error, errorInfo });
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  public override render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center min-h-[360px] p-8 text-center rounded-2xl bg-slate-950/80 border border-rose-500/30">
          <div className="w-12 h-12 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-center justify-center text-rose-400 mb-4 shadow-lg shadow-rose-950/40">
            <AlertTriangle className="w-6 h-6" />
          </div>

          <h3 className="text-base font-bold font-mono text-slate-100 mb-1">
            {this.props.fallbackTitle ?? 'Falha de Renderização no Módulo'}
          </h3>

          <p className="text-xs text-slate-400 max-w-md font-sans mb-4">
            Ocorreu uma exceção não tratada durante o processamento do componente. A integridade da
            memória física e do hardware da bancada permanece preservada.
          </p>

          {this.state.error && (
            <pre className="max-w-xl w-full p-3 mb-5 text-[11px] font-mono text-rose-300/90 bg-rose-950/30 border border-rose-900/50 rounded-lg text-left overflow-x-auto">
              {this.state.error.message}
            </pre>
          )}

          <Button variant="outline" size="sm" onClick={this.handleReset} className="gap-2">
            <RefreshCw className="w-3.5 h-3.5" />
            <span>Tentar Novamente</span>
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
