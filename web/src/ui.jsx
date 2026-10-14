import { useEffect, useState } from "react";

// Owner-editable site details. Leave a value empty to hide it.
export const SITE = {
  contactEmail: "",
  contactPhone: "",
  whatsapp: "",
  downloadUrl: "https://github.com/neutronos-tech/test-meditrono-pharmecy/releases/latest",
};

// Free-licence photos from Unsplash (unsplash.com/license).
const unsplash = (id, w = 1600) => `https://images.unsplash.com/${id}?auto=format&fit=crop&w=${w}&q=80`;
export const PHOTOS = {
  pharmacist: unsplash("photo-1580281657527-47f249e8f4df"),
  counsel: unsplash("photo-1576091358783-a212ec293ff3"),
  shelves: unsplash("photo-1642055514517-7b52288890ec"),
  inventory: unsplash("photo-1764795850513-fa8a469a4600"),
  pills: unsplash("photo-1587854692152-cbe660dbde88", 1200),
  sign: unsplash("photo-1622230208995-0f26eba75875", 1200),
  vials: unsplash("photo-1576671081837-49000212a370", 1200),
  capsules: unsplash("photo-1471864190281-a93a3070b6de", 1200),
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
export const dark = `${pill} bg-ink text-white hover:bg-brand`;
export const light = `${pill} border border-line bg-white text-ink hover:border-ink`;
export const green = `${pill} bg-brand text-white hover:bg-brand-dark`;

export function Arrow({ className = "" }) {
  return (
    <span className={`inline-flex h-7 w-7 items-center justify-center rounded-full bg-white text-ink ${className}`} aria-hidden="true">
      ↗
    </span>
  );
}

export function Eyebrow({ children, invert }) {
  return (
    <p className={`flex items-center gap-2 text-xs font-medium ${invert ? "text-white/70" : "text-muted"}`}>
      <span className="h-1.5 w-1.5 rounded-full bg-brand" /> {children}
    </p>
  );
}

export function Tag({ children, invert }) {
  return (
    <span className={`rounded-full px-3 py-1 text-xs ${invert ? "bg-white/15 text-white backdrop-blur" : "bg-cream text-ink/80"}`}>
      {children}
    </span>
  );
}

/** A rounded photo with an optional overlay; children sit on top. */
export function Photo({ src, alt = "", className = "", shade = false, children }) {
  return (
    <div className={`relative overflow-hidden rounded-[24px] bg-cream ${className}`}>
      <img src={src} alt={alt} loading="lazy" className="absolute inset-0 h-full w-full object-cover" />
      {shade && <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-black/10 to-transparent" />}
      <div className="relative h-full">{children}</div>
    </div>
  );
}

export function Wordmark({ className = "" }) {
  return (
    <span className={`font-semibold leading-[0.85] tracking-[-0.055em] ${className}`}>
      Meditrino<sup className="ml-1 align-super text-[0.22em] font-medium tracking-normal">®</sup>
    </span>
  );
}

export function PageHead({ eyebrow, title, muted, children }) {
  return (
    <section className="px-5 pb-12 pt-16 sm:px-10 sm:pt-24">
      <Eyebrow>{eyebrow}</Eyebrow>
      <h1 className="mt-6 max-w-4xl text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">
        {title} {muted && <span className="text-muted">{muted}</span>}
      </h1>
      {children}
    </section>
  );
}
