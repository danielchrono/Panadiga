import React, { useState, useEffect, Suspense, lazy } from 'react';
import { Sidebar, ViewKey } from './components/Sidebar';
import { Header } from './components/Header';
import { isDemoModeActive, setDemoMode, listenEvent } from './services/ipc';
import { PicoDeviceStatus } from './models/pico';
import { ToastProvider } from './context/ToastContext';
import { ErrorBoundary } from './components/ui/error-boundary';
import { Loader2, AlertCircle, RefreshCw } from 'lucide-react';
import { Button } from './components/ui/button';

// Lazy-loaded views for optimal code-splitting and reduced initial bundle size
const DashboardView = lazy(() =>
  import('./views/DashboardView').then(m => ({ default: m.DashboardView }))
);
const NvramView = lazy(() =>
  import('./views/NvramView').then(m => ({ default: m.NvramView }))
);
const HddStorageView = lazy(() =>
  import('./views/HddStorageView').then(m => ({ default: m.HddStorageView }))
);
const PicoBenchView = lazy(() =>
  import('./views/PicoBenchView').then(m => ({ default: m.PicoBenchView }))
);
const FrmCryptoView = lazy(() =>
  import('./views/FrmCryptoView').then(m => ({ default: m.FrmCryptoView }))
);
const ManualsView = lazy(() =>
  import('./views/ManualsView').then(m => ({ default: m.ManualsView }))
);
const HexInspectorView = lazy(() =>
  import('./views/HexInspectorView').then(m => ({ default: m.HexInspectorView }))
);

const ViewLoadingFallback: React.FC = () => (
  <div className="flex flex-col items-center justify-center min-h-[400px] h-full gap-3 text-slate-400">
    <Loader2 className="w-8 h-8 animate-spin text-cyan-400" />
    <span className="text-sm font-mono tracking-wide animate-pulse">
      Carregando subsistema de bancada...
    </span>
  </div>
);

export const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<ViewKey>('dashboard');
  const [demoMode, setDemoState] = useState<boolean>(isDemoModeActive());
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [hardwareStatus, setHardwareStatus] = useState<{
    connected: boolean;
    vdd_volts: number;
  }>({
    connected: false,
    vdd_volts: 0.0,
  });

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenEvent<PicoDeviceStatus>('pico_telemetry', incoming => {
      setHardwareStatus({
        connected: incoming.connected,
        vdd_volts: incoming.vdd_volts,
      });
    }).then(cleanup => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const viewTitles: Record<ViewKey, string> = {
    dashboard: 'Visão Geral do Gravador & Status da Bancada',
    nvram: 'Flash Spansion S29JL064H & Cirurgia NVRAM U99',
    hdd: 'Armazenamento MEIHDFS & Forensic do Disco Rígido',
    picobench: 'Bancada Raspberry Pi Pico & DWire32A OCD Master',
    frmcrypto: 'Criptoanálise Óptica PANA_DVD.FRM & Dicionário ECB',
    manuals: 'Manuais de Serviço & Diagramas Esquemáticos (15 PDFs)',
    hexinspector: 'Inspetor Hexadecimal & Alinhamento LBA',
  };

  const handleToggleDemoMode = (active: boolean) => {
    setDemoMode(active);
    setDemoState(active);
  };

  const handleRetry = () => {
    setError(null);
    setIsLoading(false);
  };

  if (isLoading) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bench-950 text-slate-100">
        <ViewLoadingFallback />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bench-950 p-6">
        <div className="flex flex-col items-center max-w-md p-8 rounded-2xl bg-slate-900 border border-rose-500/30 text-center">
          <AlertCircle className="w-12 h-12 text-rose-400 mb-4" />
          <h2 className="text-lg font-bold font-mono text-slate-100 mb-2">Falha Crítica no Subsistema</h2>
          <p className="text-xs text-slate-400 font-sans mb-6">{error}</p>
          <Button variant="default" size="sm" onClick={handleRetry} className="gap-2">
            <RefreshCw className="w-3.5 h-3.5" />
            <span>Recarregar Subsistema</span>
          </Button>
        </div>
      </div>
    );
  }

  return (
    <ToastProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-bench-950 text-slate-100 font-sans">
        {/* Left Sidebar Navigation */}
        <Sidebar
          currentView={currentView}
          onSelectView={setCurrentView}
          vddVolts={hardwareStatus.vdd_volts}
          hardwareConnected={hardwareStatus.connected}
          nvramCleaned={false}
        />

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
          <Header
            currentViewTitle={viewTitles[currentView]}
            demoMode={demoMode}
            onToggleDemoMode={handleToggleDemoMode}
            hardwareConnected={hardwareStatus.connected}
            vddVolts={hardwareStatus.vdd_volts}
          />

          {/* View Content with declarative Suspense loading & ErrorBoundary */}
          <main className="flex-1 overflow-y-auto p-6 scrollbar-thin scrollbar-thumb-slate-800">
            <ErrorBoundary fallbackTitle={`Falha ao carregar ${viewTitles[currentView]}`}>
              <Suspense fallback={<ViewLoadingFallback />}>
                {currentView === 'dashboard' && (
                  <DashboardView onNavigate={setCurrentView} />
                )}
                {currentView === 'nvram' && <NvramView />}
                {currentView === 'hdd' && <HddStorageView />}
                {currentView === 'picobench' && <PicoBenchView />}
                {currentView === 'frmcrypto' && <FrmCryptoView />}
                {currentView === 'manuals' && <ManualsView />}
                {currentView === 'hexinspector' && <HexInspectorView />}
              </Suspense>
            </ErrorBoundary>
          </main>
        </div>
      </div>
    </ToastProvider>
  );
};

export default App;
