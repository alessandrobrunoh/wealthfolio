# Subscriptions add-on

Manage recurring subscriptions and estimate their monthly cost inside
Wealthfolio. Each subscription stores a name, amount, currency, next billing
date, and interval from 1 to 12 months. Marking a charge as paid advances its
next date without creating a Wealthfolio activity or moving money.

## Build and install

From the repository root:

```sh
pnpm --filter subscriptions-addon build
pnpm --filter subscriptions-addon package
```

In Wealthfolio, open **Settings → Add-ons → Installed → Install from file** and
select the ZIP created in `dist/`.

Subscription data is saved through the add-on's private durable storage. No
account or transaction access, network access, or additional permission is
required. Storage follows Wealthfolio's existing paired-device sync behavior.
