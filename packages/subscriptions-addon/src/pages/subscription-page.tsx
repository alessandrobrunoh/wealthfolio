import type { AddonContext } from "@wealthfolio/addon-sdk";
import { useAddonTranslation } from "@wealthfolio/addon-sdk";
import { useAmountFormatting, useDateFormatting } from "@wealthfolio/ui";
import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  advanceBillingDate,
  getCalendarDay,
  isCalendarDate,
  MAX_INTERVAL_MONTHS,
} from "../lib/recurrence";

const STORAGE_KEY = "subscriptions:v1";

interface Subscription {
  id: string;
  name: string;
  amount: number;
  currency: string;
  intervalMonths: number;
  nextPaymentDate: string;
  billingDay: number;
}

interface SubscriptionForm {
  name: string;
  amount: string;
  currency: string;
  intervalMonths: string;
  nextPaymentDate: string;
}

interface SubscriptionPageProps {
  context: AddonContext;
}

class InvalidStoredSubscriptionsError extends Error {}

function localDateInputValue(date = new Date()): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(
    date.getDate(),
  ).padStart(2, "0")}`;
}

function createEmptyForm(): SubscriptionForm {
  return {
    name: "",
    amount: "",
    currency: "EUR",
    intervalMonths: "1",
    nextPaymentDate: localDateInputValue(),
  };
}

function isCurrencyCode(value: string): boolean {
  if (!/^[A-Z]{3}$/.test(value)) return false;
  try {
    new Intl.NumberFormat(undefined, { style: "currency", currency: value });
    return true;
  } catch {
    return false;
  }
}

function isSubscription(value: unknown): value is Subscription {
  if (!value || typeof value !== "object") return false;
  const subscription = value as Partial<Subscription>;
  return (
    typeof subscription.id === "string" &&
    subscription.id.length > 0 &&
    typeof subscription.name === "string" &&
    subscription.name.trim().length > 0 &&
    typeof subscription.amount === "number" &&
    Number.isFinite(subscription.amount) &&
    subscription.amount > 0 &&
    typeof subscription.currency === "string" &&
    isCurrencyCode(subscription.currency) &&
    Number.isInteger(subscription.intervalMonths) &&
    (subscription.intervalMonths ?? 0) >= 1 &&
    (subscription.intervalMonths ?? MAX_INTERVAL_MONTHS + 1) <= MAX_INTERVAL_MONTHS &&
    typeof subscription.nextPaymentDate === "string" &&
    isCalendarDate(subscription.nextPaymentDate) &&
    Number.isInteger(subscription.billingDay) &&
    (subscription.billingDay ?? 0) >= 1 &&
    (subscription.billingDay ?? 32) <= 31
  );
}

function parseStoredSubscriptions(value: string): Subscription[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    throw new InvalidStoredSubscriptionsError("Invalid subscription data");
  }
  if (!Array.isArray(parsed) || !parsed.every(isSubscription)) {
    throw new InvalidStoredSubscriptionsError("Invalid subscription data");
  }
  return parsed;
}

function createId(): string {
  return (
    globalThis.crypto?.randomUUID?.() ??
    `subscription-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`
  );
}

export function SubscriptionPage({ context }: SubscriptionPageProps) {
  const { t } = useAddonTranslation();
  const { formatAmount } = useAmountFormatting();
  const { formatCalendarDate } = useDateFormatting();
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [form, setForm] = useState<SubscriptionForm>(createEmptyForm);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [confirmRemoveId, setConfirmRemoveId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [storageError, setStorageError] = useState<"load" | "save" | "invalid" | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [loadAttempt, setLoadAttempt] = useState(0);

  useEffect(() => {
    let isCurrent = true;
    setIsLoading(true);
    setStorageError(null);

    const loadSubscriptions = async () => {
      try {
        const stored = await context.api.storage.get(STORAGE_KEY);
        const loaded = stored === null ? [] : parseStoredSubscriptions(stored);
        if (isCurrent) {
          setSubscriptions(loaded);
        }
      } catch (error) {
        if (isCurrent) {
          setStorageError(error instanceof InvalidStoredSubscriptionsError ? "invalid" : "load");
        }
      } finally {
        if (isCurrent) {
          setIsLoading(false);
        }
      }
    };

    void loadSubscriptions();
    return () => {
      isCurrent = false;
    };
  }, [context, loadAttempt]);

  const orderedSubscriptions = useMemo(
    () =>
      [...subscriptions].sort((left, right) =>
        left.nextPaymentDate.localeCompare(right.nextPaymentDate),
      ),
    [subscriptions],
  );
  const monthlyTotals = useMemo(() => {
    const totals = new Map<string, number>();
    subscriptions.forEach((subscription) => {
      totals.set(
        subscription.currency,
        (totals.get(subscription.currency) ?? 0) +
          subscription.amount / subscription.intervalMonths,
      );
    });
    return Array.from(totals.entries()).sort(([left], [right]) => left.localeCompare(right));
  }, [subscriptions]);

  const persistSubscriptions = async (nextSubscriptions: Subscription[]): Promise<boolean> => {
    setIsSaving(true);
    setStorageError(null);
    try {
      await context.api.storage.set(STORAGE_KEY, JSON.stringify(nextSubscriptions));
      setSubscriptions(nextSubscriptions);
      return true;
    } catch {
      setStorageError("save");
      return false;
    } finally {
      setIsSaving(false);
    }
  };

  const resetForm = () => {
    setForm(createEmptyForm());
    setEditingId(null);
    setFormError(null);
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFormError(null);
    const name = form.name.trim();
    const amount = Number(form.amount);
    const currency = form.currency.trim().toUpperCase();
    const intervalMonths = Number(form.intervalMonths);

    if (!name || !Number.isFinite(amount) || amount <= 0 || !isCalendarDate(form.nextPaymentDate)) {
      return;
    }
    if (!isCurrencyCode(currency)) {
      setFormError(t("invalidCurrency"));
      return;
    }
    if (
      !Number.isInteger(intervalMonths) ||
      intervalMonths < 1 ||
      intervalMonths > MAX_INTERVAL_MONTHS
    ) {
      return;
    }

    const existing = editingId
      ? subscriptions.find((subscription) => subscription.id === editingId)
      : null;
    const nextSubscription: Subscription = {
      id: editingId ?? createId(),
      name,
      amount,
      currency,
      intervalMonths,
      nextPaymentDate: form.nextPaymentDate,
      billingDay: getCalendarDay(form.nextPaymentDate),
    };
    const nextSubscriptions = existing
      ? subscriptions.map((subscription) =>
          subscription.id === existing.id ? nextSubscription : subscription,
        )
      : [...subscriptions, nextSubscription];

    if (await persistSubscriptions(nextSubscriptions)) {
      resetForm();
    }
  };

  const editSubscription = (subscription: Subscription) => {
    setEditingId(subscription.id);
    setConfirmRemoveId(null);
    setForm({
      name: subscription.name,
      amount: String(subscription.amount),
      currency: subscription.currency,
      intervalMonths: String(subscription.intervalMonths),
      nextPaymentDate: subscription.nextPaymentDate,
    });
    setFormError(null);
  };

  const markPaid = async (subscription: Subscription) => {
    const nextPaymentDate = advanceBillingDate(
      subscription.nextPaymentDate,
      subscription.intervalMonths,
      subscription.billingDay,
    );
    const nextSubscriptions = subscriptions.map((current) =>
      current.id === subscription.id ? { ...current, nextPaymentDate } : current,
    );
    await persistSubscriptions(nextSubscriptions);
  };

  const removeSubscription = async (id: string) => {
    if (confirmRemoveId !== id) {
      setConfirmRemoveId(id);
      return;
    }
    if (
      await persistSubscriptions(subscriptions.filter((subscription) => subscription.id !== id))
    ) {
      setConfirmRemoveId(null);
      if (editingId === id) resetForm();
    }
  };

  const today = localDateInputValue();
  const errorText =
    storageError === "load"
      ? t("loadError")
      : storageError === "save"
        ? t("saveError")
        : storageError === "invalid"
          ? t("invalidStoredData")
          : null;
  const hasLoadProblem = storageError === "load" || storageError === "invalid";
  const actionsDisabled = isLoading || isSaving || hasLoadProblem;

  return (
    <main className="subscriptions-addon">
      <header className="subscriptions-header">
        <div>
          <h1>{t("title")}</h1>
          <p>{t("description")}</p>
        </div>
      </header>

      <section className="subscriptions-summary" aria-label={t("monthlyEstimate")}>
        <h2>{t("monthlyEstimate")}</h2>
        {monthlyTotals.length > 0 ? (
          <div className="subscriptions-total-list">
            {monthlyTotals.map(([currency, amount]) => (
              <p key={currency}>
                <span>{currency}</span>
                <strong>{formatAmount(amount, currency)}</strong>
              </p>
            ))}
          </div>
        ) : (
          <p className="subscriptions-muted">—</p>
        )}
      </section>

      <form className="subscriptions-form" onSubmit={handleSubmit}>
        <h2>{t(editingId ? "editSubscription" : "addSubscription")}</h2>
        <div className="subscriptions-fields">
          <label>
            <span>{t("name")}</span>
            <input
              required
              maxLength={80}
              value={form.name}
              onChange={(event) => setForm({ ...form, name: event.currentTarget.value })}
            />
          </label>
          <label>
            <span>{t("amount")}</span>
            <input
              required
              type="number"
              min="0.01"
              step="any"
              value={form.amount}
              onChange={(event) => setForm({ ...form, amount: event.currentTarget.value })}
            />
          </label>
          <label>
            <span>{t("currency")}</span>
            <input
              required
              maxLength={3}
              minLength={3}
              autoCapitalize="characters"
              value={form.currency}
              onChange={(event) =>
                setForm({ ...form, currency: event.currentTarget.value.toUpperCase() })
              }
            />
          </label>
          <label>
            <span>{t("nextPayment")}</span>
            <input
              required
              type="date"
              value={form.nextPaymentDate}
              onChange={(event) => setForm({ ...form, nextPaymentDate: event.currentTarget.value })}
            />
          </label>
          <label>
            <span>{t("interval")}</span>
            <input
              required
              type="number"
              min="1"
              max={MAX_INTERVAL_MONTHS}
              step="1"
              value={form.intervalMonths}
              onChange={(event) => setForm({ ...form, intervalMonths: event.currentTarget.value })}
            />
          </label>
        </div>
        {formError && (
          <p className="subscriptions-error" role="alert">
            {formError}
          </p>
        )}
        <div className="subscriptions-form-actions">
          <button className="subscriptions-primary-button" type="submit" disabled={actionsDisabled}>
            {t("save")}
          </button>
          {editingId && (
            <button
              className="subscriptions-secondary-button"
              type="button"
              onClick={resetForm}
              disabled={actionsDisabled}
            >
              {t("cancel")}
            </button>
          )}
        </div>
      </form>

      {errorText && (
        <div className="subscriptions-error" role="alert">
          <p>{errorText}</p>
          {hasLoadProblem && (
            <button
              className="subscriptions-secondary-button"
              type="button"
              onClick={() => setLoadAttempt((attempt) => attempt + 1)}
            >
              {t("retry")}
            </button>
          )}
        </div>
      )}
      <p className="subscriptions-notice">{t("noActivityNotice")}</p>

      <section className="subscriptions-list" aria-label={t("title")}>
        {isLoading ? (
          <p className="subscriptions-muted">{t("loading")}</p>
        ) : hasLoadProblem ? null : orderedSubscriptions.length === 0 ? (
          <div className="subscriptions-empty">
            <h2>{t("emptyTitle")}</h2>
            <p>{t("emptyDescription")}</p>
          </div>
        ) : (
          orderedSubscriptions.map((subscription) => {
            const nextDate = formatCalendarDate(subscription.nextPaymentDate);
            const isOverdue = subscription.nextPaymentDate < today;
            const isConfirmingRemove = confirmRemoveId === subscription.id;

            return (
              <article className="subscription-card" key={subscription.id}>
                <div className="subscription-card-main">
                  <div>
                    <h3>{subscription.name}</h3>
                    <p className="subscriptions-muted">
                      {t(subscription.intervalMonths === 1 ? "everyMonth" : "everyMonths", {
                        count: subscription.intervalMonths,
                      })}
                    </p>
                    <p className={isOverdue ? "subscriptions-overdue" : "subscriptions-muted"}>
                      {t(isOverdue ? "overdue" : "due", { date: nextDate })}
                    </p>
                  </div>
                  <strong className="subscription-amount">
                    {formatAmount(subscription.amount, subscription.currency)}
                  </strong>
                </div>
                <div className="subscription-actions">
                  <button
                    className="subscriptions-secondary-button"
                    type="button"
                    onClick={() => void markPaid(subscription)}
                    disabled={actionsDisabled}
                  >
                    {t("markPaid")}
                  </button>
                  <button
                    className="subscriptions-secondary-button"
                    type="button"
                    onClick={() => editSubscription(subscription)}
                    disabled={actionsDisabled}
                  >
                    {t("edit")}
                  </button>
                  {isConfirmingRemove ? (
                    <>
                      <span className="subscriptions-remove-prompt">{t("confirmRemove")}</span>
                      <button
                        className="subscriptions-danger-button"
                        type="button"
                        onClick={() => void removeSubscription(subscription.id)}
                        disabled={actionsDisabled}
                        aria-label={t("confirm")}
                      >
                        {t("confirm")}
                      </button>
                      <button
                        className="subscriptions-secondary-button"
                        type="button"
                        onClick={() => setConfirmRemoveId(null)}
                      >
                        {t("cancel")}
                      </button>
                    </>
                  ) : (
                    <button
                      className="subscriptions-secondary-button"
                      type="button"
                      onClick={() => void removeSubscription(subscription.id)}
                      disabled={actionsDisabled}
                      aria-label={`${t("remove")} ${subscription.name}`}
                    >
                      {t("remove")}
                    </button>
                  )}
                </div>
              </article>
            );
          })
        )}
      </section>
    </main>
  );
}
