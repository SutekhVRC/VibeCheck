import PageHeader from "@/components/PageHeader";
import { useCoreEventContext } from "@/context/CoreEvents";
import { OSC } from "@/data/constants";
import { DiagValue, useDiagnostics } from "@/hooks/useDiagnostics";
import Button from "@/layout/Button";
import Tooltip from "@/layout/Tooltip";
import { copyText, errorText } from "@/lib/clipboard";
import { cn } from "@/lib/utils";
import type { FeDiagLogEntry } from "@bindings/FeDiagLogEntry";
import type { FeDiagPort } from "@bindings/FeDiagPort";
import type { FeDiagToy } from "@bindings/FeDiagToy";
import type { FeVCToy } from "@bindings/FeVCToy";
import {
  Activity,
  ClipboardCopy,
  Eraser,
  Pause,
  Play,
  Power,
} from "lucide-react";
import { Fragment, ReactNode, useState } from "react";
import { toast } from "sonner";

/** The OSC input's module id (src-tauri toy_graph.rs OSC_MODULE) */
const OSC_MODULE = "osc";

/** Log entries from the OSCQuery lookup (src-tauri osc/avatar_query.rs) */
const OSCQUERY_MODULE = "oscquery";

/** Most log lines rendered at once (newest first) */
const LOG_ROWS = 500;

type Tab = "values" | "log";

/*
 * Live diagnostics: every value VibeCheck receives (e.g. each avatar
 * parameter), what each toy's settings make of it, and a log of every OSC
 * message, including ones no toy uses, plus the values VibeCheck read from
 * VRChat's OSCQuery because VRChat never sends them. VibeCheck keeps the OSC port open the
 * whole time, so testers never need a separate OSC tool (switching tools
 * while VRChat runs can make VRChat stop sending OSC until it restarts).
 */
export default function Diagnostics({ toys }: { toys: FeVCToy[] }) {
  const { isEnabled, toggleIsEnabled } = useCoreEventContext();
  const [tab, setTab] = useState<Tab>("values");
  const [paused, setPaused] = useState(false);
  const [filter, setFilter] = useState("");
  const { view, clear } = useDiagnostics(paused);

  const needle = filter.trim().toLowerCase();
  const matches = (text: string) =>
    needle === "" || text.toLowerCase().includes(needle);

  const values = view.values
    .filter((v) => matches(v.port))
    .sort((a, b) => a.port.localeCompare(b.port));
  const log = view.log.filter((e) => matches(`${e.label} ${e.value}`));

  async function copy() {
    const text =
      tab === "values"
        ? valuesText(values, view.toys, toys)
        : log.map(logLine).join("\n");
    try {
      await copyText(text);
      toast.info("Copied to clipboard");
    } catch (e) {
      toast.error(`Could not copy to clipboard!\n${errorText(e)}`);
    }
  }

  return (
    <div className="flex h-full w-full flex-col">
      <PageHeader
        className="px-2 pb-2"
        icon={<Activity className="h-5 w-5" />}
        title="Diagnostics"
        subtitle="Live values from VRChat and what your toys make of them"
      />

      {!isEnabled ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-3 px-6 text-center">
          <Activity className="h-8 w-8 text-theme-muted" />
          <div className="text-theme-text">
            Turn VibeCheck on to see live values
          </div>
          <div className="max-w-md text-sm text-theme-muted">
            While it&rsquo;s on, VibeCheck listens to VRChat on your OSC port
            and shows everything that arrives here, so you don&rsquo;t need a
            separate OSC tool.
          </div>
          <Button onClick={toggleIsEnabled} className="flex items-center gap-2">
            <Power className="h-4 w-4" />
            Turn on
          </Button>
        </div>
      ) : (
        <>
          <div className="flex shrink-0 flex-wrap items-center gap-2 px-2 pb-2">
            <TabButton
              active={tab === "values"}
              onClick={() => setTab("values")}
            >
              Values
            </TabButton>
            <TabButton active={tab === "log"} onClick={() => setTab("log")}>
              OSC log
            </TabButton>
            <input
              type="search"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter, e.g. OGB/ or Boop"
              aria-label="Filter"
              className="min-w-[8rem] flex-1 rounded-full border border-theme-border bg-theme-surface px-3 py-1 text-sm text-theme-text placeholder:text-theme-muted focus:border-theme-primary focus:outline-none"
            />
            <IconButton
              tooltip={paused ? "Resume live updates" : "Pause live updates"}
              onClick={() => setPaused((p) => !p)}
              active={paused}
            >
              {paused ? (
                <Play className="h-4 w-4" />
              ) : (
                <Pause className="h-4 w-4" />
              )}
            </IconButton>
            <IconButton
              tooltip="Clear (values come back when they next change)"
              onClick={clear}
            >
              <Eraser className="h-4 w-4" />
            </IconButton>
            <IconButton
              tooltip="Copy what's shown, e.g. for a bug report"
              onClick={copy}
            >
              <ClipboardCopy className="h-4 w-4" />
            </IconButton>
          </div>

          <div className="scrollbar min-h-0 w-full flex-1 overflow-y-auto">
            <div className="flex flex-col gap-4 px-2 pb-4 pt-1">
              {tab === "values" ? (
                <ValuesTab
                  values={values}
                  toys={view.toys}
                  toyNames={toys}
                  matches={matches}
                />
              ) : (
                <LogTab log={log} />
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

function ValuesTab({
  values,
  toys,
  toyNames,
  matches,
}: {
  values: DiagValue[];
  toys: FeDiagToy[];
  toyNames: FeVCToy[];
  matches: (text: string) => boolean;
}) {
  // Group by input: OSC first, then any other input in name order
  const modules = [...new Set(values.map((v) => v.module))].sort((a, b) =>
    a === OSC_MODULE ? -1 : b === OSC_MODULE ? 1 : a.localeCompare(b),
  );
  const now = Date.now();

  return (
    <>
      {modules.length === 0 && (
        <Section title="Avatar parameters">
          <p className="text-theme-muted">
            Nothing yet. Values show up here as VRChat sends them: move, or
            change a parameter on your avatar.
          </p>
        </Section>
      )}
      {modules.map((module) => {
        const rows = values.filter((v) => v.module === module);
        return (
          <Section
            key={module}
            title={
              module === OSC_MODULE ? "Avatar parameters" : `Input: ${module}`
            }
            hint={`${rows.length} ${rows.length === 1 ? "value" : "values"}`}
          >
            <div className="grid grid-cols-[minmax(0,1fr)_7rem_3.5rem_3.5rem] items-center gap-x-3 gap-y-1">
              <ColumnHead>Parameter</ColumnHead>
              <ColumnHead>Value</ColumnHead>
              <ColumnHead right>Rate</ColumnHead>
              <ColumnHead right>Changed</ColumnHead>
              {rows.map((v) => (
                <ValueRow key={v.port} value={v} now={now} />
              ))}
            </div>
          </Section>
        );
      })}

      {toys.map((toy) => (
        <ToySection
          key={toy.toy_id}
          toy={toy}
          name={
            toyNames.find((t) => t.toy_id === toy.toy_id)?.toy_name ??
            `Toy ${toy.toy_id}`
          }
          matches={matches}
        />
      ))}
    </>
  );
}

/**
 * Copies a parameter's name. Avatar parameters are copied without
 * /avatar/parameters/, the way a toy's parameter field takes them.
 */
async function copyParameter(name: string) {
  try {
    await copyText(name);
    toast.info(`Copied ${name}`);
  } catch (e) {
    toast.error(`Could not copy to clipboard!\n${errorText(e)}`);
  }
}

function ValueRow({ value, now }: { value: DiagValue; now: number }) {
  const short = value.port.startsWith(OSC.PARAM_PREFIX)
    ? value.port.slice(OSC.PARAM_PREFIX.length)
    : value.port;
  return (
    <>
      <span className="flex min-w-0 items-center gap-1.5">
        <button
          type="button"
          onClick={() => copyParameter(short)}
          className="truncate rounded font-mono text-xs transition-colors hover:text-theme-primary"
          title={`${value.port}\nClick to copy`}
        >
          {short}
        </button>
        {value.fromOscQuery && (
          <span
            className="shrink-0 rounded border border-theme-border px-1 text-[0.6rem] leading-4 text-theme-muted"
            title="Read from VRChat's OSCQuery: VRChat hasn't sent it over OSC. It's set when the avatar loads and VibeCheck needs it, e.g. an SPS plug's length."
          >
            OSCQuery
          </span>
        )}
      </span>
      <Level value={value.value} isBool={value.isBool} />
      <span className="text-right text-xs tabular-nums text-theme-muted">
        {value.rate >= 0.1 ? `${value.rate.toFixed(1)}/s` : "-"}
      </span>
      <span className="text-right text-xs tabular-nums text-theme-muted">
        {age(now - value.changedAt)}
      </span>
    </>
  );
}

/**
 * One toy's graph. Each motor has a chain per input (its input processor,
 * and each parameter): what each step makes of the input, then what that
 * chain last sent to the motor.
 */
function ToySection({
  toy,
  name,
  matches,
}: {
  toy: FeDiagToy;
  name: string;
  matches: (text: string) => boolean;
}) {
  const motors = [...new Set(toy.sent.map((s) => s.motor))].map((motor) => {
    const chains = toy.sent
      .filter((s) => s.motor === motor)
      .map((sent) => {
        const prefix = chainPrefix(sent.from);
        const steps = toy.ports.filter((p) => p.name.startsWith(`${prefix}/`));
        return { sent, prefix, steps };
      });
    return { motor, motorName: motor.split("/").slice(1).join("/"), chains };
  });
  const shown = motors.filter(
    (m) =>
      matches(`${name} ${m.motorName}`) ||
      m.chains.some((c) => c.steps.some((s) => matches(s.name))),
  );
  if (shown.length === 0) return null;

  return (
    <Section title={name} hint="Toy">
      <div className="flex flex-col gap-3">
        {shown.map(({ motor, motorName, chains }) => (
          <div key={motor}>
            <div className="mb-1 text-xs font-semibold text-theme-secondary">
              {motorName.replace(/_/g, " ")}
            </div>
            <div className="grid grid-cols-[minmax(0,1fr)_7rem] items-center gap-x-3 gap-y-1">
              {chains.map(({ sent, prefix, steps }) => (
                <Fragment key={sent.from}>
                  {steps.map((step) => (
                    <StepRow key={step.name} step={step} motor={motor} />
                  ))}
                  <span className="text-xs font-semibold text-theme-text">
                    {stepLabel(prefix.slice(motor.length + 1))} sent
                    <span className="font-normal text-theme-muted">
                      {" "}
                      (before master intensity)
                    </span>
                  </span>
                  <Level value={sent.value} />
                </Fragment>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Section>
  );
}

/**
 * The chain a sink's source belongs to: its path without the last node and
 * port, e.g. `toy:lush:0/Vibrator_0/param0` from
 * `toy:lush:0/Vibrator_0/param0/levels/out`
 */
function chainPrefix(from: string): string {
  return from.split("/").slice(0, -2).join("/");
}

function StepRow({ step, motor }: { step: FeDiagPort; motor: string }) {
  return (
    <>
      <span className="truncate text-xs text-theme-muted" title={step.name}>
        {stepLabel(step.name.slice(motor.length + 1))}
      </span>
      <Level value={step.value} />
    </>
  );
}

/**
 * A readable name for a graph node output, from its path under the motor:
 * `input_processor/out`, `param0/mode/out`, `param0/levels/out` and so on
 */
function stepLabel(path: string): string {
  const parts = path.split("/").filter((p) => p !== "out");
  const words: Record<string, string> = {
    input_processor: "Input processor",
    round: "rounded",
    mode: "mode",
    levels: "levels",
  };
  return parts
    .map((p) => {
      const param = /^param(\d+)$/.exec(p);
      if (param) return `Parameter ${Number(param[1]) + 1}`;
      return words[p] ?? p;
    })
    .join(" › ");
}

function LogTab({ log }: { log: FeDiagLogEntry[] }) {
  if (log.length === 0) {
    return (
      <Section title="OSC log">
        <p className="text-theme-muted">
          Nothing yet. Every OSC message VibeCheck receives shows up here,
          including ones no toy uses. Values VibeCheck reads from VRChat&rsquo;s
          OSCQuery show up here too, marked OSCQuery.
        </p>
      </Section>
    );
  }
  const rows = log.slice(-LOG_ROWS).reverse();
  return (
    <Section
      title="OSC log"
      hint={`Newest first${log.length > LOG_ROWS ? `, last ${LOG_ROWS}` : ""}`}
    >
      <div className="flex flex-col font-mono text-xs">
        {rows.map((e) => (
          <div
            key={e.seq}
            className={cn(
              "grid grid-cols-[4.5rem_minmax(0,1fr)_auto] gap-x-3 py-0.5",
              e.label.startsWith("/avatar/change") && "text-theme-secondary",
              e.module === OSCQUERY_MODULE && "text-theme-primary",
            )}
            title={
              e.module === OSCQUERY_MODULE
                ? "Read from VRChat's OSCQuery: VRChat never sent it over OSC"
                : undefined
            }
          >
            <span className="text-right tabular-nums text-theme-muted">
              {seconds(e.at_ms)}
            </span>
            <span className="truncate" title={e.label}>
              {e.label}
            </span>
            <span className="text-theme-muted">{e.value}</span>
          </div>
        ))}
      </div>
    </Section>
  );
}

/** A 0 to 1 level as a bar and number, true/false for booleans */
function Level({
  value,
  isBool = false,
}: {
  value: number | null;
  isBool?: boolean;
}) {
  if (value === null) {
    return <span className="text-xs text-theme-muted">nothing yet</span>;
  }
  if (isBool) {
    return (
      <span
        className={cn(
          "text-xs font-semibold",
          value >= 0.5 ? "text-theme-success" : "text-theme-muted",
        )}
      >
        {value >= 0.5 ? "true" : "false"}
      </span>
    );
  }
  const fill = Math.min(Math.max(value, 0), 1);
  return (
    <span className="flex items-center gap-2">
      <span className="h-1.5 flex-1 overflow-hidden rounded-full bg-theme-border/60">
        <span
          className="block h-full rounded-full bg-theme-primary"
          style={{ width: `${fill * 100}%` }}
        />
      </span>
      <span className="w-10 text-right text-xs tabular-nums">
        {formatNumber(value)}
      </span>
    </span>
  );
}

function Section({
  title,
  hint,
  children,
}: {
  title: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-2xl border border-theme-border/60 bg-theme-raised p-4 text-left text-sm">
      <div className="mb-3 flex items-baseline justify-between gap-2">
        <h2 className="text-base font-semibold text-theme-secondary">
          {title}
        </h2>
        {hint && <span className="text-xs text-theme-muted">{hint}</span>}
      </div>
      {children}
    </section>
  );
}

function ColumnHead({
  children,
  right = false,
}: {
  children: ReactNode;
  right?: boolean;
}) {
  return (
    <span
      className={cn(
        "text-[0.7rem] font-semibold uppercase tracking-wider text-theme-muted",
        right && "text-right",
      )}
    >
      {children}
    </span>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      className={cn(
        "rounded-full px-3 py-1 text-sm font-semibold transition-colors",
        active
          ? "bg-theme-primary/20 text-theme-primary ring-1 ring-theme-primary/60"
          : "text-theme-muted hover:bg-theme-raised hover:text-theme-text",
      )}
    >
      {children}
    </button>
  );
}

function IconButton({
  tooltip,
  onClick,
  active = false,
  children,
}: {
  tooltip: string;
  onClick: () => void;
  active?: boolean;
  children: ReactNode;
}) {
  return (
    <Tooltip text={tooltip}>
      <button
        type="button"
        onClick={onClick}
        aria-label={tooltip}
        aria-pressed={active}
        className={cn(
          "rounded-full p-2 transition-colors",
          active
            ? "bg-theme-primary/20 text-theme-primary"
            : "text-theme-muted hover:bg-theme-raised hover:text-theme-primary",
        )}
      >
        {children}
      </button>
    </Tooltip>
  );
}

/** Up to 3 decimals, without trailing zeros */
function formatNumber(value: number): string {
  return Number(value.toFixed(3)).toString();
}

function age(ms: number): string {
  if (ms < 1000) return "now";
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  return m < 60 ? `${m}m` : `${Math.floor(m / 60)}h`;
}

/** Seconds since diagnostics started, like osc-tool's timestamps */
function seconds(ms: number): string {
  return `${(ms / 1000).toFixed(3)}s`;
}

/** A log entry the way `osc-tool listen` prints a message */
function logLine(e: FeDiagLogEntry): string {
  const stamp = seconds(e.at_ms).padStart(10);
  return e.value ? `${stamp}  ${e.label}  ${e.value}` : `${stamp}  ${e.label}`;
}

/** The Values tab as plain text, for bug reports */
function valuesText(
  values: DiagValue[],
  toys: FeDiagToy[],
  toyNames: FeVCToy[],
): string {
  const lines = values.map(
    (v) =>
      `${v.module}  ${v.port}  ${v.isBool ? (v.value >= 0.5 ? "true" : "false") : formatNumber(v.value)}${v.fromOscQuery ? "  (from OSCQuery)" : ""}`,
  );
  for (const toy of toys) {
    const name =
      toyNames.find((t) => t.toy_id === toy.toy_id)?.toy_name ??
      `Toy ${toy.toy_id}`;
    lines.push("", `${name}:`);
    for (const p of toy.ports) {
      lines.push(
        `  ${p.name}  ${p.value === null ? "-" : formatNumber(p.value)}`,
      );
    }
    for (const sent of toy.sent) {
      lines.push(
        `  ${sent.from} -> ${sent.motor}  ${sent.value === null ? "-" : formatNumber(sent.value)}`,
      );
    }
  }
  return lines.join("\n");
}
