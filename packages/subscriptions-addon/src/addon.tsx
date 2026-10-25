import type { AddonContext, AddonEnableFunction } from "@wealthfolio/addon-sdk";
import { registerTranslations } from "@wealthfolio/addon-sdk";
import { SubscriptionPage } from "./pages/subscription-page";
import "./subscriptions.css";

let addonContext: AddonContext | undefined;

const SubscriptionsRoute = () =>
  addonContext ? <SubscriptionPage context={addonContext} /> : null;

const translations = {
  en: {
    title: "Subscriptions",
    description: "Keep track of recurring charges and when they are due.",
    monthlyEstimate: "Estimated monthly cost",
    addSubscription: "Add subscription",
    editSubscription: "Edit subscription",
    name: "Name",
    amount: "Amount",
    currency: "Currency",
    nextPayment: "Next payment",
    interval: "Billing interval",
    everyMonth: "Every month",
    everyMonths: "Every {{count}} months",
    save: "Save",
    cancel: "Cancel",
    emptyTitle: "No subscriptions yet",
    emptyDescription: "Add a subscription to see its next charge and monthly estimate.",
    due: "Due {{date}}",
    overdue: "Overdue since {{date}}",
    markPaid: "Mark paid",
    edit: "Edit",
    remove: "Remove",
    confirmRemove: "Remove this subscription?",
    confirm: "Remove subscription",
    loading: "Loading subscriptions…",
    retry: "Retry",
    loadError: "Subscriptions could not be loaded. Your saved data was left unchanged.",
    saveError: "Subscriptions could not be saved. Please try again.",
    invalidStoredData: "Saved subscription data is invalid. It was not overwritten.",
    invalidCurrency: "Enter a valid three-letter currency code, such as EUR.",
    noActivityNotice:
      "Due dates are estimates. This add-on does not create activities or payments.",
  },
  it: {
    title: "Abbonamenti",
    description: "Tieni traccia delle spese ricorrenti e delle prossime scadenze.",
    monthlyEstimate: "Costo mensile stimato",
    addSubscription: "Aggiungi abbonamento",
    editSubscription: "Modifica abbonamento",
    name: "Nome",
    amount: "Importo",
    currency: "Valuta",
    nextPayment: "Prossimo pagamento",
    interval: "Intervallo",
    everyMonth: "Ogni mese",
    everyMonths: "Ogni {{count}} mesi",
    save: "Salva",
    cancel: "Annulla",
    emptyTitle: "Nessun abbonamento",
    emptyDescription: "Aggiungi un abbonamento per vedere la prossima scadenza e la stima mensile.",
    due: "Scade il {{date}}",
    overdue: "Scaduto dal {{date}}",
    markPaid: "Segna come pagato",
    edit: "Modifica",
    remove: "Rimuovi",
    confirmRemove: "Rimuovere questo abbonamento?",
    confirm: "Rimuovi abbonamento",
    loading: "Caricamento abbonamenti…",
    retry: "Riprova",
    loadError: "Impossibile caricare gli abbonamenti. I dati salvati non sono stati modificati.",
    saveError: "Impossibile salvare gli abbonamenti. Riprova.",
    invalidStoredData:
      "I dati degli abbonamenti salvati non sono validi. Non sono stati sovrascritti.",
    invalidCurrency: "Inserisci un codice valuta valido di tre lettere, ad esempio EUR.",
    noActivityNotice: "Le scadenze sono stime. Questo add-on non crea attività o pagamenti.",
  },
};

const enable: AddonEnableFunction = (context) => {
  addonContext = context;
  registerTranslations(translations);

  context.router.add({
    id: "subscriptions",
    path: "/addons/subscriptions-addon",
    component: SubscriptionsRoute,
  });

  context.onDisable(() => {
    addonContext = undefined;
  });
};

export default enable;
