import { Metadata } from "next";
import { getProfiles } from "../../lib/profiles-data";
import ProfilesBrowse from "../../components/ProfilesBrowse";

export const metadata: Metadata = {
  title: "Profiles | ObjectiveAI",
  description: "Learned weights for functions, trained to optimize ensemble voting",
};

export const revalidate = 120; // ISR: 2-minute cache

export default async function ProfilesPage() {
  const profiles = await getProfiles();
  return <ProfilesBrowse initialProfiles={profiles} />;
}
