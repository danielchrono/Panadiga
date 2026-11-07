import React from 'react';
import { PicoDeviceStatus } from '@/models';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { formatVoltage, isTargetPowered } from '@/utils/formatters';

export interface PicoBenchMetricsProps {
  deviceStatus: PicoDeviceStatus | null;
}

/**
 * Pure Functional Presentation Component: Displays hardware telemetry instruments.
 */
export const PicoBenchMetrics: React.FC<PicoBenchMetricsProps> = ({ deviceStatus }) => {
  const isPowered = isTargetPowered(deviceStatus?.vdd_volts);
  const voltageStr = formatVoltage(deviceStatus?.vdd_volts);

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <MetricCard
        title="Tensão VDD (ADC0 / GP26)"
        value={voltageStr}
        unit="Volts"
        subtitle={isPowered ? 'Trilho de 3.3V Estabilizado (OK)' : 'Subtensão / Gravador Desligado'}
        variant={isPowered ? 'emerald' : 'rose'}
        badge={
          isPowered ? (
            <StatusBadge variant="success" label="POWERED" />
          ) : (
            <StatusBadge variant="error" label="UNPOWERED" />
          )
        }
      />
      <MetricCard
        title="Estado do Núcleo AM34"
        value={deviceStatus?.core_state ?? 'DISCONNECTED'}
        subtitle="Sinal OCD DCSR interceptado"
        variant={deviceStatus?.core_state === 'HALTED' ? 'cyan' : 'slate'}
        badge={
          deviceStatus?.core_state === 'HALTED' ? (
            <StatusBadge variant="info" label="OCD ACTIVE" />
          ) : (
            <StatusBadge variant="neutral" label="STANDBY" />
          )
        }
      />
      <MetricCard
        title="Frequência do Barramento"
        value="1.00"
        unit="MHz"
        subtitle="RP2040 PIO SM @ 4.0 MHz (4 clocks/bit)"
        variant="slate"
      />
      <MetricCard
        title="Modo Cold Break-In"
        value={`Modo ${deviceStatus?.breakin_mode ?? 2}`}
        subtitle="Captura síncrona na subida de reset"
        variant="emerald"
        badge={<StatusBadge variant="success" label="RECOMENDADO" />}
      />
    </div>
  );
};
