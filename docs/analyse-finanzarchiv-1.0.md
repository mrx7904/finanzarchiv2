# Analyse von Finanzarchiv 1.0

**Stand:** 25.11.2026, gelesen an Version 0.37.0 (`mrx7904/finanzarchiv`, Commit `0eb6552`). Grundlage sind Konzept, Anforderungen, die 120 Issues, die 31 Entscheidungsdateien, der Quellcode und die Git-Historie.

Diese Analyse beantwortet drei Fragen: Was kann 1.0? Wie setzt sie es um? Was machen wir in 2.0 von Anfang an anders?

## Zahlen vorweg

| Größe | Wert |
|---|---|
| Entwicklungszeit | 29.10. bis 18.11.2026, rund drei Wochen |
| Commits | 540 (162 `feat`, 75 `fix`) |
| Anforderungen | 184 (`FA-01` bis `FA-208`) |
| Issues | 120 in 18 Stufen |
| Quellcode | rund 49.000 Zeilen TypeScript in `src/` |
| Tests | 183 Dateien, rund 20.000 Zeilen |
| Datenbank | 51 Tabellen aus 47 Migrationen |
| IPC-Kanäle zwischen Fenster und Hauptprozess | 252 |

Fast jede Stufe steht auf „gebaut, Abnahme durch den Besitzer offen“. Gebaut wurde deutlich schneller, als im echten Alltag geprüft werden konnte. Das ist die wichtigste Lehre für 2.0 und kommt unten noch einmal.

## 1. Funktionsumfang

### Eingang: Wie Dokumente hereinkommen

- Dateien und Ordner ins Fenster ziehen, über „Importieren“ wählen, ohne vorher die Art anzugeben.
- Der Ordner „Finanzarchiv Eingang“ wird ohne Klick übernommen, weitere Ordner können beobachtet werden, auf Wunsch auch der Download-Ordner (nur PDFs, die wie Rechnung oder Gehaltsabrechnung aussehen).
- Postfächer: Gmail über IMAP mit App-Passwort und Microsoft 365 mit Anmeldung im Browser. Streng nur lesend, im Hintergrund alle 15 Minuten, sichere Rechnungsmails werden ohne Klick übernommen. Enthält die Mail nur einen Link, lädt die App die PDF dahinter, sonst wird die Mail selbst als PDF abgelegt.
- PayPal- und Amazon-Pay-Mails werden ohne KI gelesen und als Zahlungsbeleg oder als eigene Ausgabe verbucht.
- Eine gemeinsame Anzeige „Eingang“ für alle Wege, mit Pausieren und Rückgängig.
- Gleiche Dateien erkennt die App am Hash, doppelte Rechnungen an Nummer, Aussteller und Betrag.

### Lesen: Wie aus einer PDF Daten werden

- Text aus der Textschicht der PDF, ein lokales Modell über Ollama schlägt die Felder vor (anfangs `qwen3.5:4b`, gemessen auch 9b und 27b).
- Zu jedem Wert die Textzeile, aus der er stammt. Beträge und Daten übernimmt das Modell wörtlich, umgerechnet wird im Code.
- Lesung ohne KI für Rechnungsnummer, Daten und Gesamtbetrag, Absenderprofile aus früheren Rechnungen, Regeln aus den Korrekturen des Besitzers.
- Zweite Lesung als Bild (Vision-Modell), wenn ein entscheidender Betrag nicht im Text steht. Beide Lesarten werden verglichen.
- Eigene Leser ohne KI für Kontoauszüge der VR Bank und der ING (PDF), ING-CSV, beliebige CSV mit Spaltenwahl und die Amazon-Bestellhistorie.
- Benennung nach einer Vorlage je Dokumentart.

### Prüfen: Was der Besitzer noch sehen muss

- Seit 0.12 prüft Code jeden vorgeschlagenen Wert gegen das Dokument und das Archiv. Was alle Prüfungen besteht, wird automatisch übernommen, alles andere wartet mit Begründung.
- Die Warteschlange ist nach Grund gruppiert, mit Ja-Nein-Fragen, zwei Lesarten nebeneinander, Verlauf je Wert, „Zur Kenntnis“ für Hinweise und Rückgängig für jede automatische Entscheidung.
- Eine spätere Buchung kann einen Betrag nachträglich bestätigen.

### Archiv: Was man wiederfindet

- Dokumente als Aktenschrank mit Volltextsuche, Filtern und Vorschau.
- Rechnungen nach Monaten mit Fälligkeit, bezahlt und offen, gespeicherte Ansichten.
- Gehalt je Jahr mit Brutto, Abzügen, Netto und erkannten Lücken.
- Zahlungsbelege, die an ihre Rechnung gehängt werden und nie doppelt zählen.
- Schnellsuche mit ⌘K über alles, Tastenkürzel für fast jede Aktion.

### Geld: Konten, Zahlungen, Kontoauszüge

- Konten mit Startsaldo, Zahlungen mit Art, Methode und Typ, Daueraufträge.
- Kontoauszüge buchen, Kontostand Tag für Tag mit der Bank abgleichen, Umbuchungen zwischen eigenen Konten erkennen.
- Rechnungen und Gehälter ihrer Buchung zuordnen, nach Regeln mit Zeitfenstern und Namen. Listen „Ohne Beleg“ und „Beleg ohne Buchung“.
- Kategorien je Gegenseite, Geldfluss je Monat, die Brücke „vom Kontostand zum Kontostand“.

### Auswertungen und Steuern

- Einkommen je Monat mit Vorjahr, Ausgaben nach Kategorie, größte Empfänger, Fixkosten, Kontostände.
- Steuern: Zuordnung zu Stellen der Einkommensteuererklärung, Summen je Jahr mit Pauschalen, Feld in ELSTER, § 35a, Fristen, Bescheide, Export der Liste und der Belege. Ausdrücklich keine Steuerberatung.
- Als PDF sichern und drucken.

### Chat

- Fragen auf Deutsch in mehreren Chats. Das Modell nutzt Werkzeuge über eine nur lesende Datenbankverbindung, nennt seine Quellen, Beträge prüft der Code. Ein Chat kann sich auf ein einzelnes Dokument beziehen.

### Rahmen

- Profile für mehrere Personen mit optionalem Passwort.
- Tägliche Sicherungspunkte der Datenbank, Wiederherstellen, Startprüfung bei beschädigtem Archiv.
- Umzug per Exportdatei mit Vorschau, Archiv leeren.
- Update-Hinweis und Update aus der App, Modellverwaltung und Wahl des Modells passend zum Rechner, Einrichtung von Ollama.
- Protokoll ohne Werte aus dem Archiv, nur eine Instanz pro Datenordner.
- Startansicht mit nächsten Schritten, 17 Erfolge und Stufen, Stile als Pakete (etwa „Zelda“), „Feedback und Ideen“, Bewegung aus sechs UI-Bibliotheken.

## 2. Technische Umsetzung

### Aufbau

| Teil | Umsetzung in 1.0 |
|---|---|
| Hülle | Electron 45 mit electron-vite, drei Prozesse: Hauptprozess, Preload, Fenster |
| Oberfläche | React 19, Tailwind 4, TypeScript 7 strikt, eigene Bausteine in `ui.tsx` und `fx/` |
| Grenze Fenster zu Hauptprozess | 252 IPC-Kanäle, einmal in `src/shared/ipc.ts` deklariert, im Preload ausgeschrieben, jede Antwort mit zod geprüft |
| Datenbank | SQLite über `node:sqlite` in einer Datei, WAL, eine schreibende und eine nur lesende Verbindung für den Chat. Eine Hülle bildet noch die Schnittstelle des ursprünglichen MariaDB-Pools nach |
| Dokumente | Dateien unverändert unter ihrem SHA-256-Hash im Archivordner, Metadaten in der Datenbank |
| Beträge | Ganze Cent als `INTEGER`, nie Gleitkomma |
| KI | Offizieller Ollama-Client, strukturierte Ausgabe mit JSON-Schema aus zod, Modell lokal |
| PDF | `unpdf` (pdf.js) für den Text, `@napi-rs/canvas` als einziges natives Modul für Seitenbilder |
| Mail | `imapflow` mit einer Hülle, die jeden schreibenden IMAP-Befehl verweigert, Zugangsdaten über Electron `safeStorage` im Schlüsselbund |
| Sicherheit | `contextIsolation`, Sandbox, CSP, `hardening.ts` verbietet neue Fenster und fremde Seiten |
| Qualität | vitest, fast-check (Eigenschaftstests), Stryker (Mutationstests), oxlint mit Grenzen für Funktionslänge und Verschachtelung |
| Auslieferung | electron-builder (NSIS, dmg ad hoc signiert), anfangs GitHub Actions, seit 0.11 lokal gebaut und installiert |

### Was gut funktioniert hat und bleibt

- **Das Archiv bleibt auf dem Rechner.** Lokales Modell, Liste aller erlaubten Verbindungen, Protokoll ohne Werte.
- **Die Quelle bleibt.** Originaldatei unverändert unter ihrem Hash, jeder Wert zeigt auf seine Textzeile.
- **Code prüft das Modell.** Das Modell wird nie nach seiner Sicherheit gefragt. Übereinstimmung unabhängiger Leser und die Zeile im Text entscheiden.
- **Ganze Cent** überall.
- **Nur lesend** beim Postfach und beim Chat, durch Konstruktion statt durch Absicht.
- **Entscheidungsdateien** für alles, was nicht im Code steht.

## 3. Was wir mit dem Wissen von 1.0 besser machen

### 3.1 Ein Geldmodell von Anfang an

1.0 hat das Geld in vier Schritten gelernt: erst Zahlungen von Hand, dann „Rechnungen und Gehälter zählen als Zahlung“, dann Kontoauszüge, zuletzt „im Zeitraum der Auszüge zählen nur die Buchungen“ (#114, #119). Dazu kamen Sonderfälle wie „vorgemerkt“, „bar bezahlt“ und „PayPal bucht vom Girokonto“. Der Abschnitt „Zahlungen“ im README ist dadurch ein einziger, kaum lesbarer Absatz geworden.

**2.0:** Die **Buchung** ist die einzige Quelle für Geld. Jede Bewegung ist eine Buchung auf einem Konto, auch Bargeld (Konto „Bar“) und eine Zahlung ohne Auszug. Dokumente sind **Belege**, die mit Buchungen verknüpft werden, auch mehrere zu einer oder eine zu mehreren (Teilzahlungen). Eine offene Rechnung ist eine offene Verbindlichkeit, kein Geldfluss. Damit gibt es keine Regel mehr, welche Quelle in welchem Zeitraum zählt.

### 3.2 Herkunft je Wert als Datenmodell

In 1.0 kamen Belegzeile, zweiter Leser, Vision, Verlauf, automatische Übernahme, Korrekturen, Absenderregeln und Kategorieregeln nacheinander dazu, jedes mit eigener Tabelle (`proposals`, `auto_acceptances`, `acceptance_corrections`, `sender_corrections`, `sender_rules`, `category_rules` und weitere).

**2.0:** Jeder Wert eines Dokuments ist von Anfang an ein Eintrag mit Herkunft: welcher Leser (Code, Parser, Modell, Bild, Besitzer), welche Stelle im Dokument, welche Prüfungen er bestanden hat. Verlauf, Rückgängig und „Warum?“ ergeben sich daraus von selbst.

### 3.3 Feste Leser zuerst, KI als Rückfall

1.0 hat mit der KI begonnen und die festen Leser (Kontoauszüge, PayPal, Absenderprofile) später ergänzt. Die festen Leser sind genauer und schneller.

**2.0:** Eine Kette von Lesern mit gemeinsamer Schnittstelle: E-Rechnung (ZUGFeRD und XRechnung tragen ihre Daten maschinenlesbar), bekannte Bankformate, bekannte Absender, erst danach das lokale Modell und zuletzt das Bild. Jeder Leser wird einzeln gemessen.

### 3.4 Ein Eingang von Tag 1

1.0 hatte zuerst getrennte Wege (Ziehen, Ordner, Mail) und hat sie erst in #88 zu einem Eingang zusammengeführt.

**2.0:** Eine dauerhafte Warteschlange in der Datenbank mit festen Schritten: ablegen, Text holen, einordnen, lesen, prüfen, ablegen oder warten. Jeder Weg hinein erzeugt nur einen Auftrag. Die Warteschlange übersteht Neustarts, lässt sich pausieren und arbeitet mehrere Dokumente parallel ab. Weil 2.0 alle Dokumente neu einliest, muss sie mit Hunderten Dateien auf einmal umgehen können.

### 3.5 Messen von Anfang an

Bei der Planung von 0.12 war keine der 97 Rechnungen bestätigt, eine Referenz zum Messen fehlte also. Messungen auf dem echten Archiv liefen von Hand und landeten teils nur im Gedächtnis des Agenten.

**2.0:** Ein erzeugter Satz von Rechnungen, Gehaltsabrechnungen und Kontoauszügen mit bekannten richtigen Werten liegt im Repository (das Repo ist öffentlich, also nur erfundene Daten). Ein privater Satz echter Dokumente mit Sollwerten liegt außerhalb des Repos. Ein Befehl misst beide und schreibt das Ergebnis in eine Datei. Jede Änderung an einem Leser muss die Zahlen halten oder verbessern.

### 3.6 Regeln an einem Ort und erklärbar

Die Zuordnungsregeln von 1.0 stecken mit ihren Zeitfenstern (5 Tage vor bis 90 Tage nach Rechnungsdatum, 10 bis 50 Tage nach Monatsbeginn, 3 Tage für Umbuchungen und weitere) verteilt im Code und als Fließtext im README.

**2.0:** Jede Regel hat eine Kennung, ihre Parameter stehen an einer Stelle, und jede automatische Entscheidung speichert, welche Regel mit welchen Werten sie getroffen hat. Die Oberfläche zeigt bei jedem automatischen Schritt „Warum?“ und „Rückgängig“.

### 3.7 Die Grenze zwischen Oberfläche und Kern wird erzeugt

252 Kanäle wurden in 1.0 von Hand dreimal beschrieben: Deklaration, Preload und zod-Schema.

**2.0:** Die Befehle werden einmal in Rust geschrieben, die TypeScript-Typen für die Oberfläche erzeugt `tauri-specta`. Ein Test schlägt fehl, wenn die erzeugten Typen nicht mehr zum Rust-Code passen.

### 3.8 Ein Schema, das gleich passt

47 Migrationen in drei Wochen, Tabellen wie `documents_new` und `intake_log_new` zeigen Umbauten. Die Hülle um `node:sqlite` bildet noch den MariaDB-Pool nach.

**2.0:** Ein Schema, das das Wissen aus 1.0 schon enthält (siehe `knowledge/decision-data-model.md`), mit `STRICT`-Tabellen, Fremdschlüsseln und eingebetteten Migrationen. SQLite kommt über `rusqlite` mit eingebautem SQLite, Sicherungen über die Backup-Schnittstelle, Integritätsprüfung vor jedem Start.

### 3.9 Kleinere Schritte, Abnahme vor der nächsten Stufe

18 Stufen wurden gebaut, abgenommen ist kaum eine. Funktionen kamen schneller, als sie im Alltag erprobt wurden, und mehrere Überarbeitungen der Oberfläche (#33, #34, #78, #107, „zu bunt“) haben schon Gebautes erneut verändert.

**2.0:** Höchstens eine Stufe ist offen. Die nächste beginnt erst, wenn der Besitzer die vorige im Alltag abgenommen hat. Ab Stufe 2 nutzt der Besitzer 2.0 mit echten Dokumenten.

### 3.10 Erst das Gestaltungssystem, dann die Ansichten

1.0 hat das Aussehen nachträglich mehrfach umgebaut. `ui.tsx` hat 818 Zeilen, einzelne Ansichten 400 bis 600.

**2.0:** Zuerst Farben, Abstände, Schrift und die Grundbausteine aus den sechs Bibliotheken, in Varianten in echter Größe gezeigt und vom Besitzer gewählt. Erst dann entstehen Ansichten, und zwar weniger und klarere: Start mit Eingang, Prüfen, Dokumente, Geld, Auswertungen, Steuern, Chat, Einstellungen.

### 3.11 Plattform: Rust und Tauri

| Thema | 1.0 (Electron) | 2.0 (Tauri 2) |
|---|---|---|
| Größe | eigener Chromium, Installer über 100 MB | Fenster des Systems (WebView2 unter Windows, WebKit unter macOS), Installer rund 10 MB (zu prüfen) |
| Sicherheit | `hardening.ts` von Hand | Fähigkeiten (Capabilities) je Fenster, alles ist verboten, was nicht erlaubt ist |
| Rechenarbeit | ein Node-Hauptprozess | Rust mit echten Threads, PDF-Arbeit parallel |
| Zugangsdaten | Electron `safeStorage` | `keyring` (Windows-Anmeldeinformationen, macOS-Schlüsselbund) |
| Seitenbilder | natives Modul `@napi-rs/canvas` | PDFium über `pdfium-render`, eine Bibliothek je System wird mitgeliefert (zu prüfen) |
| Linux | mitgebaut, aber nie gestartet | später, Tauri unterstützt es |

Der Mac fragt in 1.0 nach jedem Update erneut nach dem Schlüsselbund, weil die App nur ad hoc signiert ist. Das wird 2.0 ebenso treffen, solange keine Signatur mit Apple-Entwicklerkonto vorliegt. Das ist eine offene Entscheidung für dich.

### 3.12 KI-Laufzeit

Ollama bleibt der Weg zum lokalen Modell. Es läuft auf Windows und macOS, nutzt die Grafikkarte und tauscht Modelle ohne neue App-Version. In 2.0 steht es hinter einer Schnittstelle, sodass später ein eingebettetes llama.cpp möglich bleibt. Texterkennung für Scans (in 1.0 zurückgestellt, #103) wird über die Erkennung des Betriebssystems oder Tesseract geprüft.

### 3.13 Ohne GitHub Actions

Actions laufen bei dir derzeit nicht, und schon in 1.0 haben macOS-Minuten das Kontingent aufgebraucht. 2.0 plant das von Anfang an ohne: Ein Prüfbefehl läuft lokal und ist das Tor für jede Sitzung. Releases werden je System auf dem eigenen Rechner gebaut, das macOS-Paket auf dem Mac, und per Skript als GitHub-Release hochgeladen. Es gibt keine Workflow-Dateien im Repository.

### 3.14 Öffentliches Repository

- Keine echten Dokumente, Beträge, Namen, IBANs oder Mailadressen im Repository, auch nicht in Tests. Testdaten werden erzeugt.
- Der private Messsatz und die Einstellungen des Rechners liegen außerhalb und sind ausgeschlossen.
- Commits laufen unter der GitHub-Noreply-Adresse statt unter der Mailadresse des Rechners.
- Keine Lizenz. Der Code ist lesbar, alle Rechte bleiben beim Besitzer.
- Aus Aceternity UI und 21st.dev wird weiterhin nur die Idee übernommen. In einem öffentlichen Repository wäre deren Quelltext eine Weitergabe, die die Lizenz verbietet.

### 3.15 Was 2.0 zunächst weglässt

- **Postfach sortieren** (Mails in Ordner verschieben). Das Lesen von Rechnungen aus dem Postfach bleibt.
- **Import aus 1.0.** Die Originaldokumente und Kontoauszüge werden in 2.0 neu eingelesen. Das ist zugleich der beste Test der Automatik.

Erfolge, Stile, Profile, Chat und Steuern bleiben im Konzept und kommen in späteren Stufen, nachdem das Fundament abgenommen ist.
