import { useLayoutEffect, useRef } from "react";
import gsap from "gsap";
import { ArrowUpRight, Barcode, CloudCheck, Package, WifiSlash } from "@phosphor-icons/react";
import { Laptop } from "../Mock.jsx";
import { Badge, Counter, Magnetic, ParallaxImg, Reveal, Rise3D, Tilt, Words } from "../motion.jsx";
import { Eyebrow, IMAGES, Link, Mark, SITE, WRAP } from "../ui.jsx";

const FACTS = [
  ["30", "days of selling with no internet"],
  ["7", "day grace, never locked out"],
  ["৳0", "setup fee, first month free"],
  ["1", "Windows computer is enough"],
];

const AREAS = [
  { image: IMAGES.scanner, label: "Counter", text: "Sell in seconds", alt: "Scanning a medicine box at the counter" },
  { image: IMAGES.shelves, label: "Stock", text: "Every batch tracked", alt: "Pharmacy shelves full of medicine" },
  { image: IMAGES.capsules, label: "Expiry", text: "Sell the oldest first", alt: "Green and white capsules" },
  { image: IMAGES.doctor, label: "Owner", text: "Your shop on your phone", alt: "A pharmacist checking a tablet" },
];

const STEPS = [
  ["Sign up", "Create your pharmacy account on this website."],
  ["Install", "Download the app on your shop computer and sign in."],
  ["Sell", "Use it all day, with or without internet."],
  ["Renew", "It renews itself whenever you are online."],
];

function ArrowButton({ dark = false }) {
  return (
    <span className={`flex h-10 w-10 items-center justify-center rounded-full transition duration-500 group-hover:rotate-45 ${dark ? "bg-brand-dark text-white" : "bg-white text-brand-dark"}`}>
      <ArrowUpRight size={16} weight="bold" />
    </span>
  );
}

function Hero({ go }) {
  const ref = useRef(null);
  useLayoutEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const cleanups = [];
    const ctx = gsap.context(() => {
      const intro = gsap.timeline({ defaults: { ease: "expo.out" } });
      intro
        .fromTo("[data-hero-img]", { scale: 1.2 }, { scale: 1.06, duration: 2.2 })
        .from("[data-hero-word]", { yPercent: 110, rotateX: -80, transformOrigin: "50% 100%", transformPerspective: 1000, duration: 1.5, stagger: 0.08 }, 0.25)
        .from("[data-hero-up]", { y: 30, autoAlpha: 0, duration: 1.2, stagger: 0.1 }, 0.7)
        .from("[data-hero-chip]", { y: 60, rotateX: 40, autoAlpha: 0, transformPerspective: 800, duration: 1.4, stagger: 0.15 }, 0.9);
      // Layered depth: each layer follows the pointer by a different amount.
      if (window.matchMedia("(hover: hover) and (pointer: fine)").matches) {
        const layers = gsap.utils.toArray("[data-depth]").map((el) => ({
          depth: Number(el.dataset.depth),
          x: gsap.quickTo(el, "x", { duration: 1, ease: "power3.out" }),
          y: gsap.quickTo(el, "y", { duration: 1, ease: "power3.out" }),
        }));
        const move = (event) => {
          const px = event.clientX / window.innerWidth - 0.5;
          const py = event.clientY / window.innerHeight - 0.5;
          layers.forEach((layer) => {
            layer.x(px * layer.depth);
            layer.y(py * layer.depth);
          });
        };
        const section = ref.current;
        section.addEventListener("pointermove", move);
        cleanups.push(() => section.removeEventListener("pointermove", move));
      }
      gsap.utils.toArray("[data-hero-chip]").forEach((chip, index) => {
        gsap.to(chip, { yPercent: index ? 10 : -10, duration: 3 + index, ease: "sine.inOut", yoyo: true, repeat: -1 });
      });
      gsap.to("[data-hero-img]", {
        yPercent: 7,
        ease: "none",
        scrollTrigger: { trigger: ref.current, start: "top top", end: "bottom top", scrub: true },
      });
    }, ref);
    return () => {
      cleanups.forEach((cleanup) => cleanup());
      ctx.revert();
    };
  }, []);
  const words = ["Calm", "care,", "every", "counter."];
  return (
    <section ref={ref} className="px-2 pt-2 sm:px-3 sm:pt-3">
      <div className="relative min-h-[calc(100dvh-124px)] overflow-hidden rounded-[2rem] bg-brand-dark text-white">
        <img data-hero-img data-depth="-18" src={IMAGES.hero} alt="Misty mountains above a pine forest" className="absolute inset-x-0 -top-[8%] h-[116%] w-full object-cover" />
        <div className="absolute inset-0 bg-brand-dark/45 mix-blend-multiply" />
        <div className="absolute inset-0 bg-gradient-to-t from-brand-dark/80 via-transparent to-brand-dark/30" />

        <div className={`${WRAP} relative flex min-h-[calc(100dvh-124px)] flex-col justify-between py-8`}>
          <div data-hero-up className="flex items-center justify-between gap-4">
            <Eyebrow invert>Pharmacy software for Bangladesh</Eyebrow>
            <span className="hidden text-xs text-white/70 sm:block">Free pilot open in Naogaon</span>
          </div>

          <h1 data-depth="14" className="relative text-[17vw] font-semibold leading-[0.88] tracking-[-0.055em] md:text-[12.5vw] xl:text-[190px]" aria-label="Calm care, every counter.">
            {words.map((word, index) => (
              <span key={word} aria-hidden="true" className="inline-block overflow-hidden pb-[0.06em] align-bottom">
                <span data-hero-word className={`inline-block ${index > 1 ? "font-light text-brand-light" : ""}`}>{word}&nbsp;</span>
              </span>
            ))}
          </h1>

          <div className="pointer-events-none absolute right-[6%] top-[22%] hidden flex-col items-end gap-4 lg:flex" data-depth="36">
            <div data-hero-chip className="rounded-2xl border border-white/20 bg-white/10 px-5 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.2),0_30px_60px_-30px_rgba(0,0,0,0.6)] backdrop-blur-xl">
              <p className="text-xs text-white/70">Today's sales</p>
              <p className="mt-1 text-3xl font-semibold tabular-nums tracking-tight">৳18,460</p>
            </div>
            <div data-hero-chip className="mr-16 flex items-center gap-3 rounded-2xl border border-white/20 bg-white/10 px-4 py-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.2),0_30px_60px_-30px_rgba(0,0,0,0.6)] backdrop-blur-xl">
              <span className="h-2 w-2 animate-pulse rounded-full bg-brand-light" />
              <p className="text-sm">Offline. Everything saved.</p>
            </div>
          </div>

          <div className="grid items-end gap-6 md:grid-cols-12">
            <div data-hero-up className="flex max-w-md items-center gap-4 rounded-[1.5rem] border border-white/15 bg-white/10 p-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.15)] backdrop-blur-xl md:col-span-6 lg:col-span-5">
              <img src={IMAGES.shelves} alt="" className="h-24 w-24 shrink-0 rounded-[1rem] object-cover" />
              <div className="pr-2">
                <p className="text-sm leading-snug text-white/85">Sales, stock and expiry on your shop computer. Works without internet.</p>
                <Magnetic className="mt-3">
                  <Link to="/signup" go={go} className="group inline-flex items-center gap-2 whitespace-nowrap rounded-full bg-white py-1.5 pl-4 pr-1.5 text-sm font-medium text-brand-dark">
                    Start your free month <ArrowButton dark />
                  </Link>
                </Magnetic>
              </div>
            </div>
            <div data-hero-up className="hidden justify-self-end md:col-span-3 md:col-start-10 md:block">
              <Badge text="Offline first · Made for Bangladesh · " className="h-32 w-32 rounded-full border border-white/20 bg-white/5 text-white/80 backdrop-blur-md">
                <Mark className="h-7 w-auto text-white" />
              </Badge>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home({ go }) {
  return (
    <main>
      {/* Hero: full-bleed photo, inerimbd style */}
      <Hero go={go} />

      {/* Facts */}
      <section className="border-y border-line">
        <div className={`${WRAP} grid grid-cols-2 md:grid-cols-4 md:divide-x md:divide-line`}>
          {FACTS.map(([value, label], index) => (
            <Reveal key={label} className="py-10 pr-4 md:px-8 md:py-12 md:first:pl-0">
              <Counter value={value} className="text-5xl font-semibold tracking-tighter text-brand-dark md:text-6xl" />
              <p className="mt-3 max-w-[22ch] text-sm text-muted">{label}</p>
            </Reveal>
          ))}
        </div>
      </section>

      {/* Areas */}
      <section className={`${WRAP} py-24`}>
        <div className="grid gap-6 md:grid-cols-12 md:items-end">
          <Words text="Built for every part of the counter." accent="the counter." className="max-w-[18ch] text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:col-span-7 md:text-6xl" />
          <Reveal className="md:col-span-4 md:col-start-9">
            <p className="text-muted">From the first sale of the morning to the expiry check at night, every job sits in one calm window.</p>
          </Reveal>
        </div>
        <Reveal className="mt-14 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {AREAS.map((area) => (
            <Tilt key={area.label} className="rounded-[1.75rem]"><Link to="/features" go={go} className="group relative block aspect-[4/5] overflow-hidden rounded-[1.75rem] bg-cream">
              <img src={area.image} alt={area.alt} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition duration-[1400ms] ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:scale-[1.06]" />
              <div className="absolute inset-0 bg-gradient-to-t from-brand-dark/75 via-brand-dark/5 to-transparent" />
              <div className="absolute inset-x-0 bottom-0 flex items-end justify-between p-5 text-white">
                <div>
                  <p className="text-xs uppercase tracking-[0.14em] text-white/70">{area.label}</p>
                  <p className="mt-1 text-xl font-medium tracking-tight">{area.text}</p>
                </div>
                <ArrowButton />
              </div>
            </Link></Tilt>
          ))}
        </Reveal>
      </section>

      {/* Statement */}
      <section className={`${WRAP} grid gap-10 md:grid-cols-12 md:items-center`}>
        <ParallaxImg src={IMAGES.hands} alt="Two hands resting together on a counter" className="aspect-[4/3] w-full max-w-[540px] md:col-span-6" strength={4} />
        <div className="md:col-span-5 md:col-start-8">
          <Eyebrow>Why M. Pharmacy</Eyebrow>
          <Words text="Calm software for a busy counter." accent="busy counter." className="mt-6 text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-[56px]" />
          <Reveal>
            <p className="mt-6 max-w-[46ch] leading-relaxed text-muted">
              No server in the shop, no technician and no setup fee. Every sale lives on your own computer first, so the counter keeps running through load-shedding and bad connections.
            </p>
            <Link to="/features" go={go} className="group mt-8 inline-flex items-center gap-3 rounded-full border border-line py-2 pl-5 pr-2 text-sm font-medium text-brand-dark transition hover:border-brand-dark">
              See every feature <ArrowButton dark />
            </Link>
          </Reveal>
        </div>
      </section>

      {/* Product */}
      <section className="mt-28 bg-cream py-24">
        <div className={WRAP}>
          <div className="grid gap-6 md:grid-cols-12 md:items-end">
            <Words text="Everything the counter needs, in one window." accent="one window." className="max-w-[20ch] text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:col-span-8 md:text-6xl" />
            <Reveal className="md:col-span-3 md:col-start-10">
              <p className="text-muted">The desktop app, as it looks on your shop computer.</p>
            </Reveal>
          </div>
          <Rise3D className="mx-auto mt-16 max-w-[1080px]">
            <Laptop />
          </Rise3D>
          <Reveal className="mt-16 grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
            {[
              [WifiSlash, "Works offline", "Sell for up to 30 days without internet."],
              [Package, "Batch and expiry", "Sells the oldest batch first and warns before a loss."],
              [Barcode, "Fast purchases", "Receive supplier invoices batch by batch."],
              [CloudCheck, "Owner dashboard", "Today's sales on your phone, anywhere."],
            ].map(([Icon, title, text]) => (
              <div key={title} className="border-t border-brand-dark/15 pt-5">
                <Icon size={26} weight="light" className="text-brand" />
                <p className="mt-4 font-medium text-brand-dark">{title}</p>
                <p className="mt-1 text-sm leading-relaxed text-muted">{text}</p>
              </div>
            ))}
          </Reveal>
        </div>
      </section>

      {/* Steps */}
      <section className={`${WRAP} grid gap-12 pt-28 md:grid-cols-12`}>
        <div className="md:col-span-4">
          <Eyebrow>How it works</Eyebrow>
          <Words text="Selling by this afternoon." accent="this afternoon." className="mt-6 text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-5xl" />
          <Reveal>
            <ParallaxImg src={IMAGES.sprout} alt="A young plant growing on stone" className="mt-10 hidden aspect-[4/3] max-w-[420px] md:block" strength={5} />
          </Reveal>
        </div>
        <Reveal className="divide-y divide-line border-y border-line md:col-span-7 md:col-start-6">
          {STEPS.map(([title, text], index) => (
            <div key={title} className="grid grid-cols-[56px_1fr] items-baseline gap-4 py-8 md:grid-cols-[80px_200px_1fr]">
              <span className="text-sm tabular-nums text-brand">0{index + 1}</span>
              <p className="text-2xl font-semibold tracking-tight text-brand-dark md:text-3xl">{title}</p>
              <p className="col-start-2 text-muted md:col-start-3">{text}</p>
            </div>
          ))}
        </Reveal>
      </section>
    </main>
  );
}
