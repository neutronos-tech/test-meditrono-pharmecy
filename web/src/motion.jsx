import { useEffect, useLayoutEffect, useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import Lenis from "lenis";

gsap.registerPlugin(ScrollTrigger);

const reduced = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const EASE = "expo.out";
let lenis = null;

/** Lenis smooth scrolling driven by GSAP's ticker so ScrollTrigger stays in sync. */
export function useSmoothScroll() {
  useEffect(() => {
    if (reduced()) return;
    lenis = new Lenis({ duration: 1.15, smoothWheel: true });
    lenis.on("scroll", ScrollTrigger.update);
    const raf = (time) => lenis.raf(time * 1000);
    gsap.ticker.add(raf);
    gsap.ticker.lagSmoothing(0);
    return () => {
      gsap.ticker.remove(raf);
      lenis.destroy();
      lenis = null;
    };
  }, []);
}

export function scrollToTop() {
  if (lenis) lenis.scrollTo(0, { immediate: true });
  else window.scrollTo(0, 0);
  requestAnimationFrame(() => ScrollTrigger.refresh());
}

function useGsap(setup, deps = []) {
  const ref = useRef(null);
  useLayoutEffect(() => {
    if (!ref.current || reduced()) return;
    const ctx = gsap.context(() => setup(ref.current), ref);
    return () => ctx.revert();
  }, deps);
  return ref;
}

/** Children rise and fade in, staggered, when the block scrolls into view. */
export function Reveal({ as: Tag = "div", className = "", children, delay = 0, y = 40 }) {
  const ref = useGsap((el) => {
    gsap.from(el.children, {
      y,
      autoAlpha: 0,
      duration: 1.2,
      ease: EASE,
      stagger: 0.09,
      delay,
      scrollTrigger: { trigger: el, start: "top 88%", once: true },
    });
  });
  return (
    <Tag ref={ref} className={className}>
      {children}
    </Tag>
  );
}

/** A headline whose words slide up from behind a mask, one after another. */
export function Words({ as: Tag = "h2", text, className = "", accent = "", delay = 0 }) {
  const ref = useGsap((el) => {
    gsap.from(el.querySelectorAll("[data-word]"), {
      yPercent: 115,
      duration: 1.3,
      ease: EASE,
      stagger: 0.06,
      delay,
      scrollTrigger: { trigger: el, start: "top 90%", once: true },
    });
  });
  const accentWords = new Set(accent.split(" ").filter(Boolean));
  return (
    <Tag ref={ref} className={className} aria-label={text}>
      {text.split(" ").map((word, index) => (
        <span key={index} aria-hidden="true" className="inline-block overflow-hidden pb-[0.08em] align-bottom">
          <span data-word className={`inline-block ${accentWords.has(word) ? "font-light text-brand" : ""}`}>
            {word}&nbsp;
          </span>
        </span>
      ))}
    </Tag>
  );
}

/** Image revealed by a rising clip, then drifting gently with the scroll. */
export function ParallaxImg({ src, alt = "", className = "", strength = 4 }) {
  const ref = useGsap((el) => {
    const img = el.querySelector("img");
    gsap.fromTo(el, { clipPath: "inset(100% 0% 0% 0%)" }, {
      clipPath: "inset(0% 0% 0% 0%)",
      duration: 1.4,
      ease: "power4.out",
      scrollTrigger: { trigger: el, start: "top 90%", once: true },
    });
    gsap.fromTo(img, { yPercent: -strength }, {
      yPercent: strength,
      ease: "none",
      scrollTrigger: { trigger: el, start: "top bottom", end: "bottom top", scrub: true },
    });
  });
  return (
    <div ref={ref} className={`overflow-hidden rounded-[1.75rem] bg-cream ${className}`}>
      <img src={src} alt={alt} loading="lazy" className="h-full w-full scale-[1.1] object-cover" />
    </div>
  );
}

/** Counts up to the number inside `value`; prefixes like ৳ stay in place. */
export function Counter({ value, className = "" }) {
  const match = String(value).match(/^(\D*)(\d+)(.*)$/);
  const ref = useGsap((el) => {
    if (!match) return;
    const [, prefix, digits, suffix] = match;
    const counter = { n: 0 };
    gsap.to(counter, {
      n: Number(digits),
      duration: 1.8,
      ease: EASE,
      scrollTrigger: { trigger: el, start: "top 90%", once: true },
      onUpdate: () => (el.textContent = `${prefix}${Math.round(counter.n)}${suffix}`),
    });
  }, [value]);
  return (
    <span ref={ref} className={`tabular-nums ${className}`}>
      {value}
    </span>
  );
}

/** Circular text that turns slowly, like a seal. */
export function Badge({ text, className = "" }) {
  const ref = useGsap((el) => {
    gsap.to(el.querySelector("svg"), { rotation: 360, duration: 22, ease: "none", repeat: -1, transformOrigin: "50% 50%" });
  });
  return (
    <div ref={ref} className={`relative ${className}`} aria-hidden="true">
      <svg viewBox="0 0 120 120" className="h-full w-full">
        <defs>
          <path id="badge-circle" d="M60,60 m-46,0 a46,46 0 1,1 92,0 a46,46 0 1,1 -92,0" />
        </defs>
        <text className="fill-current text-[10.5px] uppercase tracking-[0.32em]">
          <textPath href="#badge-circle">{text}</textPath>
        </text>
      </svg>
    </div>
  );
}

/** Scales a block up from slightly smaller as it scrolls into the middle. */
export function ScaleIn({ className = "", children }) {
  const ref = useGsap((el) => {
    gsap.fromTo(el, { scale: 0.86, autoAlpha: 0.4 }, {
      scale: 1,
      autoAlpha: 1,
      ease: "none",
      scrollTrigger: { trigger: el, start: "top 95%", end: "top 35%", scrub: true },
    });
  });
  return (
    <div ref={ref} className={className}>
      {children}
    </div>
  );
}
