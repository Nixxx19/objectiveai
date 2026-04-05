"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";
import styles from "./Shell.module.css";

export function Shell({ children }: { children: ReactNode }) {
  const pathname = usePathname();

  return (
    <>
      <header className={styles.header}>
        <Link href="/" className={styles.logo}>
          <span className={styles.logoMark} />
          objectiveai
        </Link>
        <nav className={styles.nav}>
          <Link
            href="/"
            className={`${styles.navLink} ${
              pathname === "/" || pathname.startsWith("/functions") ? styles.navLinkActive : ""
            }`}
          >
            functions
          </Link>
          <Link
            href="/swarms"
            className={`${styles.navLink} ${
              pathname.startsWith("/swarms") ? styles.navLinkActive : ""
            }`}
          >
            swarms
          </Link>
        </nav>
      </header>
      <main className={styles.main}>{children}</main>
    </>
  );
}
