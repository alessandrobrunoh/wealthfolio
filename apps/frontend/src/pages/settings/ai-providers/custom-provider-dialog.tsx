import { useEffect, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type {
  CreateCustomProviderRequest,
  MergedProvider,
  UpdateCustomProviderRequest,
} from "@/lib/types";
import { Button } from "@wealthfolio/ui/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@wealthfolio/ui/components/ui/dialog";
import { Input } from "@wealthfolio/ui/components/ui/input";
import { Label } from "@wealthfolio/ui/components/ui/label";
import { Textarea } from "@wealthfolio/ui/components/ui/textarea";

interface CustomProviderDialogProps {
  open: boolean;
  provider?: MergedProvider;
  isSaving: boolean;
  onOpenChange: (open: boolean) => void;
  onCreate: (request: CreateCustomProviderRequest) => Promise<void>;
  onUpdate: (request: UpdateCustomProviderRequest) => Promise<void>;
}

export function CustomProviderDialog({
  open,
  provider,
  isSaving,
  onOpenChange,
  onCreate,
  onUpdate,
}: CustomProviderDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [defaultModel, setDefaultModel] = useState("");
  const [modelIds, setModelIds] = useState("");

  useEffect(() => {
    if (!open) return;
    setName(provider?.name ?? "");
    setBaseUrl(provider?.customUrl ?? "");
    setDefaultModel(provider?.defaultModel ?? "");
    setModelIds(provider?.models.map((model) => model.id).join("\n") ?? "");
  }, [open, provider]);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const request = {
      name: name.trim(),
      baseUrl: baseUrl.trim(),
      defaultModel: defaultModel.trim(),
      models: modelIds
        .split(/[\n,]/)
        .map((modelId) => modelId.trim())
        .filter(Boolean),
    };

    try {
      if (provider) {
        await onUpdate({ ...request, providerId: provider.id });
      } else {
        await onCreate(request);
      }
      onOpenChange(false);
    } catch {
      // The caller displays the error toast and leaves the dialog open.
    }
  };

  const idPrefix = provider?.id ?? "new-custom-provider";
  const isEditing = provider !== undefined;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogTitle>
          {t(
            isEditing
              ? "ai:providerSettings.customProvider.editTitle"
              : "ai:providerSettings.customProvider.createTitle",
          )}
        </DialogTitle>
        <DialogDescription>{t("ai:providerSettings.customProvider.description")}</DialogDescription>
        <form className="space-y-4" onSubmit={(event) => void handleSubmit(event)}>
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}-name`}>
              {t("ai:providerSettings.customProvider.name")}
            </Label>
            <Input
              id={`${idPrefix}-name`}
              value={name}
              onChange={(event) => setName(event.target.value)}
              maxLength={100}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}-base-url`}>
              {t("ai:providerSettings.customProvider.baseUrl")}
            </Label>
            <Input
              id={`${idPrefix}-base-url`}
              type="url"
              value={baseUrl}
              onChange={(event) => setBaseUrl(event.target.value)}
              placeholder="https://api.z.ai/api/paas/v4"
              required
            />
            <p className="text-muted-foreground text-xs">
              {t("ai:providerSettings.customProvider.baseUrlHint")}
            </p>
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}-default-model`}>
              {t("ai:providerSettings.customProvider.defaultModel")}
            </Label>
            <Input
              id={`${idPrefix}-default-model`}
              value={defaultModel}
              onChange={(event) => setDefaultModel(event.target.value)}
              maxLength={200}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}-models`}>
              {t("ai:providerSettings.customProvider.models")}
            </Label>
            <Textarea
              id={`${idPrefix}-models`}
              value={modelIds}
              onChange={(event) => setModelIds(event.target.value)}
              placeholder="glm-5.3"
              rows={4}
            />
            <p className="text-muted-foreground text-xs">
              {t("ai:providerSettings.customProvider.modelsHint")}
            </p>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              {t("common:cancel")}
            </Button>
            <Button
              type="submit"
              disabled={isSaving || !name.trim() || !baseUrl.trim() || !defaultModel.trim()}
            >
              {t(isEditing ? "common:save" : "ai:providerSettings.customProvider.create")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
