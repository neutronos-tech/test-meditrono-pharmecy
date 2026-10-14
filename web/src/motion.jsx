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
      rotateX: -75,
      transformOrigin: "50% 100%",
      transformPerspective: 900,
      duration: 1.4,
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

/** Circular text that turns slowly around a centred child, and spins
    faster while the page is scrolling. The text is stretched to exactly one
    lap of the circle so it never overlaps itself. */
export function Badge({ text, className = "", children }) {
  const ref = useGsap((el) => {
    const spin = gsap.to(el.querySelector("svg"), { rotation: 360, duration: 24, ease: "none", repeat: -1, transformOrigin: "50% 50%" });
    ScrollTrigger.create({
      onUpdate: (self) => {
        gsap.to(spin, { timeScale: 1 + Math.min(Math.abs(self.getVelocity()) / 300, 6), duration: 0.2, overwrite: true });
        gsap.to(spin, { timeScale: 1, duration: 1.2, delay: 0.2, ease: "power2.out" });
      },
    });
  });
  const radius = 44;
  const lap = 2 * Math.PI * radius;
  return (
    <div ref={ref} className={`relative grid place-items-center ${className}`}>
      <svg viewBox="0 0 120 120" className="absolute inset-0 h-full w-full" aria-hidden="true">
        <defs>
          <path id="badge-circle" d={`M60,60 m-${radius},0 a${radius},${radius} 0 1,1 ${radius * 2},0 a${radius},${radius} 0 1,1 -${radius * 2},0`} />
        </defs>
        <text className="fill-current text-[9px] font-medium uppercase">
          <textPath href="#badge-circle" textLength={lap - 4} lengthAdjust="spacing">{text}</textPath>
        </text>
      </svg>
      <div className="relative">{children}</div>
    </div>
  );
}

/** Pointer-tracked 3D tilt with a soft light sheen (fine pointers only). */
export function Tilt({ className = "", children, max = 7 }) {
  const ref = useGsap((el) => {
    if (!window.matchMedia("(hover: hover) and (pointer: fine)").matches) return;
    gsap.set(el, { transformPerspective: 1000, transformStyle: "preserve-3d" });
    const rx = gsap.quickTo(el, "rotationX", { duration: 0.6, ease: "power3.out" });
    const ry = gsap.quickTo(el, "rotationY", { duration: 0.6, ease: "power3.out" });
    const sheen = el.querySelector("[data-sheen]");
    const move = (event) => {
      const box = el.getBoundingClientRect();
      const x = (event.clientX - box.left) / box.width - 0.5;
      const y = (event.clientY - box.top) / box.height - 0.5;
      rx(-y * max);
      ry(x * max);
      if (sheen) gsap.to(sheen, { opacity: 1, background: `radial-gradient(circle at ${(x + 0.5) * 100}% ${(y + 0.5) * 100}%, rgba(255,255,255,0.28), transparent 55%)`, duration: 0.3 });
    };
    const leave = () => {
      rx(0);
      ry(0);
      if (sheen) gsap.to(sheen, { opacity: 0, duration: 0.5 });
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerleave", leave);
    return () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerleave", leave);
    };
  });
  return (
    <div ref={ref} className={`relative will-change-transform ${className}`}>
      {children}
      <span data-sheen className="pointer-events-none absolute inset-0 rounded-[inherit] opacity-0" />
    </div>
  );
}

/** Pulls its child gently toward the cursor (fine pointers only). */
export function Magnetic({ children, strength = 0.3, className = "" }) {
  const ref = useGsap((el) => {
    if (!window.matchMedia("(hover: hover) and (pointer: fine)").matches) return;
    const x = gsap.quickTo(el, "x", { duration: 0.5, ease: "power3.out" });
    const y = gsap.quickTo(el, "y", { duration: 0.5, ease: "power3.out" });
    const move = (event) => {
      const box = el.getBoundingClientRect();
      x((event.clientX - box.left - box.width / 2) * strength);
      y((event.clientY - box.top - box.height / 2) * strength);
    };
    const leave = () => {
      x(0);
      y(0);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerleave", leave);
    return () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerleave", leave);
    };
  });
  return (
    <span ref={ref} className={`inline-block ${className}`}>
      {children}
    </span>
  );
}

/** A block that starts tilted back in 3D and stands upright as it scrolls in. */
export function Rise3D({ className = "", children }) {
  const ref = useGsap((el) => {
    gsap.fromTo(el, { rotationX: 32, scale: 0.86, y: 60, transformPerspective: 1600, transformOrigin: "50% 100%" }, {
      rotationX: 0,
      scale: 1,
      y: 0,
      ease: "none",
      scrollTrigger: { trigger: el, start: "top 95%", end: "top 30%", scrub: 0.6 },
    });
  });
  return (
    <div ref={ref} className={className}>
      {children}
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
