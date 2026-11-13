import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Auspice — Supply follows reality",
  description: "A NEAR testnet protocol for fair Genesis allocations and attested Signal Doors.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
