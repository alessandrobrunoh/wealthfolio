import { usePersistentState } from "@/hooks/use-persistent-state";

export const CONNECT_NAV_VISIBLE_STORAGE_KEY = "wealthfolio:connect-navigation-visible";

export function useConnectNavVisibility() {
  return usePersistentState(CONNECT_NAV_VISIBLE_STORAGE_KEY, true);
}
