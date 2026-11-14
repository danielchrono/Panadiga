import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { Sc0BridgePanel } from './Sc0BridgePanel';

describe('Sc0BridgePanel Component', () => {
  it('renders quick PANARSP, WAITTX and Optical Drive buttons cleanly', () => {
    const onSendRsp = vi.fn();
    const onSendDrive = vi.fn();
    const onWaittx = vi.fn();

    render(
      <Sc0BridgePanel
        status={{
          running: false,
          port_name: 'COM10',
          baud_rate: 115200,
          boot_mode: 'JIG:350',
          bytes_rx: 0,
          bytes_tx: 0,
          last_event: 'Standby',
        }}
        logs={[]}
        baudRate="115200"
        onChangeBaudRate={vi.fn()}
        bootMode="JIG:350"
        onChangeBootMode={vi.fn()}
        inputMode="line"
        onChangeInputMode={vi.fn()}
        txInput=""
        onChangeTxInput={vi.fn()}
        isBusy={false}
        error={null}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onSubmitTx={vi.fn()}
        onClearLogs={vi.fn()}
        onTriggerWaittx={onWaittx}
        onSendRspPreset={onSendRsp}
        onSendDriveCommand={onSendDrive}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: '0x15 NAK' }));
    expect(onSendRsp).toHaveBeenCalledWith('hex', '15', '4000 JIG:350 NAK');

    fireEvent.click(screen.getByRole('button', { name: 'PASS' }));
    expect(onSendDrive).toHaveBeenCalledWith('PASS');

    fireEvent.click(screen.getByRole('button', { name: /WAITTX wait 1/i }));
    expect(onWaittx).toHaveBeenCalledWith('8000 JIG:350 "wait 1" 0D 4000');
  });
});
