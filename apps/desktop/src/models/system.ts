/**
 * System Architecture & Local Workspace Status Models
 */

export type NavigationTabId =
  | 'dashboard'
  | 'nvram'
  | 'hdd'
  | 'pico'
  | 'crypto'
  | 'manuals'
  | 'hex'
  | 'rag';

export type ViewKey =
  | 'dashboard'
  | 'nvram'
  | 'hdd'
  | 'picobench'
  | 'frmcrypto'
  | 'manuals'
  | 'hexinspector';

export const ExecutionStatus = {
  IDLE: 'IDLE',
  RUNNING: 'RUNNING',
  SUCCESS: 'SUCCESS',
  ERROR: 'ERROR',
} as const;

export type ExecutionStatus = (typeof ExecutionStatus)[keyof typeof ExecutionStatus];

export interface BinaryFileStatus {
  id: string;
  name: string;
  path: string;
  exists: boolean;
  size_bytes?: number;
  formatted_size?: string;
  expected_role: string;
  status: 'ready' | 'missing' | 'warning';
}

export interface SystemOverview {
  product_model: string;
  soc_processor: string;
  core_architecture: string;
  os_system: string;
  flash_rom: string;
  hdd_specs: string;
  optical_drive: string;
  debug_port: string;
  error_code: string;
  error_description: string;
  firmware_version: string;
  binaries: BinaryFileStatus[];
}
