import React from 'react';
import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: "Stellara AI",
  description: "Learn. Trade. Connect. Powered by AI on Stellar.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-[#f9fafb] text-[#111827]">
        {children}
      </body>
    </html>
  );
}
