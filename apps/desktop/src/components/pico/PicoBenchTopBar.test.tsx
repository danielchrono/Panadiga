import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import type { PicoDeviceStatus } from '@/models';
import { simulateMockDiagnostic } from '@/test/mocks/mockDiagnostics';
import { PicoBenchTopBar } from './PicoBenchTopBar';

const CONNECTED_ZERO_VDD_STATUS: PicoDeviceStatus = {
  connected: true,
  port: 'COM10 (RP2040)',
  baudRate: 115200,
  vdd_volts: 0,
  target_powered: false,
  breakin_mode: 1,
  core_state: 'RUNNING',
  bus_freq_hz: 24000000,
  firmware_version: '1.0.0',
  chip_id: 'RP2040',
  total_probes: 0,
  hits_detected: 0,
  open_bus_high: 0,
  open_bus_low: 0,
  ram_loader_injected: false,
  last_keepalive: '00:00:00',
};

describe('PicoBenchTopBar voltage fallback reconciliation', () => {
  it('shows low-voltage banner at 0.00V before PINCHECK and hides it when PINCHECK measures >= 2.50V', () => {
    const baseProps = {
      deviceStatus: CONNECTED_ZERO_VDD_STATUS,
      availablePorts: ['COM10 (RP2040)'],
      selectedPort: 'COM10 (RP2040)',
      isErasing: false,
      onSelectPort: vi.fn(),
      onToggleConnection: vi.fn(),
      onInjectAndErase: vi.fn(),
    };

    const { rerender } = render(<PicoBenchTopBar {...baseProps} latestDiagnostic={null} />);

    expect(
      screen.getByText(/Sonda Pico Operacional, mas Trilho Alvo em 0\.00V \(< 2\.50V\)/)
    ).toBeDefined();

    const pincheckResult = { ...simulateMockDiagnostic('PINCHECK'), vdd_volts: 3.24 };
    rerender(<PicoBenchTopBar {...baseProps} latestDiagnostic={pincheckResult} />);

    expect(
      screen.queryByText(/Sonda Pico Operacional, mas Trilho Alvo em .* \(< 2\.50V\)/)
    ).toBeNull();
  });
});
