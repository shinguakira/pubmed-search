import { useId } from "react";
import { HelpCircle } from "lucide-react";

import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

export type AppFilterMode = "include" | "exclude";

interface Props {
  value: string;
  mode: AppFilterMode;
  onValueChange: (v: string) => void;
  onModeChange: (m: AppFilterMode) => void;
  /** Pressing Enter in the keyword field re-submits with the staged value. */
  onSubmit?: () => void;
  /** Page-slice size after the filter (visible result count). */
  shownCount?: number;
  /** Page-slice size before the filter — from `unfiltered_count` on the response. */
  pageCount?: number;
  /** Whether the *applied* (URL-committed) filter is currently active. */
  active?: boolean;
}

export function AppFilterBar({
  value,
  mode,
  onValueChange,
  onModeChange,
  onSubmit,
  shownCount,
  pageCount,
  active = false,
}: Props) {
  const inputId = useId();
  return (
    <div className="border-b border-paper-rule/60 bg-paper-light/80">
      <div
        className="flex w-full flex-wrap items-center gap-2 px-4 py-2"
        data-testid="app-filter-bar"
      >
        <label
          htmlFor={inputId}
          className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-paper-brown"
        >
          App filter
        </label>

        <TooltipProvider delayDuration={120}>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label="What is the app filter?"
                data-testid="app-filter-hint"
                className="inline-flex h-5 w-5 items-center justify-center rounded-full text-paper-brown transition-colors hover:bg-paper-dark/40 hover:text-paper-rust"
              >
                <HelpCircle className="h-3.5 w-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent
              side="bottom"
              className="max-w-sm bg-paper-ink font-serif text-[12px] leading-snug text-paper-light"
            >
              Filters the page-slice <strong>on the backend</strong>, after PubMed responds. The
              PubMed query is unchanged, so the corpus total stays the same — only the listed rows
              shrink.
              <br />
              <br />
              <strong>Include</strong>: keep only results whose title, abstract, authors or journal
              contains the keyword.
              <br />
              <strong>Exclude</strong>: drop results that contain the keyword.
              <br />
              <br />
              Matching is case-insensitive substring. The filter is applied{" "}
              <strong>only when you press Search</strong> — typing here or flipping include/exclude
              does <em>not</em> auto-trigger a refetch.
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        <Input
          id={inputId}
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && onSubmit) {
              e.preventDefault();
              onSubmit();
            }
          }}
          placeholder="Keyword to include or exclude — press Search to apply"
          className="h-8 min-w-[200px] flex-1 rounded-none border-paper-rule bg-paper px-3 font-serif text-sm"
          data-testid="app-filter-input"
        />

        <Select value={mode} onValueChange={(v) => onModeChange(v as AppFilterMode)}>
          <SelectTrigger
            className="h-8 w-[120px]"
            data-testid="app-filter-mode"
            aria-label="App filter mode"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="include">Include</SelectItem>
            <SelectItem value="exclude">Exclude</SelectItem>
          </SelectContent>
        </Select>

        {active && shownCount !== undefined && pageCount !== undefined && (
          <span
            data-testid="app-filter-badge"
            className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-rust"
          >
            {shownCount} / {pageCount} shown
          </span>
        )}
      </div>
    </div>
  );
}
