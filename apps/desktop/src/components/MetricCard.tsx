import React from 'react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: string | number;
  unit?: string;
  subtitle?: string;
  icon?: React.ReactNode;
  badge?: React.ReactNode;
  variant?: 'emerald' | 'amber' | 'cyan' | 'rose' | 'slate';
  className?: string;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  unit,
  subtitle,
  icon,
  badge,
  variant = 'slate',
  className = '',
}) => {
  const borderVariants = {
    emerald: 'border-emerald-500/30 hover:border-emerald-500/60 shadow-emerald-950/20',
    amber: 'border-amber-500/30 hover:border-amber-500/60 shadow-amber-950/20',
    cyan: 'border-cyan-500/30 hover:border-cyan-500/60 shadow-cyan-950/20',
    rose: 'border-rose-500/30 hover:border-rose-500/60 shadow-rose-950/20',
    slate: 'border-slate-800 hover:border-slate-700 shadow-slate-950/20',
  };

  const textVariants = {
    emerald: 'text-emerald-400',
    amber: 'text-amber-400',
    cyan: 'text-cyan-400',
    rose: 'text-rose-400',
    slate: 'text-slate-100',
  };

  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-xl bg-bench-900/80 backdrop-blur border p-4 transition-all duration-200 shadow-lg',
        borderVariants[variant],
        className
      )}
    >
      <div className="flex items-center justify-between gap-2 mb-2">
        <span className="text-xs font-mono font-medium tracking-wide text-slate-400 uppercase">
          {title}
        </span>
        <div className="flex items-center gap-1.5">
          {badge}
          {icon && <span className="text-slate-400">{icon}</span>}
        </div>
      </div>

      <div className="flex items-baseline gap-1.5">
        <span className={`text-2xl font-bold font-mono tracking-tight ${textVariants[variant]}`}>
          {value}
        </span>
        {unit && (
          <span className="text-xs font-mono font-medium text-slate-500">
            {unit}
          </span>
        )}
      </div>

      {subtitle && (
        <p className="mt-1 text-xs text-slate-400 truncate font-mono">
          {subtitle}
        </p>
      )}
    </div>
  );
};
