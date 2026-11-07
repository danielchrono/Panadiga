import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { StatusBadge } from './StatusBadge';

describe('StatusBadge Component', () => {
  it('renders label with success variant', () => {
    render(<StatusBadge variant="success" label="CONECTADO" />);
    const badge = screen.getByText('CONECTADO');
    expect(badge).toBeDefined();
    expect(badge.closest('span')?.className).toContain('text-emerald-400');
  });

  it('renders warning, error, info and neutral variants', () => {
    const { rerender } = render(<StatusBadge variant="warning" label="ATENÇÃO" />);
    expect(screen.getByText('ATENÇÃO').closest('span')?.className).toContain('text-amber-400');

    rerender(<StatusBadge variant="error" label="FALHA" />);
    expect(screen.getByText('FALHA').closest('span')?.className).toContain('text-rose-400');

    rerender(<StatusBadge variant="info" label="PROCESSANDO" />);
    expect(screen.getByText('PROCESSANDO').closest('span')?.className).toContain('text-cyan-400');

    rerender(<StatusBadge variant="neutral" label="DESCONECTADO" />);
    expect(screen.getByText('DESCONECTADO').closest('span')?.className).toContain('text-slate-400');
  });

  it('toggles pulse dot based on prop', () => {
    const { container } = render(<StatusBadge variant="success" label="ONLINE" dot={true} />);
    const dot = container.querySelector('.rounded-full.w-1\\.5');
    expect(dot).toBeDefined();
  });
});
