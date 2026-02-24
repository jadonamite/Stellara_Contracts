import Image from "next/image";
import { Button } from "@/components/ui/Button";
import { Container } from "@/components/ui/Container";

const featurePills = [
  { label: "AI-Powered Crypto Education", icon: "/Aipowered.png" },
  { label: "Stellar Blockchain", icon: "/stellaricon.png" },
  { label: "Community & Social", icon: "/community.png" },
  { label: "Trading & Wallet", icon: "/tradingicon.png" },
];

export function HeroSection() {
  return (
    <section className="pb-16 pt-12 sm:pb-20 sm:pt-14 lg:pb-24">
      <Container>
        <div className="grid items-center gap-12 lg:grid-cols-[1.04fr_1fr]">
          <div className="max-w-165">
            <h1 className="font-normal text-[2.5rem] leading-[100%] sm:text-[3.4rem] lg:text-[3rem]">
              <span className="text-brand-blue">Learn.</span> Trade. Connect.
              <br />
              Powered by AI on <span className="text-brand-blue">Stellar</span>.
            </h1>
            <p className="mt-7 max-w-160 text-[1.45rem] font-normal leading-tight text-white/90 sm:text-[1.55rem] lg:text-[1.75rem]">
              Stellara AI is an all-in-one Web3 academy combining AI-powered
              learning, social crypto insights, and real on-chain trading -
              built on Stellar.
            </p>
            <div className="mt-9 flex flex-wrap items-center gap-3 sm:gap-4">
              <Button variant="primary">Get Started</Button>
              <Button variant="secondary">Learn More</Button>
            </div>
          </div>

          <div className="relative mx-auto h-[500px] w-full max-w-[620px] sm:h-[600px] lg:h-[700px] lg:w-[1200px]">
            <div className="absolute inset-0 rounded-[42px] bg-[radial-gradient(circle_at_50%_45%,rgba(34,40,214,0.35),rgba(0,0,0,0.8)_58%)] blur-3xl" />
            <Image
              src="/hero-image.jpg"
              alt="Stellara AI hero visual"
              fill
              sizes="(max-width: 1023px) 100vw, 920px"
              className="object-cover"
              priority
            />
          </div>
        </div>

        <div className="mt-10 flex flex-wrap items-center justify-center gap-8 pt-8 text-[1.3rem] text-white/90 sm:text-[1.45rem]">
          {featurePills.map((feature) => (
            <div key={feature.label} className="flex items-center gap-1">
              <Image
                src={feature.icon}
                alt={`${feature.label} icon`}
                width={20}
                height={20}
                className="h-5 w-5 object-contain"
              />
              <span>{feature.label}</span>
            </div>
          ))}
        </div>
      </Container>
    </section>
  );
}
