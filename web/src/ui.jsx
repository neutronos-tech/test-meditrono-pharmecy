import { useEffect, useRef, useState } from "react";
import { animate, stagger } from "animejs";
import { ArrowUpRight } from "@phosphor-icons/react";

export const BRAND = "M. Pharmacy";

// Owner-editable site details. Leave a value empty to hide it.
export const SITE = {
  contactEmail: "",
  contactPhone: "",
  whatsapp: "",
  downloadUrl: "https://github.com/neutronos-tech/test-meditrono-pharmecy/releases/latest",
};

// Free-licence photos from Unsplash (unsplash.com/license).
const unsplash = (id, w = 1800) => `https://images.unsplash.com/${id}?auto=format&fit=crop&w=${w}&q=80`;
export const PHOTOS = {
  pharmacist: unsplash("photo-1580281657527-47f249e8f4df"),
  counsel: unsplash("photo-1576091358783-a212ec293ff3"),
  shelves: unsplash("photo-1642055514517-7b52288890ec"),
  inventory: unsplash("photo-1764795850513-fa8a469a4600"),
  pills: unsplash("photo-1587854692152-cbe660dbde88", 1200),
  sign: unsplash("photo-1622230208995-0f26eba75875"),
  vials: unsplash("photo-1576671081837-49000212a370", 1200),
  capsules: unsplash("photo-1471864190281-a93a3070b6de", 1200),
};

const reducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

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

/** Children fade and rise in a stagger the first time they scroll into view. */
export function Reveal({ as: Tag = "div", className = "", children, delay = 0 }) {
  const ref = useRef(null);
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const items = Array.from(element.children);
    if (reducedMotion()) return;
    // Hide only once JS runs, and always end visible even if a frame stalls.
    items.forEach((item) => (item.style.opacity = 0));
    const show = () => items.forEach((item) => { item.style.opacity = 1; item.style.transform = "none"; });
    let fallback = 0;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        observer.disconnect();
        fallback = setTimeout(show, 2500);
        animate(items, {
          opacity: [0, 1],
          translateY: [28, 0],
          delay: stagger(90, { start: delay }),
          duration: 1100,
          ease: "outExpo",
          onComplete: show,
        });
      },
      { threshold: 0.15 },
    );
    observer.observe(element);
    return () => {
      clearTimeout(fallback);
      observer.disconnect();
    };
  }, []);
  return (
    <Tag ref={ref} data-reveal className={className}>
      {children}
    </Tag>
  );
}

/** Counts up to `value` once visible. Non-numeric prefixes and suffixes stay put. */
export function Counter({ value, className = "" }) {
  const ref = useRef(null);
  const match = String(value).match(/^(\D*)(\d+)(.*)$/);
  useEffect(() => {
    const element = ref.current;
    if (!element || !match || reducedMotion()) return;
    const [, prefix, digits, suffix] = match;
    const target = { n: 0 };
    element.textContent = `${prefix}0${suffix}`;
    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      observer.disconnect();
      animate(target, {
        n: Number(digits),
        duration: 1600,
        ease: "outExpo",
        onUpdate: () => (element.textContent = `${prefix}${Math.round(target.n)}${suffix}`),
      });
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [value]);
  return (
    <span ref={ref} className={`font-mono tabular-nums ${className}`}>
      {value}
    </span>
  );
}

export function PageHead({ eyebrow, title, muted, children }) {
  return (
    <Reveal as="section" className="grid gap-8 px-4 pb-14 pt-16 sm:px-8 md:grid-cols-[1fr_2fr] lg:px-12 lg:pt-24">
      <Eyebrow>{eyebrow}</Eyebrow>
      <div>
        <h1 className="max-w-4xl text-4xl font-medium leading-none tracking-tighter md:text-6xl">
          {title} {muted && <span className="text-muted">{muted}</span>}
        </h1>
        {children}
      </div>
    </Reveal>
  );
}
