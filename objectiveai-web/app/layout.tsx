import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { Space_Grotesk, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import AppShell from "@/components/AppShell";
import { AuthProvider } from "@/contexts/AuthContext";
import SonarlyTracker from "@/components/SonarlyTracker";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const spaceGrotesk = Space_Grotesk({
  variable: "--font-space-grotesk",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-jetbrains-mono",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
});

export const metadata: Metadata = {
  title: {
    default: "ObjectiveAI",
    template: "%s | ObjectiveAI",
  },
  description:
    "Your agent's advisory board. Swarms of AI agents vote with weighted probabilities to produce confidence-scored outputs. One API call, collective judgment.",
  metadataBase: new URL("https://objective-ai.io"),
  openGraph: {
    type: "website",
    siteName: "ObjectiveAI",
    title: "ObjectiveAI",
    description:
      "Your agent's advisory board. Swarms of AI agents vote with weighted probabilities to produce confidence-scored outputs. One API call, collective judgment.",
    url: "https://objective-ai.io",
  },
  twitter: {
    card: "summary",
    title: "ObjectiveAI",
    description:
      "Your agent's advisory board. Swarms of AI agents vote to produce confidence-scored outputs.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${geistSans.variable} ${geistMono.variable} ${spaceGrotesk.variable} ${jetbrainsMono.variable}`}>
        <SonarlyTracker />
        <AuthProvider>
          <AppShell>{children}</AppShell>
        </AuthProvider>
      </body>
    </html>
  );
}
