import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CustomProviderDialog } from "./custom-provider-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("CustomProviderDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    document.documentElement.dir = "ltr";
  });

  it("creates a Chat Completions provider in RTL with manually entered model IDs", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    const onUpdate = vi.fn().mockResolvedValue(undefined);
    const onOpenChange = vi.fn();
    document.documentElement.dir = "rtl";
    render(
      <CustomProviderDialog
        open
        isSaving={false}
        onOpenChange={onOpenChange}
        onCreate={onCreate}
        onUpdate={onUpdate}
      />,
    );

    fireEvent.change(screen.getByLabelText("ai:providerSettings.customProvider.name"), {
      target: { value: "Z.AI" },
    });
    fireEvent.change(screen.getByLabelText("ai:providerSettings.customProvider.baseUrl"), {
      target: { value: "https://api.z.ai/api/paas/v4" },
    });
    fireEvent.change(screen.getByLabelText("ai:providerSettings.customProvider.defaultModel"), {
      target: { value: "glm-5.3" },
    });
    fireEvent.change(screen.getByLabelText("ai:providerSettings.customProvider.models"), {
      target: { value: "glm-5.3\n glm-4.6 " },
    });

    fireEvent.click(
      screen.getByRole("button", { name: "ai:providerSettings.customProvider.create" }),
    );

    await waitFor(() =>
      expect(onCreate).toHaveBeenCalledWith({
        name: "Z.AI",
        baseUrl: "https://api.z.ai/api/paas/v4",
        defaultModel: "glm-5.3",
        models: ["glm-5.3", "glm-4.6"],
      }),
    );
    expect(onUpdate).not.toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
