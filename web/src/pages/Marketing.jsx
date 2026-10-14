import { useEffect, useState } from "react";
import { api, taka } from "../api.js";
import { Eyebrow, Link, PHOTOS, PageHead, Photo, SITE, Tag, dark, light } from "../ui.jsx";

const FEATURE_GROUPS = [
  {
    title: "At the counter",
    items: [
      ["Fast sale screen", "Search by brand or generic name and sell in a few keystrokes."],
      ["Works offline", "Sales, stock and purchases keep working without internet."],
      ["Batch-aware selling", "Sells the batch that expires first, automatically."],
      ["Reversals", "Undo a mistaken sale or purchase, with a full record kept."],
    ],
  },
  {
    title: "Stock & purchases",
    items: [
      ["Bangladesh medicine catalogue", "Brands, generics, strengths and makers, kept up to date by MEDITRONO."],
      ["Expiry & low-stock alerts", "Know what to sell first, return, or reorder."],
      ["Supplier invoices", "Record each distributor's invoice and receive stock by batch."],
      ["Stock history", "Every movement has a time and a reason."],
    ],
  },
  {
    title: "For the owner",
    items: [
      ["Dashboard on your phone", "Sales and stock health from anywhere, installable on iPhone and Android."],
      ["Per-computer licences", "Each shop computer has its own licence; deactivate an old one any time."],
      ["Cloud copy", "Your data syncs to the cloud whenever you're online."],
      ["Audit trail", "Sensitive actions are recorded, so nothing disappears quietly."],
    ],
  },
];

const ROADMAP = [
  ["Smart intake", "Photograph a supplier challan or drop in a PDF: the stock fills itself in for you to check."],
  ["Shelves, boxes & fridges", "Tag where each medicine lives, including cold-chain items."],
  ["Baki khata", "Customer credit book, with reminders."],
  ["Prescription to cart", "AI reads the prescription and suggests the cart; the pharmacist always confirms."],
  ["Bangla interface", "The whole app in Bangla."],
  ["Online payment", "bKash, Nagad and card for your subscription."],
];

export function Features({ go }) {
  return (
    <main>
      <PageHead eyebrow="Features" title="Built for a busy counter." muted="Simple enough for the first day, deep enough for every day." />
      <div className="mb-3 grid gap-3 px-3 sm:grid-cols-3 sm:px-5">
        <Photo src={PHOTOS.counsel} alt="A pharmacist with a customer" className="h-64" />
        <Photo src={PHOTOS.shelves} alt="Shelves of medicine boxes" className="h-64" />
        <Photo src={PHOTOS.inventory} alt="Checking stock with a tablet" className="h-64" />
      </div>
      <div className="space-y-3 px-2 sm:px-3">
        {FEATURE_GROUPS.map((group, index) => (
          <section key={group.title} className={`rounded-[28px] p-6 sm:p-10 ${index === 1 ? "bg-brand text-white" : "bg-cream"}`}>
            <div className="grid gap-8 md:grid-cols-[220px_1fr]">
              <div>
                <p className={`text-xs ${index === 1 ? "text-white/70" : "text-muted"}`}>0{index + 1}</p>
                <h2 className="mt-2 text-2xl font-medium tracking-tight">{group.title}</h2>
              </div>
              <div className="grid gap-x-8 gap-y-6 sm:grid-cols-2">
                {group.items.map(([title, text]) => (
                  <div key={title} className={`border-t pt-4 ${index === 1 ? "border-white/20" : "border-black/10"}`}>
                    <p className="font-medium">{title}</p>
                    <p className={`mt-1 text-sm ${index === 1 ? "text-white/60" : "text-muted"}`}>{text}</p>
                  </div>
                ))}
              </div>
            </div>
          </section>
        ))}
      </div>
      <section className="px-5 py-20 sm:px-10">
        <Eyebrow>Coming next</Eyebrow>
        <h2 className="mt-6 text-3xl font-medium tracking-tight sm:text-5xl">On the roadmap</h2>
        <div className="mt-10 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {ROADMAP.map(([title, text]) => (
            <div key={title} className="rounded-[24px] border border-black/10 p-6">
              <Tag>Soon</Tag>
              <p className="mt-6 text-lg font-medium">{title}</p>
              <p className="mt-2 text-sm text-muted">{text}</p>
            </div>
          ))}
        </div>
        <Link to="/signup" go={go} className={`${dark} mt-10`}>Start free ↗</Link>
      </section>
    </main>
  );
}

export function Pricing({ go }) {
  const [plans, setPlans] = useState(null);
  useEffect(() => {
    api.plans().then(setPlans).catch(() => setPlans([]));
  }, []);
  return (
    <main>
      <PageHead eyebrow="Pricing" title="One simple price per computer." muted="First month free. No card needed." />
      <div className="mb-10 px-3 sm:px-5">
        <Photo src={PHOTOS.capsules} alt="Medicine tablets and capsules" className="h-56" />
      </div>
      <section className="grid gap-3 px-5 pb-10 sm:px-10 md:grid-cols-3">
        {plans === null && <p className="text-muted">Loading prices…</p>}
        {plans?.length === 0 && (
          <div className="rounded-[28px] bg-cream p-8 md:col-span-3">
            <p className="text-2xl font-medium">Prices announced soon</p>
            <p className="mt-2 text-muted">Join the free pilot now. You won't be charged without agreeing to a price first.</p>
            <Link to="/signup" go={go} className={`${dark} mt-6`}>Join the pilot ↗</Link>
          </div>
        )}
        {plans?.map((plan, index) => (
          <div key={plan.code} className={`flex flex-col rounded-[28px] p-8 ${index === 0 ? "bg-brand text-white" : "border border-black/10"}`}>
            <p className="text-sm opacity-60">{plan.name}</p>
            <p className="mt-8 text-5xl font-semibold tracking-tight">
              {plan.monthly_price_paisa > 0 ? taka(plan.monthly_price_paisa) : "Free"}
              {plan.monthly_price_paisa > 0 && <span className="text-base font-normal opacity-60"> /month</span>}
            </p>
            <p className="mt-2 text-sm opacity-60">
              {plan.yearly_price_paisa > 0 ? `or ${taka(plan.yearly_price_paisa)} a year, per computer` : "During the pilot"}
            </p>
            <ul className="mt-8 space-y-3 text-sm">
              <li>✓ {plan.trial_days} days free on your first computer</li>
              <li>✓ Works fully offline</li>
              <li>✓ Bangladesh medicine catalogue</li>
              <li>✓ Owner dashboard on your phone</li>
              <li>✓ {plan.ai_scan_cap} smart scans a month (when launched)</li>
            </ul>
            <Link to="/signup" go={go} className={`${index === 0 ? light : dark} mt-auto pt-3`}>Start free ↗</Link>
          </div>
        ))}
      </section>
      <section className="px-5 pb-20 sm:px-10">
        <div className="grid gap-6 rounded-[28px] bg-cream p-8 md:grid-cols-3">
          {[
            ["What if I'm offline when it's time to renew?", "The app warns you 7 days early, then keeps a 7-day read-only grace period. Your data is never locked away."],
            ["How do I pay?", "bKash, Nagad and card online are coming soon. During the pilot we activate your computer for you."],
            ["Can I use two computers?", "Yes. Each computer has its own licence, and you can deactivate an old one from your dashboard."],
          ].map(([q, a]) => (
            <div key={q}>
              <p className="font-medium">{q}</p>
              <p className="mt-2 text-sm text-muted">{a}</p>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}

export function About({ go }) {
  return (
    <main>
      <PageHead eyebrow="Part of MEDITRONO" title="One piece of Bangladesh's digital health infrastructure." muted="Built first for the people behind the pharmacy counter." />
      <section className="px-2 sm:px-3">
        <Photo src={PHOTOS.counsel} alt="A pharmacist helping a customer" shade className="h-[360px] sm:h-[520px]">
          <div className="flex h-full items-end p-6 text-white sm:p-10">
            <p className="max-w-xl text-xl font-medium sm:text-3xl">MEDITRONO connects patients, doctors and pharmacies. Meditrino Pharmacy is where it starts.</p>
          </div>
        </Photo>
      </section>
      <section className="grid gap-3 px-5 py-20 sm:px-10 md:grid-cols-3">
        {[
          ["The platform", "MEDITRONO is building shared digital health infrastructure for Bangladesh: identity, records, prescriptions, payments and a national medicine catalogue."],
          ["Why pharmacy first", "Pharmacies are where care meets medicine every day. Giving them reliable, offline-first tools is the fastest way to make health data useful."],
          ["What comes next", "Digital prescriptions from MEDITRONO doctors that arrive at your counter as an exact cart, and patients who can see which nearby pharmacy has their medicine, if you choose."],
        ].map(([title, text], index) => (
          <div key={title} className={`rounded-[24px] p-6 ${index === 1 ? "bg-brand text-white" : "bg-cream"}`}>
            <p className={`text-xs ${index === 1 ? "text-white/70" : "text-muted"}`}>0{index + 1}</p>
            <p className="mt-8 text-xl font-medium">{title}</p>
            <p className={`mt-3 text-sm ${index === 1 ? "text-white/60" : "text-muted"}`}>{text}</p>
          </div>
        ))}
      </section>
      <section className="px-5 pb-20 sm:px-10">
        <p className="max-w-4xl text-3xl font-medium leading-tight tracking-tight sm:text-[44px]">
          Your shop's data belongs to you. <span className="text-muted">MEDITRONO only reads your business data if you opt in, and the medicine catalogue flows one way: to you.</span>
        </p>
        <Link to="/contact" go={go} className={`${dark} mt-10`}>Talk to us ↗</Link>
      </section>
    </main>
  );
}

export function Contact({ go }) {
  return (
    <main>
      <PageHead eyebrow="Contact" title="Let's set up your pharmacy." muted="We'll help you load your stock and get started." />
      <div className="mb-3 px-3 sm:px-5">
        <Photo src={PHOTOS.pharmacist} alt="A pharmacist at the shelves" className="h-72" />
      </div>
      <section className="grid gap-3 px-5 pb-20 sm:px-10 md:grid-cols-2">
        <div className="rounded-[24px] bg-brand p-8 text-white">
          <p className="text-sm text-white/50">Fastest way</p>
          <p className="mt-6 text-3xl font-medium">Create your account and download the app.</p>
          <p className="mt-3 text-sm text-white/60">Your first computer gets a free month. During the pilot we'll contact you to help with setup.</p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link to="/signup" go={go} className={light}>Start free ↗</Link>
            <a href={SITE.downloadUrl} className={`${light} bg-transparent text-white hover:bg-white/10`}>Download for Windows</a>
          </div>
        </div>
        <div className="rounded-[28px] bg-cream p-8">
          <p className="text-sm text-muted">Reach us</p>
          {SITE.contactEmail || SITE.contactPhone ? (
            <div className="mt-6 space-y-4 text-2xl font-medium">
              {SITE.contactEmail && <a className="block underline decoration-black/20 underline-offset-8" href={`mailto:${SITE.contactEmail}`}>{SITE.contactEmail}</a>}
              {SITE.contactPhone && <a className="block underline decoration-black/20 underline-offset-8" href={`tel:${SITE.contactPhone}`}>{SITE.contactPhone}</a>}
            </div>
          ) : (
            <p className="mt-6 text-2xl font-medium">Contact details are coming soon.</p>
          )}
          <p className="mt-6 text-sm text-muted">Pilot area: Naogaon, Bangladesh.</p>
        </div>
      </section>
    </main>
  );
}
