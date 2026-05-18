import type { ReactElement } from "react";
import cn from "classnames";

export interface Tab {
  id: string;
  label: string;
}

interface TabBarProps {
  tabs: Tab[];
  activeTab: string;
  onSelect: (id: string) => void;
}

export function TabBar({ tabs, activeTab, onSelect }: TabBarProps): ReactElement {
  return (
    <nav
      role="tablist"
      className={cn(
        "flex",
        "gap-1",
        "px-3",
        "pt-2",
        "bg-neutral-100",
        "border-b",
        "border-neutral-300",
        "dark:bg-neutral-900",
        "dark:border-neutral-700",
      )}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTab;
        return (
          <button
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            onClick={() => onSelect(tab.id)}
            className={cn(
              "px-4",
              "py-2",
              "rounded-t-md",
              "border-b-2",
              "cursor-pointer",
              "text-sm",
              "transition-colors",
              "duration-150",
              isActive
                ? cn(
                    "bg-white",
                    "text-neutral-900",
                    "border-blue-600",
                    "font-semibold",
                    "dark:bg-neutral-800",
                    "dark:text-neutral-50",
                    "dark:border-blue-400",
                  )
                : cn(
                    "bg-transparent",
                    "text-neutral-600",
                    "border-transparent",
                    "font-medium",
                    "hover:bg-neutral-200",
                    "hover:text-neutral-900",
                    "dark:text-neutral-400",
                    "dark:hover:bg-neutral-800",
                    "dark:hover:text-neutral-50",
                  ),
            )}
          >
            {tab.label}
          </button>
        );
      })}
    </nav>
  );
}
