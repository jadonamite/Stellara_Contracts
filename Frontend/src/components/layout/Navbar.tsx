"use client";

import { useEffect } from "react";
import { Button } from "@/components/ui/Button";
import { Container } from "@/components/ui/Container";
import { Logo } from "@/components/ui/Logo";
import { useUiStore } from "@/store/ui-store";

const navLinks = [
  { label: "Academy", href: "#" },
  { label: "AI Assistant", href: "#" },
  { label: "Community", href: "#" },
  { label: "Trade", href: "#" },
  { label: "News", href: "#" },
];

export function Navbar() {
  const mobileMenuOpen = useUiStore((state) => state.mobileMenuOpen);
  const toggleMobileMenu = useUiStore((state) => state.toggleMobileMenu);
  const closeMobileMenu = useUiStore((state) => state.closeMobileMenu);

  useEffect(() => {
    const onResize = () => {
      if (window.innerWidth >= 1024) {
        closeMobileMenu();
      }
    };

    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [closeMobileMenu]);

  return (
    <header className="pt-6 sm:pt-8">
      <Container className="relative">
        <div className="hidden items-center justify-between rounded-full bg-brand-blue px-8 py-2 lg:flex">
          <Logo />
          <nav className="flex items-center gap-4 leading-[100%] font-normal text-[1.25rem] xl:text-[1.25rem]">
            {navLinks.map((item) => (
              <a
                key={item.label}
                href={item.href}
                className="hover:text-white/80"
              >
                {item.label}
              </a>
            ))}
          </nav>
          <Button
            variant="wallet"
            className="border border-white/70 font-normal px-6 py-2"
          >
            Connect Wallet
          </Button>
        </div>

        <div className="flex items-center justify-between rounded-full bg-brand-blue px-4 py-3 lg:hidden">
          <Logo />
          <button
            type="button"
            onClick={toggleMobileMenu}
            className="relative flex h-10 w-10 items-center justify-center rounded-full border border-white/60"
            aria-label="Toggle mobile menu"
            aria-expanded={mobileMenuOpen}
          >
            <span
              className={`absolute h-[2px] w-5 bg-white transition ${
                mobileMenuOpen ? "rotate-45" : "-translate-y-1.5"
              }`}
            />
            <span
              className={`absolute h-[2px] w-5 bg-white transition ${
                mobileMenuOpen ? "-rotate-45" : "translate-y-1.5"
              }`}
            />
          </button>
        </div>

        {mobileMenuOpen && (
          <div className="mt-3 rounded-3xl border border-white/10 bg-[#0c0c0c] p-5 lg:hidden">
            <nav className="flex flex-col gap-4 text-[1.8rem]">
              {navLinks.map((item) => (
                <a
                  key={item.label}
                  href={item.href}
                  onClick={closeMobileMenu}
                  className="border-b border-white/10 pb-2"
                >
                  {item.label}
                </a>
              ))}
            </nav>
            <Button
              variant="wallet"
              className="mt-5 w-full border border-white/70 py-3"
              onClick={closeMobileMenu}
            >
              Connect Wallet
            </Button>
          </div>
        )}
      </Container>
    </header>
  );
}
