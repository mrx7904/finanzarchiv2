# Finanzarchiv 2.0

Ein persönliches Finanzarchiv für den eigenen Rechner. Rechnungen, Gehaltsabrechnungen und Kontoauszüge kommen hinein, die App liest sie mit festen Lesern und einer lokalen KI, prüft jeden Wert im Code, verknüpft Belege mit Buchungen und beantwortet Fragen zur Vergangenheit. Nichts aus dem Archiv verlässt den Rechner.

Finanzarchiv 2.0 ist der Neuanfang von Finanzarchiv 1.0 (Electron) auf Basis von **Rust** und **Tauri 2**, für **Windows und macOS**, später auch Linux. Was 1.0 kann und was 2.0 daraus lernt, steht in [`docs/analyse-finanzarchiv-1.0.md`](docs/analyse-finanzarchiv-1.0.md).

## Stand

Erste Stufe „Fundament“ in Arbeit. Die App öffnet ihr Fenster unter Windows, der Mac steht noch aus. Die obere Leiste der App ist zugleich die Titelleiste des Fensters, mit dem Pfad der Seite (jede frühere Seite führt per Klick zurück), Knöpfen und unter Windows Minimieren, Maximieren und Schließen. Dateien und Ordner kommen per Dialog oder durch Ziehen ins Fenster in den „Eingang“, eine Warteschlange, die einen Absturz oder Neustart übersteht. Die Originale liegen unverändert im Archiv, beschädigte oder passwortgeschützte PDFs werden mit Grund abgelehnt. Die Gestaltung ist gewählt: „Graphit“, ruhig und ohne Farbakzent, gleich auf Windows und Mac. Im Entwicklungsbuild zeigt „Gestaltung ansehen“ alle Bausteine in Hell und Dunkel. Lesen und Prüfen folgen in Stufe 2:

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

## Selbst bauen und entwickeln

Voraussetzungen:

- Node 24 oder neuer
- Rust über [rustup](https://rustup.rs). Die passende Version aus `rust-toolchain.toml` lädt rustup beim ersten Aufruf selbst
- Windows: in Visual Studio oder den Build Tools die Arbeitsauslastung „Desktopentwicklung mit C++“. WebView2 ist in Windows 11 enthalten
- macOS: die Xcode-Befehlszeilenwerkzeuge (`xcode-select --install`)

Befehle:

| Befehl | Wirkung |
|---|---|
| `npm install` | lädt die Pakete der Oberfläche |
| `node scripts/fetch-pdfium.mjs` | lädt PDFium, die PDF-Bibliothek, für dieses System und prüft ihre Prüfsumme. Die Prüfung ruft es selbst auf |
| `npm run tauri dev` | startet die App mit Neuladen bei Änderungen |
| `npm run tauri build` | baut die App für dieses System |
| `node scripts/check.mjs` | alle Prüfungen: Links, Anforderungen und Issues, Datenschutz, Rust (fmt, clippy, Tests), TypeScript (Typen, Lint, Tests) |
| `node scripts/check.mjs --self-test` | zeigt, dass jede Prüfung bei einem eingebauten Fehler anschlägt |

## Datenschutz

Dieses Repository ist öffentlich. Es enthält keine echten Dokumente, Beträge, Namen oder Kontonummern. Alle Testdaten sind erfunden und werden erzeugt. Die Prüfung im Repository schlägt fehl, wenn etwas wie eine IBAN, eine Mailadresse oder ein persönlicher Pfad auftaucht.

## Rechte

Keine Lizenz. Der Quelltext ist öffentlich lesbar, alle Rechte bleiben beim Autor. Eine Weiterverwendung braucht seine Erlaubnis.
