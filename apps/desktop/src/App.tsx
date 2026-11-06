import React, { useState } from 'react';
import { Sidebar, ViewKey } from './components/Sidebar';
import { Header } from './components/Header';
import { DashboardView } from './views/DashboardView';
import { NvramView } from './views/NvramView';
import { HddStorageView } from './views/HddStorageView';
import { PicoBenchView } from './views/PicoBenchView';
import { FrmCryptoView } from './views/FrmCryptoView';
import { ManualsView } from './views/ManualsView';
import { HexInspectorView } from './views/HexInspectorView';
import { isDemoModeActive, setDemoMode } from './services/ipc';

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

        {/* View Content with scrolling */}
        <main className="flex-1 overflow-y-auto p-6 scrollbar-thin scrollbar-thumb-slate-800">
          {currentView === 'dashboard' && (
            <DashboardView onNavigate={setCurrentView} />
          )}
          {currentView === 'nvram' && <NvramView />}
          {currentView === 'hdd' && <HddStorageView />}
          {currentView === 'picobench' && <PicoBenchView />}
          {currentView === 'frmcrypto' && <FrmCryptoView />}
          {currentView === 'manuals' && <ManualsView />}
          {currentView === 'hexinspector' && <HexInspectorView />}
        </main>
      </div>
    </div>
  );
};

export default App;
