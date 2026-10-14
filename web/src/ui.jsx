import { useEffect, useState } from "react";
import { Counter, Reveal, Words, scrollToTop } from "./motion.jsx";
import { ArrowUpRight } from "@phosphor-icons/react";

export const BRAND = "M. Pharmacy";

// Owner-editable site details. Leave a value empty to hide it.
export const SITE = {
  contactEmail: "",
  contactPhone: "",
  whatsapp: "",
  downloadUrl: "https://github.com/neutronos-tech/test-meditrono-pharmecy/releases/latest",
};

// Brand images from the M. Pharmacy board (web/public/brand). Replace a file
// with a full-size original of the same name to sharpen it; no code change.
const brand = (name) => `/brand/${name}.webp`;
export const IMAGES = {
  valley: brand("valley"),
  capsules: brand("capsules"),
  shelves: brand("shelves"),
  doctor: brand("doctor-tablet"),
  leaf: brand("leaf"),
  building: brand("building"),
  scanner: brand("scanner"),
  laptop: brand("laptop"),
  hands: brand("hands"),
  sprout: brand("sprout"),
  sign: brand("sign"),
};

/** The "M." mark from the brand board. */
export function Mark({ className = "h-6 w-auto" }) {
  return (
    <svg viewBox="0 0 46 28" className={className} fill="currentColor" aria-hidden="true">
      <path d="M0 28V0h9.5L18 13.5 26.5 0H36v28h-8.5V13L18 24.5 8.5 13v15z" />
      <rect x="38.5" y="20" width="6" height="8" rx="1.5" />
    </svg>
  );
}

export function Logo({ invert = false, small = false }) {
  return (
    <span className={`flex items-center gap-2.5 ${invert ? "text-white" : "text-brand-dark"}`}>
      <Mark className={small ? "h-5 w-auto" : "h-6 w-auto"} />
      <span className="leading-none">
        <span className="block text-[15px] font-semibold tracking-tight">M. Pharmacy</span>
        <span className={`mt-1 block text-[10px] tracking-[0.12em] ${invert ? "text-white/60" : "text-muted"}`}>Part of MEDITRONO</span>
      </span>
    </span>
  );
}

/** One page gutter for every section, so all left edges line up. */
export const WRAP = "px-5 sm:px-8 lg:px-14";

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
    scrollToTop();
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

const press = "active:scale-[0.98] transition duration-300 ease-[cubic-bezier(0.16,1,0.3,1)]";
export const pill = `inline-flex items-center justify-center gap-2 rounded-full px-5 py-2.5 text-sm font-medium ${press}`;
export const dark = `${pill} bg-ink text-white hover:bg-brand`;
export const light = `${pill} border border-line bg-white text-ink hover:border-brand hover:text-brand`;
export const green = `${pill} bg-brand text-white hover:bg-brand-dark`;

export function Arrow({ className = "" }) {
  return (
    <span className={`inline-flex h-7 w-7 items-center justify-center rounded-full bg-white text-ink ${className}`} aria-hidden="true">
      <ArrowUpRight size={14} weight="bold" />
    </span>
  );
}

export function Eyebrow({ children, invert }) {
  return (
    <p className={`flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] ${invert ? "text-white/70" : "text-muted"}`}>
      <span className="relative flex h-1.5 w-1.5">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-brand-light opacity-60" />
        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-brand" />
      </span>
      {children}
    </p>
  );
}

export function Tag({ children, invert }) {
  return (
    <span className={`rounded-full px-3 py-1 text-xs ${invert ? "border border-white/15 bg-white/10 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.15)] backdrop-blur-md" : "bg-cream text-ink/80"}`}>
      {children}
    </span>
  );
}

/** A rounded photo; `shade` fades it into deep green so text stays legible. */
export function Photo({ src, alt = "", className = "", shade = false, children }) {
  return (
    <div className={`group relative overflow-hidden rounded-[2rem] bg-cream ${className}`}>
      <img src={src} alt={alt} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition duration-[1200ms] ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:scale-[1.03]" />
      {shade && <div className="absolute inset-0 bg-gradient-to-t from-ink/80 via-ink/20 to-transparent" />}
      <div className="relative h-full">{children}</div>
    </div>
  );
}

export function Wordmark({ className = "" }) {
  return (
    <span className={`font-semibold leading-[0.85] tracking-[-0.06em] ${className}`}>
      M<span className="text-brand">.</span> Pharmacy
    </span>
  );
}

export { Counter, Reveal };

export function PageHead({ eyebrow, title, muted, children }) {
  return (
    <section className={`${WRAP} grid gap-8 pb-14 pt-16 md:grid-cols-12 lg:pt-24`}>
      <div className="md:col-span-3"><Eyebrow>{eyebrow}</Eyebrow></div>
      <div className="md:col-span-9">
        <Words as="h1" text={title} className="max-w-4xl text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-6xl" />
        {muted && <Reveal><p className="mt-5 max-w-[52ch] text-lg leading-relaxed text-muted">{muted}</p></Reveal>}
        {children}
      </div>
    </section>
  );
}
