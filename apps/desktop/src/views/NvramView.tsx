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

/**
 * Declarative MVVM View: Spansion NOR Flash S29JL064H & NVRAM U99 Sanitizer.
 * Fully decomposed following Single Responsibility, Functional Programming, and Radix UI.
 */
export const NvramView: React.FC = () => {
  const {
    report,
    sectorMap,
    selectedSector,
    setSelectedSector,
    cleaning,
    cleanSuccess,
    filterType,
    setFilterType,
    executeSurgicalClean,
    refresh,
  } = useNvramViewModel();

  const [showBackupModal, setShowBackupModal] = useState<boolean>(false);

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Surgical Clean Notification */}
      {cleanSuccess && (
        <NvramTopBanner onOpenAudit={() => setShowBackupModal(true)} />
      )}

      {/* Top Bar: Action Panel & Spansion Model Header */}
      <NvramActionsCard
        cleaning={cleaning}
        cleanSuccess={cleanSuccess}
        onExecuteClean={executeSurgicalClean}
        onRefresh={refresh}
      />

      {/* 4 High-Contrast Telemetry Metric Cards */}
      <NvramMetricsGrid
        report={report}
        selectedSector={selectedSector}
        cleanSuccess={cleanSuccess}
      />

      {/* Interactive 71-Sector Physical Grid & Selected Sector Inspector */}
      <NvramSectorMap
        sectorMap={sectorMap}
        selectedSector={selectedSector}
        filterType={filterType}
        cleanSuccess={cleanSuccess}
        onSelectSector={setSelectedSector}
        onSelectFilter={setFilterType}
      />

      {/* 8 Microsectors Detailed Audit Table */}
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
