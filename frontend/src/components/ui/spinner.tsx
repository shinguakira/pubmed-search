import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface SpinnerProps {
  size?: "sm" | "md" | "lg";
  /// Optional italic serif label rendered to the right of the spinner.
  label?: string;
  className?: string;
}

const SIZE = {
  sm: "h-4 w-4",
  md: "h-6 w-6",
  lg: "h-10 w-10",
};

const LABEL_SIZE = {
  sm: "text-xs",
  md: "text-sm",
  lg: "text-base",
};

export function Spinner({ size = "md", label, className }: SpinnerProps) {
  return (
    <output
      aria-live="polite"
      className={cn("inline-flex items-center gap-2 text-paper-rust", className)}
    >
      <Loader2 className={cn("animate-spin", SIZE[size])} />
      {label && (
        <span className={cn("font-serif italic text-paper-brown", LABEL_SIZE[size])}>{label}</span>
      )}
      {!label && <span className="sr-only">Loading…</span>}
    </output>
  );
}
