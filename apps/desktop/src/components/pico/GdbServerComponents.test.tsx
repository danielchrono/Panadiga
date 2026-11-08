import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { GdbServerControls } from './GdbServerControls';
import { GdbExternalToolLaunchers } from './GdbExternalToolLaunchers';

describe('GdbServerControls Component (SRP Decomposition)', () => {
  it('renders offline status and allows starting GDB server', async () => {
    const onStart = vi.fn().mockResolvedValue(undefined);
    const onStop = vi.fn().mockResolvedValue(undefined);

    render(
      <GdbServerControls
        gdbStatus={{
          running: false,
          port_name: 'COM5',
          listen_addr: '127.0.0.1:3333',
          connected_clients: 0,
          last_event: 'idle',
        }}
        isStarting={false}
        onStart={onStart}
        onStop={onStop}
      />
    );

    expect(screen.getByText('OFFLINE')).toBeDefined();
    const startButton = screen.getByRole('button', { name: /INICIAR SERVIDOR GDB/i });
    expect(startButton).toBeDefined();

    fireEvent.click(startButton);
    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it('renders active listening status and allows stopping GDB server', async () => {
    const onStart = vi.fn().mockResolvedValue(undefined);
    const onStop = vi.fn().mockResolvedValue(undefined);

    render(
      <GdbServerControls
        gdbStatus={{
          running: true,
          port_name: 'COM5',
          listen_addr: '127.0.0.1:3333',
          connected_clients: 1,
          last_event: 'client_connected',
        }}
        isStarting={false}
        onStart={onStart}
        onStop={onStop}
      />
    );

    expect(screen.getByText(/CLIENTE ATIVO/i)).toBeDefined();
    const stopButton = screen.getByRole('button', { name: /Parar GDB/i });
    expect(stopButton).toBeDefined();

    fireEvent.click(stopButton);
    expect(onStop).toHaveBeenCalledTimes(1);
  });
});

describe('GdbExternalToolLaunchers Component (SRP Decomposition)', () => {
  it('renders all quick engineering tool buttons and triggers onLaunchTool', () => {
    const onLaunchTool = vi.fn().mockResolvedValue(undefined);

    render(<GdbExternalToolLaunchers onLaunchTool={onLaunchTool} />);

    const gdbButton = screen.getByRole('button', { name: /GDB Multiarch/i });
    const openocdButton = screen.getByRole('button', { name: /OpenOCD AM34/i });
    const ghidraButton = screen.getByRole('button', { name: /Scripts Ghidra/i });
    const hxdButton = screen.getByRole('button', { name: /HxD Editor/i });
    const terminalButton = screen.getByRole('button', { name: /PowerShell/i });

    expect(gdbButton).toBeDefined();
    expect(openocdButton).toBeDefined();
    expect(ghidraButton).toBeDefined();
    expect(hxdButton).toBeDefined();
    expect(terminalButton).toBeDefined();

    fireEvent.click(gdbButton);
    expect(onLaunchTool).toHaveBeenCalledWith('gdb');

    fireEvent.click(openocdButton);
    expect(onLaunchTool).toHaveBeenCalledWith('openocd');

    fireEvent.click(ghidraButton);
    expect(onLaunchTool).toHaveBeenCalledWith('ghidra');

    fireEvent.click(hxdButton);
    expect(onLaunchTool).toHaveBeenCalledWith('hxd');

    fireEvent.click(terminalButton);
    expect(onLaunchTool).toHaveBeenCalledWith('terminal');
  });
});
