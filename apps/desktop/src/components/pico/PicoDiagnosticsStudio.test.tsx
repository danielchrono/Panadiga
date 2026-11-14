import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { DEFAULT_HARDWARE_SSOT_SPEC } from '@/test/fixtures/ssotFixture';
import { simulateMockDiagnostic } from '@/test/mocks/mockDiagnostics';
import { PicoDiagnosticsStudio } from './PicoDiagnosticsStudio';

describe('PicoDiagnosticsStudio Component', () => {
  it('renders empty state initially and dispatches PINCHECK command', async () => {
    const onExecute = vi.fn().mockResolvedValue(simulateMockDiagnostic('PINCHECK'));

    render(
      <PicoDiagnosticsStudio
        ssot={DEFAULT_HARDWARE_SSOT_SPEC}
        latestResult={null}
        history={[]}
        runningCommandId={null}
        error={null}
        selectedCategory="electrical"
        onSelectCategory={vi.fn()}
        onExecute={onExecute}
        onClearHistory={vi.fn()}
      />
    );

    expect(screen.getByText('Nenhum diagnóstico executado nesta sessão')).toBeDefined();
    const runPincheck = screen.getByRole('button', { name: /Executar PINCHECK/i });
    fireEvent.click(runPincheck);
    expect(onExecute).toHaveBeenCalledWith('PINCHECK', '');
  });

  it('renders structured PINCHECK matrix and DUMP table when latestResult is populated', () => {
    const pinResult = simulateMockDiagnostic('PINCHECK');

    const { rerender } = render(
      <PicoDiagnosticsStudio
        ssot={DEFAULT_HARDWARE_SSOT_SPEC}
        latestResult={pinResult}
        history={[pinResult]}
        runningCommandId={null}
        error={null}
        selectedCategory="electrical"
        onSelectCategory={vi.fn()}
        onExecute={vi.fn()}
        onClearHistory={vi.fn()}
      />
    );

    expect(screen.getByRole('cell', { name: /GP2 \(Pin6 RXD\s*\)/ })).toBeDefined();
    expect(screen.getByText('XRESET Rise: 40 us')).toBeDefined();

    const dumpResult = simulateMockDiagnostic('DUMP', '0x807F0000 16');
    rerender(
      <PicoDiagnosticsStudio
        ssot={DEFAULT_HARDWARE_SSOT_SPEC}
        latestResult={dumpResult}
        history={[dumpResult, pinResult]}
        runningCommandId={null}
        error={null}
        selectedCategory="ocd_flash"
        onSelectCategory={vi.fn()}
        onExecute={vi.fn()}
        onClearHistory={vi.fn()}
      />
    );

    expect(screen.getByText('0x807F0000')).toBeDefined();
    expect(screen.getByText(/0x55AA0100/)).toBeDefined();
  });
});
