import React, { useState } from 'react';
import type {
  HardwareSsotSpec,
  PicoCommandCategory,
  PicoCommandSpec,
  PicoDiagnosticResult,
} from '@/models';
import {
  Alert,
  AlertDescription,
  AlertTitle,
  Badge,
  Button,
  Card,
  EmptyState,
  Input,
  ScrollArea,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui';
import {
  Activity,
  Cpu,
  History,
  Layers,
  Loader2,
  Play,
  Radio,
  ShieldAlert,
  Trash2,
  Wrench,
} from 'lucide-react';

export interface PicoDiagnosticsStudioProps {
  ssot: HardwareSsotSpec;
  latestResult: PicoDiagnosticResult | null;
  history: PicoDiagnosticResult[];
  runningCommandId: string | null;
  error: string | null;
  selectedCategory: PicoCommandCategory;
  onSelectCategory: (cat: PicoCommandCategory) => void;
  onExecute: (commandId: string, arg?: string) => Promise<PicoDiagnosticResult | null>;
  onClearHistory: () => void;
}

const COMMAND_TABS: Array<{
  value: PicoCommandCategory;
  label: string;
  Icon: React.ComponentType<{ className?: string }>;
}> = [
  { value: 'electrical', label: 'Elétrica P7501', Icon: Activity },
  { value: 'ocd_flash', label: 'OCD & NOR Flash', Icon: Cpu },
  { value: 'uart_fmu', label: 'UART & FMU Bootrom', Icon: Radio },
  { value: 'probe', label: 'Controle Sonda', Icon: Wrench },
];

interface CommandCardProps {
  cmd: PicoCommandSpec;
  argValue: string;
  onArgChange: (val: string) => void;
  isBusy: boolean;
  isCurrent: boolean;
  onRun: () => void;
}

const DiagnosticCommandCard: React.FC<CommandCardProps> = ({
  cmd,
  argValue,
  onArgChange,
  isBusy,
  isCurrent,
  onRun,
}) => {
  const [confirmArmed, setConfirmArmed] = useState(false);

  const handleTrigger = () => {
    if (cmd.dangerous && !confirmArmed) {
      setConfirmArmed(true);
      setTimeout(() => setConfirmArmed(false), 4000);
      return;
    }
    setConfirmArmed(false);
    onRun();
  };

  return (
    <div className="rounded-xl bg-slate-950/90 border border-slate-800 p-3.5 flex flex-col justify-between gap-2.5">
      <div className="space-y-1">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <h4 className="text-xs font-bold text-slate-100 leading-snug">{cmd.label}</h4>
            <code className="text-[10px] text-cyan-400 font-mono block">{cmd.syntax}</code>
          </div>
          {cmd.dangerous && (
            <Badge variant="destructive" className="shrink-0 text-[9px]">
              CRÍTICO
            </Badge>
          )}
        </div>
        <p className="text-[11px] text-slate-400 font-sans leading-relaxed">{cmd.description}</p>
      </div>

      <div className="space-y-2 pt-1">
        {cmd.arg_label && (
          <div className="space-y-1.5">
            {cmd.arg_options.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {cmd.arg_options.map(opt => (
                  <button
                    key={opt.value}
                    type="button"
                    onClick={() => onArgChange(opt.value)}
                    className={`px-2 py-0.5 rounded text-[10px] font-mono border transition-colors ${
                      argValue === opt.value
                        ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 font-bold'
                        : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            )}
            <Input
              value={argValue}
              onChange={e => onArgChange(e.target.value)}
              placeholder={cmd.arg_placeholder ?? cmd.arg_label}
              aria-label={`${cmd.id} argumento`}
              className="h-8 text-[11px]"
            />
          </div>
        )}

        <Button
          type="button"
          size="sm"
          variant={confirmArmed ? 'destructive' : cmd.dangerous ? 'outline' : 'default'}
          disabled={isBusy}
          onClick={handleTrigger}
          className="w-full h-8 text-xs font-bold gap-1.5"
        >
          {isCurrent ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              <span>Executando {cmd.id}...</span>
            </>
          ) : confirmArmed ? (
            <>
              <ShieldAlert className="w-3.5 h-3.5" />
              <span>Confirmar {cmd.id}?</span>
            </>
          ) : (
            <>
              <Play className="w-3 h-3" />
              <span>Executar {cmd.id}</span>
            </>
          )}
        </Button>
      </div>
    </div>
  );
};

export const PicoDiagnosticsStudio: React.FC<PicoDiagnosticsStudioProps> = ({
  ssot,
  latestResult,
  history,
  runningCommandId,
  error,
  selectedCategory,
  onSelectCategory,
  onExecute,
  onClearHistory,
}) => {
  const [argMap, setArgMap] = useState<Record<string, string>>({});
  const [activeTab, setActiveTab] = useState<string>(selectedCategory);
  const [inspectedResult, setInspectedResult] = useState<PicoDiagnosticResult | null>(null);

  const activeResult = inspectedResult ?? latestResult;
  const isBusy = runningCommandId !== null;

  const getArgForCmd = (cmd: PicoCommandSpec): string =>
    argMap[cmd.id] !== undefined ? argMap[cmd.id] : cmd.default_arg ?? '';

  return (
    <Card className="p-4 sm:p-5 shadow-xl space-y-4 font-mono">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 border-b border-slate-800 pb-3">
        <div>
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="text-sm font-bold text-slate-100">
              Estúdio Granular P7501 Async UART, Elétrica &amp; Probes NOR Flash
            </h3>
            <Badge variant="default">{ssot.pico_commands.length} COMANDOS FIRMWARE</Badge>
          </div>
          <p className="text-xs text-slate-400 font-sans mt-0.5">
            Espelha 100% do CLI embarcado (`p7501_cli.c` — P7501 UART 115200 8N1) via USB CDC, Android OTG e BLE NUS.
          </p>
        </div>

        {history.length > 0 && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => {
              setInspectedResult(null);
              onClearHistory();
            }}
            className="text-xs text-slate-400 hover:text-rose-400 gap-1.5"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Limpar Histórico</span>
          </Button>
        )}
      </div>

      <Tabs
        value={activeTab}
        onValueChange={val => {
          setActiveTab(val);
          if (val !== 'memory') {
            onSelectCategory(val as PicoCommandCategory);
          }
        }}
      >
        <TabsList className="w-full h-auto flex-wrap justify-start gap-1">
          {COMMAND_TABS.map(({ value, label, Icon }) => (
            <TabsTrigger key={value} value={value} className="gap-1.5">
              <Icon className="w-3.5 h-3.5" />
              <span>{label}</span>
            </TabsTrigger>
          ))}
          <TabsTrigger value="memory" className="gap-1.5">
            <Layers className="w-3.5 h-3.5" />
            <span>Mapa Memória AM34</span>
          </TabsTrigger>
        </TabsList>

        {COMMAND_TABS.map(({ value }) => {
          const cmds = ssot.pico_commands.filter(c => c.category === value);
          return (
            <TabsContent key={value} value={value}>
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
                {cmds.map(cmd => (
                  <DiagnosticCommandCard
                    key={cmd.id}
                    cmd={cmd}
                    argValue={getArgForCmd(cmd)}
                    onArgChange={val => setArgMap(prev => ({ ...prev, [cmd.id]: val }))}
                    isBusy={isBusy}
                    isCurrent={runningCommandId === cmd.id}
                    onRun={() => {
                      setInspectedResult(null);
                      void onExecute(cmd.id, getArgForCmd(cmd));
                    }}
                  />
                ))}
              </div>
            </TabsContent>
          );
        })}

        <TabsContent value="memory">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <div className="rounded-xl bg-slate-950 border border-slate-800 p-3.5 space-y-2">
              <h4 className="text-xs font-bold text-cyan-300">Mapa de Memória Matsushita AM34</h4>
              <div className="overflow-x-auto">
                <table className="w-full text-[11px] font-mono">
                  <thead>
                    <tr className="text-left text-slate-500 border-b border-slate-800">
                      <th className="py-1.5 pr-2">Região</th>
                      <th className="py-1.5 pr-2">Início</th>
                      <th className="py-1.5 pr-2">Fim</th>
                      <th className="py-1.5">Permissão</th>
                    </tr>
                  </thead>
                  <tbody>
                    {ssot.memory_regions.map(reg => (
                      <tr key={reg.name} className="border-b border-slate-900 text-slate-300">
                        <td className="py-1.5 pr-2">{reg.name}</td>
                        <td className="py-1.5 pr-2 text-cyan-400">{reg.start}</td>
                        <td className="py-1.5 pr-2 text-slate-400">{reg.end}</td>
                        <td className="py-1.5 text-amber-300">{reg.perms}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            <div className="rounded-xl bg-slate-950 border border-slate-800 p-3.5 space-y-2">
              <h4 className="text-xs font-bold text-emerald-300">Geometrias Top-Boot Spansion NOR</h4>
              <div className="overflow-x-auto">
                <table className="w-full text-[11px] font-mono">
                  <thead>
                    <tr className="text-left text-slate-500 border-b border-slate-800">
                      <th className="py-1.5 pr-2">Chip</th>
                      <th className="py-1.5 pr-2">Base NVRAM</th>
                      <th className="py-1.5">Micro-Setores</th>
                    </tr>
                  </thead>
                  <tbody>
                    {ssot.flash_geometries.map(geo => (
                      <tr key={geo.size_mb} className="border-b border-slate-900 text-slate-300">
                        <td className="py-1.5 pr-2">{geo.chip_name}</td>
                        <td className="py-1.5 pr-2 text-cyan-400">{geo.nvram_base_hex}</td>
                        <td className="py-1.5 text-emerald-300">{geo.microsector_range}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </TabsContent>
      </Tabs>

      {error && (
        <Alert variant="destructive">
          <ShieldAlert className="w-4 h-4 text-rose-400 shrink-0" />
          <div>
            <AlertTitle>Erro na Execução Diagnóstica</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </div>
        </Alert>
      )}

      {isBusy && (
        <Alert variant="info">
          <Loader2 className="w-4 h-4 text-cyan-400 animate-spin shrink-0" />
          <div>
            <AlertTitle>Sonda Pico Executando {runningCommandId}</AlertTitle>
            <AlertDescription>
              Aguardando janela de captura / resposta determinística no barramento...
            </AlertDescription>
          </div>
        </Alert>
      )}

      {/* Structured Output & History Inspector */}
      <div className="rounded-xl bg-slate-950 border border-slate-800 p-4 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div className="flex items-center gap-2">
            <History className="w-4 h-4 text-cyan-400" />
            <span className="text-xs font-bold text-slate-200">
              Inspetor Estruturado de Diagnóstico
            </span>
          </div>

          {history.length > 0 && (
            <div className="flex items-center gap-1.5 flex-wrap">
              {history.slice(0, 6).map((item, idx) => (
                <button
                  key={`${item.raw_command}-${idx}`}
                  type="button"
                  onClick={() => setInspectedResult(item)}
                  className={`px-2 py-0.5 rounded text-[10px] border transition-colors ${
                    activeResult === item
                      ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 font-bold'
                      : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'
                  }`}
                >
                  {item.command_id} ({item.elapsed_ms}ms)
                </button>
              ))}
            </div>
          )}
        </div>

        {!activeResult ? (
          <EmptyState
            title="Nenhum diagnóstico executado nesta sessão"
            description="Selecione uma aba acima (ex: Elétrica P7501 -> PINCHECK) para auditar os 6 pinos, capturar boot UART ou executar probes OCD."
            className="min-h-[160px] p-4 border-slate-800/70"
          />
        ) : (
          <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={activeResult.success ? 'success' : 'destructive'}>
                {activeResult.success ? 'OK' : 'FALHA / ALERTA'}
              </Badge>
              <code className="text-xs text-cyan-300 font-bold">{activeResult.raw_command}</code>
              <Badge variant="secondary">{activeResult.elapsed_ms} ms</Badge>
              {activeResult.vdd_volts !== null && activeResult.vdd_volts !== undefined && (
                <Badge variant="default">VDD {activeResult.vdd_volts.toFixed(2)}V</Badge>
              )}
              {activeResult.detected_flash_mb && (
                <Badge variant="success">Flash {activeResult.detected_flash_mb} MB</Badge>
              )}
              {activeResult.xreset_rise_us && (
                <Badge variant="secondary">XRESET Rise: {activeResult.xreset_rise_us}</Badge>
              )}
              {activeResult.captured_bytes !== null && activeResult.captured_bytes !== undefined && (
                <Badge variant="secondary">{activeResult.captured_bytes} bytes UART/FMU</Badge>
              )}
            </div>

            {activeResult.pincheck_rows.length > 0 && (
              <div className="overflow-x-auto rounded-lg border border-slate-800">
                <table className="w-full text-[11px]">
                  <thead className="bg-slate-900 text-slate-400">
                    <tr>
                      <th className="text-left p-2">Pino</th>
                      <th className="text-left p-2">Hi-Z</th>
                      <th className="text-left p-2">Pull-Down</th>
                      <th className="text-left p-2">Pull-Up</th>
                      <th className="text-left p-2">Estado Elétrico</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activeResult.pincheck_rows.map(row => (
                      <tr key={row.pin_label} className="border-t border-slate-800/80 text-slate-200">
                        <td className="p-2 font-bold text-cyan-300">{row.pin_label}</td>
                        <td className="p-2">{row.hiz}</td>
                        <td className="p-2">{row.pull_down}</td>
                        <td className="p-2">{row.pull_up}</td>
                        <td className="p-2 font-sans text-slate-300">{row.state}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {activeResult.transition_rows.length > 0 && (
              <div className="overflow-x-auto rounded-lg border border-slate-800">
                <table className="w-full text-[11px]">
                  <thead className="bg-slate-900 text-slate-400">
                    <tr>
                      <th className="text-left p-2">Sinal</th>
                      <th className="text-left p-2">GPIO</th>
                      <th className="text-left p-2">Nível Final</th>
                      <th className="text-left p-2">Transições</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activeResult.transition_rows.map(row => (
                      <tr key={`${row.signal}-${row.gpio}`} className="border-t border-slate-800/80">
                        <td className="p-2 font-bold text-cyan-300">{row.signal}</td>
                        <td className="p-2 text-slate-400">{row.gpio}</td>
                        <td className="p-2 text-slate-200">{row.final_level ?? '-'}</td>
                        <td className="p-2 text-emerald-400 font-bold">{row.transitions}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {activeResult.dump_rows.length > 0 && (
              <div className="overflow-x-auto rounded-lg border border-slate-800">
                <table className="w-full text-[11px]">
                  <thead className="bg-slate-900 text-slate-400">
                    <tr>
                      <th className="text-left p-2">Endereço</th>
                      <th className="text-left p-2">Palavras 32-bit</th>
                      <th className="text-left p-2">ASCII LE</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activeResult.dump_rows.map(row => (
                      <tr key={row.address} className="border-t border-slate-800/80">
                        <td className="p-2 font-bold text-cyan-400">{row.address}</td>
                        <td className="p-2 text-slate-200">{row.words.join(' ')}</td>
                        <td className="p-2 text-amber-300">{row.ascii}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            <ScrollArea className="max-h-44 rounded-lg bg-slate-900/90 border border-slate-800 p-3">
              <pre className="text-[11px] text-slate-300 whitespace-pre-wrap break-words font-mono">
                {activeResult.raw_output}
              </pre>
            </ScrollArea>
          </div>
        )}
      </div>
    </Card>
  );
};
