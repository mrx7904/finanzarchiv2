# Erzeugter Testkorpus

Alles in diesem Ordner ist erfunden: Absender, Arbeitgeber, Bank, Rechnungsnummern, Beträge und IBANs. Die IBANs haben nur eine gültige Prüfsumme, damit ein Leser sie als IBAN erkennt. Kein Dokument stammt aus einem echten Archiv.

| Ordner | Inhalt |
|---|---|
| `rechnungen/` | 30 Rechnungen. Jede sechste mit 7 % Umsatzsteuer, fünf als Scan ohne Textschicht (ein Seitenbild als JPEG), die letzte als E-Rechnung mit eingebetteter `factur-x.xml` |
| `gehalt/` | 12 Gehaltsabrechnungen eines Jahres, im November mit Weihnachtsgeld |
| `kontoauszuege/` | 6 Kontoauszüge eines Kontos in zwei Aufbauten: A mit Vorzeichen, B mit S und H hinter dem Betrag |

Neben jeder PDF liegt eine JSON-Datei mit den richtigen Werten, so wie sie gedruckt sind, und den Beträgen in Cent.

## Erzeugen und messen

```powershell
npm run corpus
npm run measure
```

`npm run corpus` schreibt den Korpus mit dem Startwert 2026 neu. Derselbe Startwert ergibt dieselben Dateien, ein Test prüft das. `npm run measure` misst die Leser an diesem Korpus und schreibt das Ergebnis nach `measurements/`. Ist in `.env` oder der Umgebung `CORPUS_PRIVATE` gesetzt, misst er auch die eigenen Dokumente in diesem Ordner. Deren Ergebnis landet nur in `measurements/private/`, das nicht ins Repository kommt.
