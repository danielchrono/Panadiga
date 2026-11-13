import React, { useState } from 'react';
import { useNvramViewModel } from '@/viewmodels/useNvramViewModel';
import {
  NvramTopBanner,
  NvramActionsCard,
  NvramMetricsGrid,
  NvramSectorMap,
  NvramMicrosectorsTable,
  NvramAuditDialog,
} from '@/components/nvram';
import { useTranslation } from '@/i18n/useTranslation';
import { AlertCircle, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';

/**
 * Declarative MVVM View: Spansion NOR Flash S29JL064H & NVRAM U99 Sanitizer.
 * Fully decomposed following Single Responsibility, Functional Programming, and Radix UI.
 */
export const NvramView: React.FC = () => {
  const { t } = useTranslation();
  const {
    report,
    filteredSectors,
    selectedSector,
    setSelectedSector,
    loading,
    capturingLive,
    liveCapture,
    cleaning,
    cleanSuccess,
    filterType,
    setFilterType,
    error,
    executeSurgicalClean,
    captureFromLivePico,
    clearLiveCapture,
    refresh,
  } = useNvramViewModel();

  const [showBackupModal, setShowBackupModal] = useState<boolean>(false);

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Surgical Clean Notification */}
      {cleanSuccess && (
        <NvramTopBanner onOpenAudit={() => setShowBackupModal(true)} />
      )}

      {/* Error Banner */}
      {error && (
        <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/40 text-rose-300 text-xs flex items-center justify-between shadow-lg">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
            <span>Falha ao carregar dump de flash NOR: {error}</span>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={refresh}
            className="text-rose-300 hover:text-white hover:bg-rose-900/40 gap-1.5"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            <span>Tentar Novamente</span>
          </Button>
        </div>
      )}

      {/* Top Bar: Action Panel & Flash Model Header */}
      <NvramActionsCard
        report={report}
        cleaning={cleaning}
        cleanSuccess={cleanSuccess}
        liveCapture={liveCapture}
        capturingLive={capturingLive}
        onExecuteClean={executeSurgicalClean}
        onCaptureLive={() => {
          void captureFromLivePico();
        }}
        onClearLiveCapture={() => {
          void clearLiveCapture();
        }}
        onRefresh={refresh}
      />

      {/* 4 High-Contrast Telemetry Metric Cards */}
      <NvramMetricsGrid
        report={report}
        selectedSector={selectedSector}
        cleanSuccess={cleanSuccess}
      />

      {/* Interactive Sector Physical Grid & Selected Sector Inspector */}
      <NvramSectorMap
        sectorMap={filteredSectors}
        selectedSector={selectedSector}
        filterType={filterType}
        cleanSuccess={cleanSuccess}
        onSelectSector={setSelectedSector}
        onSelectFilter={setFilterType}
      />

      {/* Microsectors Detailed Audit Table */}
      {report && (
        <NvramMicrosectorsTable report={report} cleanSuccess={cleanSuccess} />
      )}

      {/* Accessible Radix UI Audit & Hash Verification Modal */}
      <NvramAuditDialog
        isOpen={showBackupModal}
        onClose={() => setShowBackupModal(false)}
        report={report}
      />
    </div>
  );
};
