import { useState } from "react";
import { Check, ChevronDown } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

export interface Filters {
  textAvailability: string[];
  articleAttribute: string[];
  articleTypes: string[];
  pubDate: string; // "any" | "1y" | "5y" | "10y" | "custom"
  customFrom?: string;
  customTo?: string;
  species: string[];
  languages: string[];
  sex: string[];
  age: string[];
  other: string[];
}

export const emptyFilters: Filters = {
  textAvailability: [],
  articleAttribute: [],
  articleTypes: [],
  pubDate: "any",
  species: [],
  languages: [],
  sex: [],
  age: [],
  other: [],
};

interface SectionDef {
  key: keyof Filters;
  title: string;
  options: { value: string; label: string }[];
}

// Matches PubMed's order: Publication date → Text availability → Article attribute
// → Article type → Article Language → Species → Sex → Age → Other.
const SECTIONS: SectionDef[] = [
  {
    key: "textAvailability",
    title: "TEXT AVAILABILITY",
    options: [
      { value: "fha[Filter]", label: "Abstract" },
      { value: "free full text[Filter]", label: "Free full text" },
      { value: "full text[Filter]", label: "Full text" },
    ],
  },
  {
    key: "articleAttribute",
    title: "ARTICLE ATTRIBUTE",
    options: [
      { value: "hasabstract[Filter]", label: "Has abstract" },
      { value: "associated data[Filter]", label: "Associated data" },
    ],
  },
  {
    key: "articleTypes",
    title: "ARTICLE TYPE",
    options: [
      { value: "Books and Documents[pt]", label: "Books and Documents" },
      { value: "Clinical Trial[pt]", label: "Clinical Trial" },
      { value: "Meta-Analysis[pt]", label: "Meta-Analysis" },
      { value: "Randomized Controlled Trial[pt]", label: "Randomized Controlled Trial" },
      { value: "Review[pt]", label: "Review" },
      { value: "Systematic Review[pt]", label: "Systematic Review" },
    ],
  },
  {
    key: "languages",
    title: "ARTICLE LANGUAGE",
    options: [
      { value: "english[lang]", label: "English" },
      { value: "japanese[lang]", label: "Japanese" },
      { value: "french[lang]", label: "French" },
      { value: "german[lang]", label: "German" },
      { value: "spanish[lang]", label: "Spanish" },
      { value: "chinese[lang]", label: "Chinese" },
    ],
  },
  {
    key: "species",
    title: "SPECIES",
    options: [
      { value: "humans[mesh]", label: "Humans" },
      { value: "animals[mesh]", label: "Other animals" },
    ],
  },
  {
    key: "sex",
    title: "SEX",
    options: [
      { value: "female[mesh]", label: "Female" },
      { value: "male[mesh]", label: "Male" },
    ],
  },
  {
    key: "age",
    title: "AGE",
    options: [
      { value: "infant[mesh]", label: "Infant: birth–23 months" },
      { value: "child[mesh]", label: "Child: 6–12 years" },
      { value: "adolescent[mesh]", label: "Adolescent: 13–18 years" },
      { value: "adult[mesh]", label: "Adult: 19+ years" },
      { value: "aged[mesh]", label: "Aged: 65+ years" },
    ],
  },
  {
    key: "other",
    title: "OTHER",
    options: [
      { value: "humans[mesh] NOT animals[mesh]", label: "Exclude animal studies" },
      { value: "medline[sb]", label: "MEDLINE" },
      { value: "pubmed pmc[sb]", label: "PubMed Central" },
    ],
  },
];

const DATE_OPTIONS = [
  { value: "any", label: "Any time" },
  { value: "1y", label: "1 year" },
  { value: "5y", label: "5 years" },
  { value: "10y", label: "10 years" },
  { value: "custom", label: "Custom range" },
];

interface Props {
  value: Filters;
  onChange: (next: Filters) => void;
}

export function FiltersSidebar({ value, onChange }: Props) {
  const toggleArray = (key: keyof Filters, v: string) => {
    const current = value[key] as string[];
    const next = current.includes(v) ? current.filter((x) => x !== v) : [...current, v];
    onChange({ ...value, [key]: next });
  };

  const activeCount =
    value.textAvailability.length +
    value.articleAttribute.length +
    value.articleTypes.length +
    value.languages.length +
    value.species.length +
    value.sex.length +
    value.age.length +
    value.other.length +
    (value.pubDate !== "any" ? 1 : 0);

  return (
    <aside className="space-y-1 font-serif text-sm text-paper-ink">
      <div className="flex items-baseline justify-between border-b border-double border-paper-rule/70 pb-1.5">
        <h2 className="font-serif text-[11px] font-bold uppercase tracking-[0.3em] text-paper-brown">
          Filters
        </h2>
        {activeCount > 0 && (
          <button
            type="button"
            className="font-serif text-[11px] text-paper-rust hover:underline"
            onClick={() => onChange(emptyFilters)}
          >
            Reset ({activeCount})
          </button>
        )}
      </div>

      <FilterSection title="PUBLICATION DATE" defaultOpen>
        <div className="space-y-1.5 py-1">
          {DATE_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() => onChange({ ...value, pubDate: opt.value })}
              className={cn(
                "flex w-full items-center justify-between rounded-sm px-2 py-1 text-left text-[13px] hover:bg-paper-dark/50",
                value.pubDate === opt.value && "bg-paper-dark/70 font-medium text-paper-rust",
              )}
            >
              <span>{opt.label}</span>
              {value.pubDate === opt.value && <Check className="h-3.5 w-3.5" />}
            </button>
          ))}
          {value.pubDate === "custom" && (
            <div className="grid grid-cols-2 gap-2 px-2 pt-2">
              <div>
                <Label className="text-xs text-muted-foreground">From</Label>
                <Input
                  type="number"
                  placeholder="2020"
                  value={value.customFrom ?? ""}
                  onChange={(e) => onChange({ ...value, customFrom: e.target.value })}
                  className="h-8"
                />
              </div>
              <div>
                <Label className="text-xs text-muted-foreground">To</Label>
                <Input
                  type="number"
                  placeholder="2025"
                  value={value.customTo ?? ""}
                  onChange={(e) => onChange({ ...value, customTo: e.target.value })}
                  className="h-8"
                />
              </div>
            </div>
          )}
        </div>
      </FilterSection>

      {SECTIONS.map((s) => (
        <FilterSection key={s.key} title={s.title}>
          <div className="space-y-1 py-1">
            {s.options.map((opt) => {
              const checked = (value[s.key] as string[]).includes(opt.value);
              return (
                <label
                  key={opt.value}
                  className="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-0.5 hover:bg-paper-dark/50"
                >
                  <Checkbox
                    checked={checked}
                    onCheckedChange={() => toggleArray(s.key, opt.value)}
                    className="border-paper-brown data-[state=checked]:bg-paper-rust data-[state=checked]:text-paper-light"
                  />
                  <span className="text-[13px] text-paper-ink">{opt.label}</span>
                </label>
              );
            })}
          </div>
        </FilterSection>
      ))}
    </aside>
  );
}

function FilterSection({
  title,
  children,
  defaultOpen,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen ?? true);
  return (
    <div className="border-b border-paper-rule/50">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center justify-between py-1.5 text-left font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-paper-brown hover:text-paper-ink"
      >
        {title}
        <ChevronDown
          className={cn("h-3 w-3 transition-transform", open ? "rotate-180" : "rotate-0")}
        />
      </button>
      {open && <div className="pb-1.5">{children}</div>}
    </div>
  );
}

export function filtersToQueryFragments(f: Filters): string[] {
  const out: string[] = [];
  out.push(...f.textAvailability);
  out.push(...f.articleAttribute);
  out.push(...f.articleTypes);
  out.push(...f.languages);
  out.push(...f.species);
  out.push(...f.sex);
  out.push(...f.age);
  out.push(...f.other);
  if (f.pubDate !== "any") {
    if (f.pubDate === "custom") {
      const from = (f.customFrom || "").trim();
      const to = (f.customTo || "").trim();
      if (from && to) out.push(`${from}:${to}[dp]`);
      else if (from) out.push(`${from}:3000[dp]`);
      else if (to) out.push(`1800:${to}[dp]`);
    } else {
      const years = parseInt(f.pubDate, 10);
      if (!Number.isNaN(years)) {
        const now = new Date();
        const fromYear = now.getFullYear() - years;
        out.push(`${fromYear}:${now.getFullYear()}[dp]`);
      }
    }
  }
  return out;
}
