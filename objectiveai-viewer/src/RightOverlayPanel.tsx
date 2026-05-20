import { useEffect, useRef, useState, type Dispatch, type SetStateAction } from "react";
import cn from "classnames";
import type {
  AgentCompletionsMessageRichContentPart,
  FilesystemConfigFavorite,
} from "@objectiveai/sdk";
import { useFavoriteAgents } from "./useFavoriteAgents";
import { ChatPane } from "./ChatPane";

export interface PanelTab {
  id: string;
  favorite: FilesystemConfigFavorite;
  draft: string;
  attachments: AgentCompletionsMessageRichContentPart[];
}

interface RightOverlayPanelProps {
  panelTabs: PanelTab[];
  setPanelTabs: Dispatch<SetStateAction<PanelTab[]>>;
  activePanelTabId: string | null;
  setActivePanelTabId: Dispatch<SetStateAction<string | null>>;
}

export function RightOverlayPanel({
  panelTabs,
  setPanelTabs,
  activePanelTabId,
  setActivePanelTabId,
}: RightOverlayPanelProps) {
  const { favorites, loading, error } = useFavoriteAgents();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const plusRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!dropdownOpen) return;
    const onDocDown = (e: MouseEvent) => {
      if (plusRef.current && !plusRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocDown);
    return () => document.removeEventListener("mousedown", onDocDown);
  }, [dropdownOpen]);

  const openFavorite = (favorite: FilesystemConfigFavorite) => {
    const id = crypto.randomUUID();
    const tab: PanelTab = { id, favorite, draft: "", attachments: [] };
    setPanelTabs((prev) => [...prev, tab]);
    setActivePanelTabId(id);
    setDropdownOpen(false);
  };

  const closeTab = (id: string) => {
    setPanelTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (activePanelTabId === id) {
        setActivePanelTabId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
  };

  const updateTab = (id: string, patch: Partial<PanelTab>) => {
    setPanelTabs((prev) =>
      prev.map((t) => (t.id === id ? { ...t, ...patch } : t)),
    );
  };

  const activeTab = panelTabs.find((t) => t.id === activePanelTabId) ?? null;

  return (
    <div
      className={cn(
        "absolute",
        "top-0",
        "right-0",
        "bottom-0",
        "w-96",
        "z-10",
        "bg-white",
        "dark:bg-neutral-950",
        "border-l",
        "border-neutral-300",
        "dark:border-neutral-700",
        "shadow-lg",
        "flex",
        "flex-col",
      )}
      role="complementary"
    >
      <nav
        role="tablist"
        className={cn(
          "flex",
          "flex-row",
          "items-stretch",
          "gap-1",
          "px-2",
          "pt-2",
          "bg-neutral-100",
          "dark:bg-neutral-900",
          "border-b",
          "border-neutral-300",
          "dark:border-neutral-700",
          "overflow-x-auto",
        )}
      >
        {panelTabs.map((tab) => {
          const isActive = tab.id === activePanelTabId;
          return (
            <div
              key={tab.id}
              className={cn(
                "flex",
                "flex-row",
                "items-center",
                "rounded-t-md",
                "border-b-2",
                isActive
                  ? cn(
                      "bg-white",
                      "border-blue-600",
                      "dark:bg-neutral-800",
                      "dark:border-blue-400",
                    )
                  : cn(
                      "bg-transparent",
                      "border-transparent",
                      "hover:bg-neutral-200",
                      "dark:hover:bg-neutral-800",
                    ),
              )}
            >
              <button
                type="button"
                role="tab"
                aria-selected={isActive}
                onClick={() => setActivePanelTabId(tab.id)}
                className={cn(
                  "px-3",
                  "py-2",
                  "text-sm",
                  "cursor-pointer",
                  "max-w-40",
                  "truncate",
                  isActive
                    ? cn(
                        "text-neutral-900",
                        "dark:text-neutral-50",
                        "font-semibold",
                      )
                    : cn("text-neutral-600", "dark:text-neutral-400"),
                )}
              >
                {tab.favorite.name}
              </button>
              <button
                type="button"
                onClick={() => closeTab(tab.id)}
                aria-label={`Close ${tab.favorite.name}`}
                className={cn(
                  "pr-2",
                  "py-1",
                  "text-neutral-500",
                  "dark:text-neutral-400",
                  "hover:text-neutral-900",
                  "dark:hover:text-neutral-50",
                  "cursor-pointer",
                  "text-sm",
                )}
              >
                {"×"}
              </button>
            </div>
          );
        })}

        <div ref={plusRef} className={cn("relative", "flex", "items-stretch")}>
          <button
            type="button"
            onClick={() => {
              if (loading) return;
              setDropdownOpen((v) => !v);
            }}
            aria-label="Open favorite"
            className={cn(
              "px-3",
              "py-2",
              "rounded-t-md",
              "border-b-2",
              "border-transparent",
              "text-sm",
              "text-neutral-600",
              "dark:text-neutral-400",
              "hover:bg-neutral-200",
              "dark:hover:bg-neutral-800",
              loading ? "cursor-wait" : "cursor-pointer",
            )}
          >
            {"+"}
          </button>

          {dropdownOpen && !loading && (
            <FavoritesDropdown
              favorites={favorites}
              error={error}
              onPick={openFavorite}
            />
          )}
        </div>
      </nav>

      <div className={cn("flex", "flex-col", "flex-1", "min-h-0")}>
        {activeTab ? (
          <ActiveTabBody
            tab={activeTab}
            onDraftChange={(draft) => updateTab(activeTab.id, { draft })}
            onAttachmentsChange={(attachments) =>
              updateTab(activeTab.id, { attachments })
            }
            onSend={() =>
              updateTab(activeTab.id, { draft: "", attachments: [] })
            }
          />
        ) : (
          <div
            className={cn(
              "flex",
              "flex-1",
              "items-center",
              "justify-center",
              "px-4",
              "text-center",
              "text-sm",
              "text-neutral-500",
              "dark:text-neutral-400",
            )}
          >
            Open a favorite from + to start a conversation.
          </div>
        )}
      </div>
    </div>
  );
}

interface FavoritesDropdownProps {
  favorites: FilesystemConfigFavorite[];
  error: string | null;
  onPick: (favorite: FilesystemConfigFavorite) => void;
}

function FavoritesDropdown({ favorites, error, onPick }: FavoritesDropdownProps) {
  return (
    <div
      role="menu"
      className={cn(
        "absolute",
        "top-full",
        "left-0",
        "mt-1",
        "min-w-56",
        "max-h-80",
        "overflow-y-auto",
        "rounded-md",
        "border",
        "border-neutral-300",
        "dark:border-neutral-700",
        "bg-white",
        "dark:bg-neutral-900",
        "shadow-lg",
        "z-20",
        "py-1",
      )}
    >
      {error && (
        <div
          className={cn(
            "px-3",
            "py-2",
            "text-xs",
            "text-red-600",
            "dark:text-red-400",
          )}
        >
          {error}
        </div>
      )}
      {favorites.length === 0 ? (
        <div
          className={cn(
            "px-3",
            "py-2",
            "text-sm",
            "text-neutral-500",
            "dark:text-neutral-400",
          )}
        >
          (no favorites yet)
        </div>
      ) : (
        favorites.map((fav) => (
          <button
            key={fav.name}
            type="button"
            role="menuitem"
            onClick={() => onPick(fav)}
            className={cn(
              "block",
              "w-full",
              "text-left",
              "px-3",
              "py-2",
              "cursor-pointer",
              "hover:bg-neutral-100",
              "dark:hover:bg-neutral-800",
            )}
          >
            <div
              className={cn(
                "text-sm",
                "text-neutral-900",
                "dark:text-neutral-50",
              )}
            >
              {fav.name}
            </div>
            {fav.note && (
              <div
                className={cn(
                  "text-xs",
                  "text-neutral-500",
                  "dark:text-neutral-400",
                  "truncate",
                )}
              >
                {fav.note}
              </div>
            )}
          </button>
        ))
      )}
    </div>
  );
}

interface ActiveTabBodyProps {
  tab: PanelTab;
  onDraftChange: (value: string) => void;
  onAttachmentsChange: (
    next: AgentCompletionsMessageRichContentPart[],
  ) => void;
  onSend: () => void;
}

function ActiveTabBody({
  tab,
  onDraftChange,
  onAttachmentsChange,
  onSend,
}: ActiveTabBodyProps) {
  return (
    <div className={cn("flex", "flex-col", "flex-1", "min-h-0")}>
      <div
        className={cn(
          "flex-1",
          "min-h-0",
          "overflow-y-auto",
          "p-4",
          "text-sm",
          "text-neutral-500",
          "dark:text-neutral-400",
        )}
      >
        <div className={cn("text-xs", "uppercase", "tracking-wide")}>
          Agent
        </div>
        <div
          className={cn(
            "mt-1",
            "text-neutral-900",
            "dark:text-neutral-50",
            "font-semibold",
          )}
        >
          {tab.favorite.name}
        </div>
        {tab.favorite.note && (
          <div className={cn("mt-1", "text-sm")}>{tab.favorite.note}</div>
        )}
        <div
          className={cn(
            "mt-6",
            "text-xs",
            "italic",
            "text-neutral-400",
            "dark:text-neutral-500",
          )}
        >
          (Chat history will render here.)
        </div>
      </div>

      <ChatPane
        value={tab.draft}
        onChange={onDraftChange}
        attachments={tab.attachments}
        onAttachmentsChange={onAttachmentsChange}
        onSend={onSend}
      />
    </div>
  );
}
