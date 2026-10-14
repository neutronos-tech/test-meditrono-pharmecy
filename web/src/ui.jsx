import { useEffect, useState } from "react";

// Owner-editable site details. Leave a value empty to hide it.
export const SITE = {
  contactEmail: "",
  contactPhone: "",
  downloadUrl: "https://github.com/neutronos-tech/test-meditrono-pharmecy/releases/latest",
};

export function usePath() {
  const [path, setPath] = useState(window.location.pathname);
  useEffect(() => {
    const onPop = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);
  const go = (to) => {
    window.history.pushState(null, "", to);
    setPath(to);
    window.scrollTo(0, 0);
  };
  return [path, go];
}

export function Link({ to, go, className = "", children }) {
  return (
    <a
      href={to}
      className={className}
      onClick={(event) => {
        if (event.metaKey || event.ctrlKey) return;
        event.preventDefault();
        go(to);
      }}
    >
      {children}
    </a>
  );
}

export const pill = "inline-flex items-center justify-center gap-2 rounded-full px-5 py-2.5 text-sm font-medium transition";
export const dark = `${pill} bg-ink text-white hover:bg-black`;
export const light = `${pill} border border-black/10 bg-white text-ink hover:bg-cream`;
export const green = `${pill} bg-brand text-white hover:bg-brand-dark`;

export function Eyebrow({ children, invert }) {
  return (
    <p className={`flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.18em] ${invert ? "text-white/50" : "text-muted"}`}>
      <span className={`h-px w-6 ${invert ? "bg-white/30" : "bg-black/20"}`} /> {children}
    </p>
  );
}

export function Tag({ children, invert }) {
  return (
    <span className={`rounded-full border px-3 py-1 text-[11px] ${invert ? "border-white/25 text-white/80" : "border-black/15 text-ink/70"}`}>
      {children}
    </span>
  );
}

const SCENES = {
  meadow: { sky: ["#dfe9f2", "#f4f1e8"], hills: ["#9cc9a0", "#5fa36f", "#2f8a63"], sun: "#f7efdc" },
  dusk: { sky: ["#2a3a34", "#7d9a86"], hills: ["#5d8a6c", "#3f6e52", "#1f4d38"], sun: "#d8bb7e" },
  mint: { sky: ["#e4f2ea", "#fbf7ef"], hills: ["#b7dcc4", "#7fbf95", "#4c9f72"], sun: "#ffffff" },
  gold: { sky: ["#f7efdc", "#fbf7ef"], hills: ["#d8e6b8", "#a9c98b", "#6fa86c"], sun: "#d8bb7e" },
};

/** Soft rolling-hills artwork that stands in for photography. */
export function Landscape({ scene = "meadow", className = "", children, capsules = true }) {
  const s = SCENES[scene];
  const id = `sky-${scene}`;
  return (
    <div className={`relative overflow-hidden rounded-[28px] ${className}`}>
      <svg className="absolute inset-0 h-full w-full" viewBox="0 0 1200 600" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
        <defs>
          <linearGradient id={id} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor={s.sky[0]} />
            <stop offset="1" stopColor={s.sky[1]} />
          </linearGradient>
        </defs>
        <rect width="1200" height="600" fill={`url(#${id})`} />
        <circle cx="930" cy="150" r="70" fill={s.sun} opacity="0.85" />
        <path d="M0 360 C 200 280 380 300 560 340 S 920 260 1200 320 V600 H0Z" fill={s.hills[0]} />
        <path d="M0 430 C 240 360 420 400 640 420 S 1000 360 1200 400 V600 H0Z" fill={s.hills[1]} />
        <path d="M0 520 C 260 460 520 500 760 500 S 1080 470 1200 490 V600 H0Z" fill={s.hills[2]} />
        {capsules && (
          <g>
            <g transform="translate(150 470) rotate(-28)">
              <rect width="120" height="44" rx="22" fill="#fff" />
              <rect width="60" height="44" rx="22" fill="#d8bb7e" />
              <rect x="38" width="22" height="44" fill="#d8bb7e" />
            </g>
            <circle cx="300" cy="520" r="18" fill="#fff" opacity="0.9" />
            <circle cx="1040" cy="470" r="12" fill="#fff" opacity="0.8" />
          </g>
        )}
      </svg>
      <div className="relative h-full">{children}</div>
    </div>
  );
}

export function Wordmark({ className = "" }) {
  return (
    <span className={`font-semibold leading-[0.8] tracking-[-0.06em] ${className}`}>
      Meditrino<sup className="ml-1 align-super text-[0.25em] font-medium tracking-normal">®</sup>
    </span>
  );
}

export function PageHead({ eyebrow, title, muted, children }) {
  return (
    <section className="px-5 pb-10 pt-14 sm:px-10 sm:pt-20">
      <Eyebrow>{eyebrow}</Eyebrow>
      <h1 className="mt-6 max-w-4xl text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">
        {title} {muted && <span className="text-muted">{muted}</span>}
      </h1>
      {children}
    </section>
  );
}
