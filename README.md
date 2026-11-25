# Finanzarchiv 2.0

Ein persönliches Finanzarchiv für den eigenen Rechner. Rechnungen, Gehaltsabrechnungen und Kontoauszüge kommen hinein, die App liest sie mit festen Lesern und einer lokalen KI, prüft jeden Wert im Code, verknüpft Belege mit Buchungen und beantwortet Fragen zur Vergangenheit. Nichts aus dem Archiv verlässt den Rechner.

Finanzarchiv 2.0 ist der Neuanfang von Finanzarchiv 1.0 (Electron) auf Basis von **Rust** und **Tauri 2**, für **Windows und macOS**, später auch Linux. Was 1.0 kann und was 2.0 daraus lernt, steht in [`docs/analyse-finanzarchiv-1.0.md`](docs/analyse-finanzarchiv-1.0.md).

## Stand

Konzeptphase. Es gibt noch keinen Programmcode. Die erste Stufe („Fundament“) ist in Issues geschnitten:

- Konzept: [`spec/concept.md`](spec/concept.md)
- Anforderungen: [`spec/requirements.md`](spec/requirements.md)
- Issues und Status: [`issues/README.md`](issues/README.md)
- Entscheidungen: [`knowledge/`](knowledge/)

## Aufbau

| Teil | Technik |
|---|---|
| Kern | Rust: Datenbank, Dokumente, Lesen, Prüfen, Zuordnen, Geld |
| Oberfläche | React mit TypeScript und Tailwind im Fenster des Systems (Tauri 2) |
| Archiv | eine SQLite-Datei und ein Ordner mit den unveränderten Originalen |
| KI | lokal über Ollama |

Geprüft wird lokal mit `node scripts/check.mjs`. GitHub Actions wird nicht verwendet, Releases werden auf dem jeweiligen Rechner gebaut.

## Datenschutz

Dieses Repository ist öffentlich. Es enthält keine echten Dokumente, Beträge, Namen oder Kontonummern. Alle Testdaten sind erfunden und werden erzeugt. Die Prüfung im Repository schlägt fehl, wenn etwas wie eine IBAN, eine Mailadresse oder ein persönlicher Pfad auftaucht.

## Rechte

Keine Lizenz. Der Quelltext ist öffentlich lesbar, alle Rechte bleiben beim Autor. Eine Weiterverwendung braucht seine Erlaubnis.
