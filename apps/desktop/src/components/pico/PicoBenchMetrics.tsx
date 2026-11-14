import React from 'react';
import type { HardwareSsotSpec, PicoDeviceStatus } from '@/models';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import {
  formatVoltage,
  isTargetPowered,
  isTargetVoltageNominal,
} from '@/utils/formatters';

export interface PicoBenchMetricsProps {
  deviceStatus: PicoDeviceStatus | null;
  ssot: HardwareSsotSpec;
  detectedFlashMb?: number | null;
}

/**
 * Pure Functional Presentation Component: Displays hardware telemetry instruments
 * strictly synchronized with backend SSOT electrical thresholds.
 */
export const PicoBenchMetrics: React.FC<PicoBenchMetricsProps> = ({
  deviceStatus,
  ssot,
  detectedFlashMb,
}) => {
  const { thresholds } = ssot;
  const vdd = deviceStatus?.vdd_volts ?? 0;
  const isPowered = isTargetPowered(vdd, thresholds.min_vdd_threshold_volts);
  const isNominal = isTargetVoltageNominal(
    vdd,
    thresholds.safe_operating_min_volts,
    thresholds.max_vdd_ceiling_volts
  );
  const voltageStr = formatVoltage(vdd);
  const mhzStr = (thresholds.bus_freq_hz / 1_000_000).toFixed(2);

  const railSubtitle = isNominal
    ? 'Trilho de 3.3V Estabilizado (Nominal)'
    : isPowered
      ? `Alvo Ativo (${voltageStr}V >= ${thresholds.min_vdd_threshold_volts.toFixed(1)}V)`
      : `VDD < ${thresholds.min_vdd_threshold_volts.toFixed(1)}V (Desligado / Cabo Solto)`;

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <MetricCard
        title="Tensão VDD (ADC0 / GP26)"
        value={voltageStr}
        unit="Volts"
        subtitle={railSubtitle}
        variant={isNominal ? 'emerald' : isPowered ? 'amber' : 'rose'}
        badge={
          isNominal ? (
            <StatusBadge variant="success" label="POWERED" />
          ) : isPowered ? (
            <StatusBadge variant="warning" label="LOW RAIL" />
          ) : (
            <StatusBadge variant="error" label="UNPOWERED" />
          )
        }
      />
      <MetricCard
        title="Estado do Núcleo AM34"
        value={deviceStatus?.core_state ?? 'DISCONNECTED'}
        subtitle={
          detectedFlashMb
            ? `Flash NOR Detectada: ${detectedFlashMb} MB`
            : 'Canal P7501 Async UART (115200 8N1)'
        }
        variant={deviceStatus?.core_state === 'HALTED' ? 'cyan' : 'slate'}
        badge={
          deviceStatus?.core_state === 'HALTED' ? (
            <StatusBadge variant="info" label="OCD ACTIVE" />
          ) : deviceStatus?.core_state === 'RUNNING' ? (
            <StatusBadge variant="success" label="PRONTO (3.3V)" />
          ) : (
            <StatusBadge variant="neutral" label="STANDBY" />
          )
        }
      />
      <MetricCard
        title="Frequência do Barramento"
        value={mhzStr}
        unit="MHz"
        subtitle="P7501 UART 115200 / PIO Legado"
        variant="slate"
      />
      <MetricCard
        title="Modo Cold Break-In"
        value={`Modo ${deviceStatus?.breakin_mode ?? 2}`}
        subtitle="Captura síncrona na subida de reset"
        variant="emerald"
        badge={
          <StatusBadge
            variant={(deviceStatus?.breakin_mode ?? 2) === 2 ? 'success' : 'info'}
            label={(deviceStatus?.breakin_mode ?? 2) === 2 ? 'RECOMENDADO' : 'CUSTOM'}
          />
        }
      />
    </div>
  );
};
