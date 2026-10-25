import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { CONNECT_NAV_VISIBLE_STORAGE_KEY, useConnectNavVisibility } from "./connect-nav-visibility";

describe("useConnectNavVisibility", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("shows Connect by default and persists a hidden preference", () => {
    const { result } = renderHook(() => useConnectNavVisibility());

    expect(result.current[0]).toBe(true);
    act(() => result.current[1](false));

    expect(result.current[0]).toBe(false);
    expect(localStorage.getItem(CONNECT_NAV_VISIBLE_STORAGE_KEY)).toBe("false");
  });

  it("restores the saved visibility choice", () => {
    localStorage.setItem(CONNECT_NAV_VISIBLE_STORAGE_KEY, "false");

    const { result } = renderHook(() => useConnectNavVisibility());

    expect(result.current[0]).toBe(false);
  });
});
