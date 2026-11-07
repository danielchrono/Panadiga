import React from 'react';
import { StatusBadge, StatusVariant } from '@/components/StatusBadge';

export interface MbrSectorRow {
  label: string;
  value: React.ReactNode;
  valueClassName?: string;
  title?: string;
}

export interface MbrSectorDetailsCardProps {
  title: string;
  titleColor?: string;
  badgeLabel: string;
  badgeVariant?: StatusVariant;
  rows: MbrSectorRow[];
}

export const MbrSectorDetailsCard: React.FC<MbrSectorDetailsCardProps> = ({
  title,
  titleColor = 'text-cyan-400',
  badgeLabel,
  badgeVariant = 'info',
  rows,
}) => (
  <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
    <div className="flex items-center justify-between border-b border-slate-800 pb-2">
      <span className={`font-bold ${titleColor}`}>{title}</span>
      <StatusBadge variant={badgeVariant} label={badgeLabel} />
    </div>
    <div className="space-y-1.5 text-slate-400">
      {rows.length === 0 ? (
        <div className="py-2 text-center text-xs text-slate-500 font-mono">
          Nenhum parâmetro disponível
        </div>
      ) : (
        rows.map((row, idx) => (
          <div key={idx} className="flex justify-between">
            <span>{row.label}</span>
            <span
              className={row.valueClassName ?? 'text-slate-200'}
              title={row.title}
            >
              {row.value}
            </span>
          </div>
        ))
      )}
    </div>
  </div>
);
