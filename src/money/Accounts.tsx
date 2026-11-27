import { useCallback, useEffect, useState } from "react";
import { type AccountRow, commands } from "../bindings";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { BarActions, Button, EmptyState, List, ListRow } from "../ui";
import { accountState, bankLine } from "./describe";

// "Konten" as designed (#48, knowledge/decision-views.md): the sum over all accounts as the one large figure, how
// many agree with their bank, then every account with its balance, the bank's balance with its day and "stimmt",
// "prüfen" or "ohne Bank". The top bar holds "Kontoauszug importieren" and "Alle Umsätze".
export function Accounts({ onOpen }: { onOpen: (account: AccountRow | null) => void }) {
  const [accounts, setAccounts] = useState<AccountRow[] | null>(null);
  const [total, setTotal] = useState<{ text: string; day: string } | null>(null);
  const [queued, setQueued] = useState<string | null>(null);
  const reload = useCallback(async () => {
    const [listed, balances] = await Promise.all([commands.listAccounts(), commands.balancesByMonth()]);
    if (listed.status === "ok") setAccounts(listed.data);
    if (balances.status === "ok") {
      const last = balances.data.total.at(-1);
      setTotal(last ? { text: last.text, day: last.day } : null);
    }
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);
  useIntakeChanged(() => void reload());

  const agreeing = accounts?.filter((account) => accountState(account).label === "stimmt").length ?? 0;
  return (
    <>
      <BarActions>
        <Button
          onClick={() =>
            void commands.importFiles().then((result) => setQueued(result.status === "ok" ? `${result.data.files} im Eingang` : result.error.message))
          }
        >
          Kontoauszug importieren …
        </Button>
        <Button kind="primary" disabled={!accounts || accounts.length === 0} onClick={() => onOpen(null)}>
          Alle Umsätze
        </Button>
      </BarActions>
      <div className="mx-auto flex w-full max-w-[1120px] flex-col gap-6 overflow-y-auto px-6 pt-6 pb-8">
        {accounts !== null && accounts.length === 0 ? (
          <EmptyState icon="card" title="Noch keine Konten" hint="Ein Konto entsteht mit seinem ersten Kontoauszug." />
        ) : (
          <>
            <section aria-label="Alle Konten" className="flex flex-col gap-1">
              <span className="text-small text-fg-muted">Alle Konten{total ? ` · ${total.day.split("-").reverse().join(".")}` : ""}</span>
              <span className="text-[34px] leading-tight font-semibold tracking-tight tabular-nums">{total?.text ?? "…"}</span>
              <span className="text-small text-fg-muted">
                {accounts ? `${agreeing} von ${accounts.length} ${accounts.length === 1 ? "Konto stimmt" : "Konten stimmen"} mit der Bank` : ""}
                {queued ? ` · ${queued}` : ""}
              </span>
            </section>
            <section aria-label="Konten" className="rounded-panel bg-surface shadow-panel">
              <List label="Konten">
                {(accounts ?? []).map((account) => (
                  <ListRow
                    key={account.id}
                    title={account.name}
                    detail={account.findings.length > 0 ? account.findings.join(" ") : bankLine(account.bankBalance, account.bankDay)}
                    figure={account.balance ?? "ohne Startsaldo"}
                    state={accountState(account)}
                    onSelect={() => onOpen(account)}
                  />
                ))}
              </List>
            </section>
          </>
        )}
      </div>
    </>
  );
}
