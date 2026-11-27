import { useCallback, useEffect, useState } from "react";
import { type AccountRow, commands } from "../bindings";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { Button, EmptyState, List, ListRow } from "../ui";
import { bankLine } from "./describe";

// "Konten" of #26: every account with its balance against the bank, and what does not match.
export function Accounts({ onOpen }: { onOpen: (account: AccountRow | null) => void }) {
  const [accounts, setAccounts] = useState<AccountRow[]>([]);
  const reload = useCallback(async () => {
    const result = await commands.listAccounts();
    if (result.status === "ok") setAccounts(result.data);
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);
  useIntakeChanged(() => void reload());

  return (
    <section className="flex w-full max-w-3xl flex-col gap-3">
      <div className="flex items-center gap-2">
        <Button kind="primary" disabled={accounts.length === 0} onClick={() => onOpen(null)}>
          Alle Umsätze
        </Button>
      </div>
      {accounts.length === 0 ? (
        <EmptyState icon="card" title="Noch keine Konten" hint="Ein Konto entsteht mit seinem ersten Kontoauszug, als PDF oder CSV." />
      ) : (
        <List label="Konten">
          {accounts.map((account) => (
            <ListRow
              key={account.id}
              title={account.name}
              detail={account.findings.length > 0 ? account.findings.join(" ") : bankLine(account.bankBalance, account.bankDay)}
              figure={account.balance ?? "ohne Startsaldo"}
              state={account.findings.length > 0 ? { tone: "warn", label: "prüfen" } : { tone: "ok", label: "stimmt" }}
              onSelect={() => onOpen(account)}
            />
          ))}
        </List>
      )}
    </section>
  );
}
