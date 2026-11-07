import React, { useState, Suspense, lazy } from 'react';
import { Sidebar, ViewKey } from './components/Sidebar';
import { Header } from './components/Header';
import { isDemoModeActive, setDemoMode } from './services/ipc';
import { Loader2 } from 'lucide-react';

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

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bench-950 text-slate-100 font-sans">
      {/* Left Sidebar Navigation */}
      <Sidebar
        currentView={currentView}
        onSelectView={setCurrentView}
        vddVolts={3.28}
        nvramCleaned={false}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
        <Header
          currentViewTitle={viewTitles[currentView]}
          demoMode={demoMode}
          onToggleDemoMode={handleToggleDemoMode}
        />

        {/* View Content with declarative Suspense loading */}
        <main className="flex-1 overflow-y-auto p-6 scrollbar-thin scrollbar-thumb-slate-800">
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
        </main>
      </div>
    </div>
  );
};

export default App;
