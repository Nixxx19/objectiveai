"use client";

import {
  createContext,
  useContext,
  useMemo,
  ReactNode,
} from "react";
import {
  SessionProvider,
  useSession,
  signIn,
  signOut as nextAuthSignOut,
} from "next-auth/react";
import { Provider } from "@/lib/provider";

export type User = {
  id: string;
  email: string;
  name: string;
  avatar?: string;
};

export type AuthError = {
  message: string;
  code?: string;
};

type AuthContextType = {
  user: User | null;
  isLoading: boolean;
  isAuthenticating: boolean;
  tokenSession: Provider.TokenSession | null;
  signInWithGoogle: () => Promise<void>;
  signInWithGitHub: () => Promise<void>;
  signInWithX: () => Promise<void>;
  signInWithReddit: () => Promise<void>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthContextType | undefined>(undefined);

function AuthContextInner({ children }: { children: ReactNode }) {
  const { data: session, status } = useSession();

  const tokenSession = useMemo<Provider.TokenSession | null>(() => {
    if (!session) return null;
    return Provider.TokenSession.fromSession(session);
  }, [session]);

  const user = useMemo<User | null>(() => {
    if (!session?.user) return null;
    return {
      id: session.user.email ?? "",
      email: session.user.email ?? "",
      name: session.user.name ?? "",
      avatar: session.user.image ?? undefined,
    };
  }, [session]);

  const signInWithGoogle = async (): Promise<void> => {
    await signIn("google");
  };

  const signInWithGitHub = async (): Promise<void> => {
    await signIn("github");
  };

  const signInWithX = async (): Promise<void> => {
    await signIn("twitter");
  };

  const signInWithReddit = async (): Promise<void> => {
    await signIn("reddit");
  };

  const handleSignOut = async (): Promise<void> => {
    await nextAuthSignOut();
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isLoading: status === "loading",
        isAuthenticating: status === "loading",
        tokenSession,
        signInWithGoogle,
        signInWithGitHub,
        signInWithX,
        signInWithReddit,
        signOut: handleSignOut,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function AuthProvider({ children }: { children: ReactNode }) {
  return (
    <SessionProvider>
      <AuthContextInner>{children}</AuthContextInner>
    </SessionProvider>
  );
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
