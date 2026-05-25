// Hand-rolled SVG icons for the article-modal newsletter animation.
// Designed so that individual parts (wings, flap, blade) can be
// targeted from CSS for keyframed motion — lucide's flat single-path
// icons don't give us enough hinge points.

interface IconProps {
  className?: string;
}

/* Carrier bird, top-down view.
   Wings are split into `.bird-wing-left` / `.bird-wing-right` groups
   so each rotates around its own body-side hinge. */
export function CarrierBird({ className }: IconProps) {
  return (
    <svg viewBox="0 0 80 60" className={className} fill="currentColor">
      {/* tail */}
      <path d="M22 32 L8 26 L8 38 Z" />
      {/* body */}
      <ellipse cx="38" cy="32" rx="16" ry="7" />
      {/* head */}
      <circle cx="58" cy="28" r="6" />
      {/* eye highlight + pupil */}
      <circle cx="60" cy="26" r="1.4" fill="#fff" />
      <circle cx="60.4" cy="26" r="0.8" fill="#1a1a1a" />
      {/* beak */}
      <path d="M64 28 L74 27 L64 30 Z" fill="hsl(38 75% 48%)" />
      {/* mini envelope held in beak */}
      <g transform="translate(70 22) rotate(8)">
        <rect
          width="16"
          height="11"
          fill="hsl(35 30% 86%)"
          stroke="hsl(20 35% 30%)"
          strokeWidth="0.6"
        />
        <path d="M0 0 L8 7 L16 0" fill="none" stroke="hsl(20 35% 30%)" strokeWidth="0.6" />
      </g>
      {/* left wing — pivots around (34, 30) */}
      <g className="bird-wing bird-wing-left">
        <path d="M34 30 Q26 6 12 16 Q22 28 34 32 Z" />
        <path
          d="M28 22 Q22 14 16 18"
          fill="none"
          stroke="hsl(20 50% 22%)"
          strokeWidth="0.6"
          opacity="0.6"
        />
      </g>
      {/* right wing — back wing, half-opacity for depth */}
      <g className="bird-wing bird-wing-right">
        <path d="M42 30 Q50 6 64 16 Q54 26 42 32 Z" opacity="0.7" />
      </g>
    </svg>
  );
}

/* A proper sealed envelope, with separate flap path so we could
   animate it independently if we ever wanted to. Currently used in
   Phase 1 (the icon flying in from off-screen). */
export function SealedEnvelope({ className }: IconProps) {
  return (
    <svg viewBox="0 0 120 90" className={className}>
      {/* envelope body */}
      <rect
        x="6"
        y="22"
        width="108"
        height="62"
        fill="hsl(35 30% 82%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2.5"
      />
      {/* lower diagonal seams */}
      <path
        d="M6 84 L42 55"
        fill="none"
        stroke="hsl(20 35% 30%)"
        strokeWidth="1.5"
        opacity="0.55"
      />
      <path
        d="M114 84 L78 55"
        fill="none"
        stroke="hsl(20 35% 30%)"
        strokeWidth="1.5"
        opacity="0.55"
      />
      {/* the flap (drawn closed) */}
      <path
        d="M6 22 L60 60 L114 22 Z"
        fill="hsl(35 28% 75%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2.5"
      />
      {/* wax seal */}
      <circle
        cx="60"
        cy="50"
        r="9"
        fill="hsl(0 65% 36%)"
        stroke="hsl(0 60% 22%)"
        strokeWidth="1.4"
      />
      <text
        x="60"
        y="54"
        textAnchor="middle"
        fontSize="11"
        fill="hsl(0 60% 18%)"
        fontFamily="serif"
        fontWeight="700"
      >
        P
      </text>
    </svg>
  );
}

/* Letter-opener / paper knife. Used as a one-shot overlay that
   sweeps across the modal flap before the flap rotates open. */
export function LetterOpener({ className }: IconProps) {
  return (
    <svg viewBox="0 0 100 16" className={className} aria-hidden>
      {/* wooden handle */}
      <rect x="0" y="3" width="26" height="10" rx="1.5" fill="hsl(20 50% 20%)" />
      <rect x="2" y="5" width="22" height="6" fill="hsl(20 42% 30%)" />
      <line x1="6" y1="5" x2="6" y2="11" stroke="hsl(20 60% 12%)" strokeWidth="0.4" />
      <line x1="14" y1="5" x2="14" y2="11" stroke="hsl(20 60% 12%)" strokeWidth="0.4" />
      <line x1="22" y1="5" x2="22" y2="11" stroke="hsl(20 60% 12%)" strokeWidth="0.4" />
      {/* bolster */}
      <rect x="26" y="2" width="3" height="12" fill="hsl(45 25% 70%)" />
      {/* blade */}
      <path
        d="M29 4 L92 6.5 L100 8 L92 9.5 L29 12 Z"
        fill="hsl(0 0% 78%)"
        stroke="hsl(0 0% 45%)"
        strokeWidth="0.4"
      />
      {/* blade highlight */}
      <path d="M30 6.5 L90 7.5 L90 8.5 L30 9.5 Z" fill="hsl(0 0% 96%)" opacity="0.85" />
    </svg>
  );
}

/* A folded newsletter — used for the `unfold` variant's flying icon. */
export function FoldedNewsletter({ className }: IconProps) {
  return (
    <svg viewBox="0 0 100 80" className={className}>
      {/* back page */}
      <rect
        x="14"
        y="10"
        width="72"
        height="64"
        fill="hsl(40 36% 90%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2"
      />
      {/* front page */}
      <rect
        x="8"
        y="14"
        width="72"
        height="64"
        fill="hsl(40 40% 94%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2"
      />
      {/* masthead */}
      <rect x="16" y="20" width="56" height="6" fill="hsl(20 50% 26%)" />
      {/* faux columns of text */}
      {[34, 40, 46, 52, 58, 64, 70].map((y) => (
        <line
          key={y}
          x1="16"
          y1={y}
          x2={y % 12 === 0 ? 60 : 72}
          y2={y}
          stroke="hsl(20 35% 30%)"
          strokeWidth="0.8"
          opacity="0.7"
        />
      ))}
      {/* fold crease */}
      <line
        x1="44"
        y1="14"
        x2="44"
        y2="78"
        stroke="hsl(20 35% 30%)"
        strokeWidth="0.6"
        opacity="0.4"
        strokeDasharray="2 2"
      />
    </svg>
  );
}

/* A small rolled-up letter — used for the `drop` variant's icon. */
export function RolledLetter({ className }: IconProps) {
  return (
    <svg viewBox="0 0 100 80" className={className}>
      {/* paper sheet */}
      <rect
        x="14"
        y="14"
        width="72"
        height="60"
        fill="hsl(40 42% 94%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2"
      />
      {/* lines of text */}
      {[24, 32, 40, 48, 56, 64].map((y, i) => (
        <line
          key={y}
          x1="22"
          y1={y}
          x2={i % 2 === 0 ? 76 : 64}
          y2={y}
          stroke="hsl(20 35% 30%)"
          strokeWidth="0.9"
          opacity="0.65"
        />
      ))}
      {/* curled top edge */}
      <path
        d="M14 14 Q50 4 86 14 Q50 22 14 14 Z"
        fill="hsl(40 36% 88%)"
        stroke="hsl(20 35% 30%)"
        strokeWidth="2"
      />
      {/* ribbon */}
      <rect x="44" y="14" width="12" height="60" fill="hsl(0 60% 38%)" opacity="0.85" />
      <rect
        x="44"
        y="14"
        width="12"
        height="60"
        fill="none"
        stroke="hsl(0 60% 22%)"
        strokeWidth="0.6"
      />
    </svg>
  );
}
