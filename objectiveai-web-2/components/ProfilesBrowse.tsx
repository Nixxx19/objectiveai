"use client";

import { useState, useEffect, useMemo } from "react";
import type { ProfileMeta } from "@/lib/profiles/types";
import { fetchAllProfiles } from "@/lib/profiles/fetch";
import { ProfileCard } from "./ProfileCard";
import styles from "./ProfileCard.module.css";

export function ProfilesBrowse() {
  const [profiles, setProfiles] = useState<ProfileMeta[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchAllProfiles()
      .then((data) => {
        if (!cancelled) {
          setProfiles(data);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message);
          setLoading(false);
        }
      });
    return () => { cancelled = true; };
  }, []);

  // Official profiles first, then alphabetical
  const sorted = useMemo(() => {
    return [...profiles].sort((a, b) => {
      const aOfficial = a.owner === "ObjectiveAI" ? 0 : 1;
      const bOfficial = b.owner === "ObjectiveAI" ? 0 : 1;
      if (aOfficial !== bOfficial) return aOfficial - bOfficial;
      return a.name.localeCompare(b.name);
    });
  }, [profiles]);

  if (loading) {
    return (
      <div className={styles.loading} role="status" aria-live="polite">
        <span className={styles.loadingDot} />
        loading profiles
      </div>
    );
  }

  if (error) {
    return <div className={styles.error} role="alert">{error}</div>;
  }

  return (
    <div className={styles.browse}>
      <div className={styles.pageHeader}>
        <h1 className={styles.pageTitle}>profiles</h1>
        <span className={styles.pageCount}>{profiles.length}</span>
      </div>
      <div className={styles.grid}>
        {sorted.map((p) => (
          <ProfileCard key={`${p.owner}/${p.repository}`} profile={p} />
        ))}
      </div>
    </div>
  );
}
