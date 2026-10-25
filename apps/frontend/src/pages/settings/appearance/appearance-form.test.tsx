import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { AppearanceForm } from "./appearance-form";

const platform = vi.hoisted(() => ({ isMobile: false, isMacOS: false, isTauri: true }));
const connectNavigation = vi.hoisted(() => ({ visible: true, setVisible: vi.fn() }));
vi.mock("@/hooks/use-platform", () => ({ usePlatform: () => platform }));
vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: () => ({
    settings: { theme: "light", font: "font-mono", menuBarVisible: true },
    updateSettings: vi.fn(() => Promise.resolve()),
  }),
}));
vi.mock("@/pages/layouts/navigation/navigation-mode-context", () => ({
  useNavigationMode: () => ({ mode: "sidebar", setMode: vi.fn() }),
}));
vi.mock("@/pages/layouts/navigation/connect-nav-visibility", () => ({
  useConnectNavVisibility: () => [connectNavigation.visible, connectNavigation.setVisible],
}));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

beforeEach(() => {
  platform.isMobile = false;
  platform.isMacOS = false;
  platform.isTauri = true;
  connectNavigation.visible = true;
  connectNavigation.setVisible.mockReset();
});

it("offers the menu bar setting in the Windows and Linux app", () => {
  render(<AppearanceForm />);
  expect(screen.getByText("settings:appearance_menu_bar")).toBeInTheDocument();
});

it("hides the menu bar setting in the macOS app, which always keeps its menu", () => {
  platform.isMacOS = true;
  render(<AppearanceForm />);
  expect(screen.queryByText("settings:appearance_menu_bar")).not.toBeInTheDocument();
});

it("leaves the web app's menu bar setting alone on macOS", () => {
  platform.isMacOS = true;
  platform.isTauri = false;
  render(<AppearanceForm />);
  expect(screen.getByText("settings:appearance_menu_bar")).toBeInTheDocument();
});

it("offers a setting to show or hide Connect in navigation", () => {
  render(<AppearanceForm />);
  const connectSwitch = screen.getByRole("switch", {
    name: "settings:appearance_connect_navigation",
  });
  expect(connectSwitch).toBeInTheDocument();
  fireEvent.click(connectSwitch);
  expect(connectNavigation.setVisible).toHaveBeenCalledWith(false);
});
