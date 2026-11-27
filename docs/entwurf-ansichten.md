# Entwurf: die Ansichten von Finanzarchiv 2.0

**Stand:** 2026-11-27 · **Entwurf, nichts davon ist entschieden.** Der Besitzer geht ihn durch, ändert und wählt. Was bleibt, wird zu einer Entscheidung in `knowledge/` und zu Issues, nach denen das Frontend zusammengeführt wird.

Der Entwurf hat zwei Teile. Die Landkarte sagt, welche Ansichten es gibt, was in welche gehört und wie man von einer zur anderen kommt. Die Regeln sagen, was in jeder Ansicht gleich ist. Bilder in echter Größe folgen danach, je Ansicht in hell und dunkel.

Grundlage sind der Look Graphit und die Bausteine aus `src/ui/` ([`decision-ui-libraries.md`](../knowledge/decision-ui-libraries.md)), die Kopfleiste als Titelleiste ([`decision-app-shell.md`](../knowledge/decision-app-shell.md)), die Anforderungen bis FA-46 ([`requirements.md`](../spec/requirements.md)) und der Satz aus der [Analyse von 1.0](analyse-finanzarchiv-1.0.md), Abschnitt 3.10: weniger und klarere Ansichten.

## Warum ein Gesamtentwurf

Abgenommen sind bisher das Aussehen und die Bausteine. Die drei gebauten Ansichten sind funktional zusammengesetzt, während die Funktionen entstehen. Dabei ist Folgendes gewachsen, das der Entwurf auflöst:

| Heute | Im Entwurf |
|---|---|
| Sechs Knöpfe in einer Reihe über "Dokumente" (Dateien, Ordner importieren, Ordner, Postfächer, Bestellungen, Namensvorlagen) | Ein Knopf "Importieren" in der Kopfleiste. Ordner, Postfächer und Namensvorlagen stehen in "Einstellungen", Bestellungen sind eine Seite unter "Dokumente" |
| "Kategorien …" als Dialog in "Konten" | Eine Seite in "Einstellungen" |
| Die Zeile "Das Archiv ist bereit." und die Version stehen mittig über jeder Ansicht | Ein gesundes Archiv sagt nichts. Die Version steht in "Einstellungen", ein Problem des Archivs füllt das ganze Fenster |
| Der Eingang ist ein Kasten über "Dokumente" | Er ist von jeder Ansicht aus sichtbar, am Fuß der Seitenleiste |
| Die Aktionen einer Ansicht stehen im Inhalt | Sie stehen in der Kopfleiste |
| Die Umsätze sind eine eigene Liste, das Detail klappt in der Zeile auf | Der Listenbaustein in einer dichten Form, das Detail steht daneben |
| "Dokumente" hat zwei Spalten, das Muster von #07 drei | Eine Regel, wann zwei und wann drei gelten |
| Die Schnellsuche gibt es nur im Muster | Sie gilt überall |

## Teil 1: die Landkarte

### Die Seitenleiste

Acht Ansichten in vier Gruppen, in jeder Ansicht gleich:

| Gruppe | Ansicht | Frage, die sie beantwortet | Stand |
|---|---|---|---|
| Heute | **Start** | Was wartet auf mich, und wie steht das Jahr? | offen (#40) |
| | **Prüfen** (mit Zahl) | Was kann nur ich entscheiden? | gebaut (#17) |
| Archiv | **Dokumente** | Was liegt im Archiv, und was steht darin? | gebaut, nicht gestaltet |
| | **Konten** | Was ist auf meinen Konten geschehen? | gebaut, nicht gestaltet (#26) |
| Verstehen | **Auswertungen** | Was hat ein Jahr gebracht und gekostet? | offen (#37, #38) |
| | **Steuern** | Was gehört in die Erklärung? | offen (#41) |
| | **Chat** | Eine Frage in Worten | offen (#42) |
| Fuß | **Eingang** (Anzeige) | Wird gerade etwas gelesen, ist etwas gescheitert? | gebaut als Kasten (#05) |
| | **Einstellungen** | Woher kommt etwas, und wie verhält sich die App? | fehlt |

Über den Gruppen steht die Schnellsuche (Strg+K, auf dem Mac ⌘K). Sie findet Ansichten, Dokumente, Parteien, Buchungen und Aktionen wie "Importieren".

Gegenüber dem Muster von #07 fallen "Rechnungen" und "Zahlungen" als eigene Ansichten weg: Eine Rechnung ist ein Dokument mit der Art Rechnung, also ein Filter in "Dokumente", und Zahlungen sind die Umsätze in "Konten".

### Was in welche Ansicht gehört

**Start.** Ein Satz, was wartet. Darunter "Als Nächstes" mit den Schritten, der dringendste zuerst, jeder mit genau einem Knopf: Dokumente in "Prüfen", Bestellungen ohne Rechnung, ein Postfach oder Ordner mit einem Problem, ein fehlender Kontoauszug, das Modell nicht erreichbar, ein Sparhinweis. Daneben drei Zahlen des laufenden Jahres (Einnahmen, Ausgaben, Saldo), jede führt in ihre Auswertung. Ist nichts zu tun, sagt Start das in einem Satz und zeigt nur die Zahlen.

**Prüfen.** Drei Ebenen, die der Pfad zeigt: Gründe, dann die Dokumente eines Grundes, dann die Frage mit der Seite daneben und der markierten Stelle. Nach einer Antwort kommt die nächste Frage von selbst. Bleibt, wie es gebaut ist, und bekommt nur die Regeln aus Teil 2.

**Dokumente.** Die Liste aller Dokumente mit Suche und Filtern (Art, Zustand, Jahr), daneben die Seite und die Angaben des gewählten Dokuments: Name, gelesene Werte mit ihrer Quelle, "Warum?" und "Rückgängig", und die Buchung, die es bezahlt hat. Unter "Dokumente" liegt die Seite **Bestellungen** (Amazon, mit "Rechnung fehlt").

**Konten.** Die Konten mit ihrem Saldo gegen die Bank. Ein Konto führt zu seinen **Umsätzen**, "Alle Umsätze" zu denen aller Konten. Neben der Liste steht das Detail der gewählten Buchung: Beleg, Kategorie, Partei über das Jahr, Umbuchung, die Stelle im Kontoauszug. Später liegt hier auch die Seite **Wiederkehrend** (#34).

**Auswertungen.** Ein Jahr, das der Besitzer wählt. Abschnitte: Einnahmen und Ausgaben nach Monat, Kategorie und Partei (#33), wiederkehrende Zahlungen (#34), Gehalt (#35), Salden über die Zeit (#36), Sparhinweise (#39). Jede Zahl öffnet die Umsätze dahinter. Wie die Abschnitte aussehen, wählt der Besitzer aus Varianten (#37), dieser Entwurf legt nur den Platz fest.

**Steuern.** Ein Steuerjahr, gegliedert nach Formular und Abschnitt. Jede Zeile führt zu ihrer Rechnung oder Buchung. Keine Erstattung, und der Hinweis, dass der Besitzer selbst prüft.

**Chat.** Links die Gespräche, rechts das Gespräch. Unter jeder Antwort stehen ihre Quellen als Dokumente und Buchungen, die sich öffnen lassen.

**Eingang.** Eine ruhige Zeile am Fuß der Seitenleiste, nur sichtbar, wenn etwas läuft, angehalten ist oder gescheitert ist ("3 werden gelesen"). Ein Klick öffnet die Aufträge mit ihrer Stufe, "Pausieren" und die abgelehnten Dateien mit ihrem Grund. So sieht man aus jeder Ansicht, dass die App arbeitet.

**Einstellungen.** Eine Ansicht mit eigenen Seiten, links ihre Liste, rechts die gewählte Seite:

| Seite | Inhalt | Kommt aus |
|---|---|---|
| Ordner | beobachtete Ordner, der Eingangsordner, die Downloads | Dialog "Ordner" in "Dokumente" |
| Postfächer | Gmail mit App-Passwort, "Jetzt lesen", "Testen" | Dialog "Postfächer" in "Dokumente" |
| Namen | die Vorlagen je Art des Dokuments | Dialog "Namensvorlagen" in "Dokumente" |
| Kategorien | die Liste der Kategorien, später die Parteien und ihre Schreibweisen | Dialog "Kategorien" in "Konten" |
| Modell | ob Ollama erreichbar ist und welches Modell liest | neu |
| Archiv | wo es liegt, die Sicherungen, später Wiederherstellen und Umziehen | neu, Stufe 6 |
| Darstellung | hell, dunkel oder wie das System | neu |
| Über | Version, die übernommenen Stücke und ihre Lizenzen | Statuszeile, Tabelle in `decision-ui-libraries.md` |

Stufe 6 (Updates, Profile, Erfolge, Stile, Feedback) bekommt hier ihre Seiten, keine neue Ansicht in der Seitenleiste.

### Wie man sich bewegt

- **Die Seitenleiste** wechselt die Ansicht. Jede Ansicht merkt sich, wo man in ihr war (Filter, gewähltes Dokument), solange die App läuft.
- **Der Pfad** in der Kopfleiste zeigt, wo man in einer Ansicht steht, und führt per Klick zurück: "Konten › Girokonto", "Prüfen › Betrag unklar › Rechnung 0413", "Dokumente › Bestellungen", "Einstellungen › Postfächer".
- **Sprünge** führen von einer Sache zu der, die zu ihr gehört. Das Ziel öffnet sich in seiner eigenen Ansicht:

| Von | Nach |
|---|---|
| Schritt auf Start | die Stelle, an der er erledigt wird |
| Zahl auf Start | ihr Abschnitt in "Auswertungen" |
| Zahl, Balken oder Zeile in "Auswertungen" | "Umsätze", gefiltert |
| Buchung | ihr Beleg in "Dokumente", ihr Kontoauszug |
| Dokument | die Buchung, die es bezahlt hat, sein Zahlungsbeleg |
| Zeile in "Steuern" | ihre Rechnung oder Buchung |
| Quelle unter einer Antwort im Chat | das Dokument oder die Buchung |
| Eingang, abgelehnte Datei | nichts, der Grund steht dabei |
| Problem eines Ordners oder Postfachs | seine Seite in "Einstellungen" |

- **Zurück nach einem Sprung:** ein Pfeil links vom Pfad und Alt+Pfeil links führen dorthin zurück, woher man kam. Der Pfad selbst zeigt nur den Ort, nie den Weg.

## Teil 2: Regeln, die in jeder Ansicht gelten

### Der Rahmen

1. Jede Ansicht steht in der `AppShell`. Keine Ansicht zeichnet eine eigene Kopfzeile, einen eigenen Titel oder eine zweite Navigation.
2. **Die Kopfleiste trägt die Aktionen der Ansicht:** höchstens ein schwarzer Hauptknopf, höchstens ein zweiter Knopf, alles Weitere in einem Menü "…". Im Inhalt stehen nur Aktionen, die zu einer Zeile oder zum gewählten Stück gehören.
3. **Was man einmal einrichtet, steht in "Einstellungen"**, nie als Knopf in einer Ansicht. Eine Ansicht darf dorthin führen, wenn etwas fehlt ("Postfach einrichten").
4. Der Inhalt beginnt oben links. Nichts steht mittig außer einem Leerzustand.
5. Kein erklärender Satz über einer Ansicht. Was eine Ansicht ist, sagen ihr Name und ihr Inhalt. Text erklärt nur einen Leerzustand, einen Fehler oder eine Frage. Das folgt dem Wort des Besitzers vom 2026-11-26: "nicht so textlastig".

### Drei Formen, mehr nicht

| Form | Aufbau | Ansichten |
|---|---|---|
| **Liste mit Detail** | Liste links in fester Breite, das gewählte Stück rechts | Dokumente, Umsätze, Bestellungen, Prüfen (Dokumente eines Grundes), Chat, Einstellungen |
| **Übersicht** | eine Spalte bis etwa 960 px Breite, Abschnitte untereinander | Start, Konten, Auswertungen, Steuern, Prüfen (Gründe) |
| **Frage** | die Seite des Dokuments links, die Frage rechts | Prüfen (Frage) |

- **Zwei oder drei Spalten:** Ist das gewählte Stück ein Dokument mit gelesenen Werten, gelten drei Spalten wie im Muster von #07 (Liste, Seite, Angaben). Sonst gelten zwei (Liste, Detail). Ist nichts gewählt, füllt die Liste die Breite nicht aus, rechts steht ein ruhiger Hinweis.
- **Schmales Fenster:** Das Fenster ist mindestens 960 px breit. Unter etwa 1200 px rücken die Angaben unter die Seite, statt eine dritte Spalte zu bilden. Nichts wird verdeckt, nichts läuft waagerecht aus dem Fenster.
- Jede Spalte rollt für sich, die Kopfleiste und die Seitenleiste stehen fest.

### Listen

6. Jede Liste ist der Baustein `List` mit `ListRow`: vorn ein Zeichen oder die erste Seite, dann Titel und eine Zeile Detail, rechts der Betrag, ganz rechts der Zustand.
7. Für Umsätze kommt eine **dichte Zeile** dazu (etwa 40 px statt 56 px), weil dort hunderte Zeilen stehen. Sie wird wie jedes neue Stück zuerst in der Gestaltungsansicht gezeigt.
8. Das Detail einer Zeile steht daneben, es klappt nie in der Liste auf.
9. **Filter stehen im Kopf der Liste:** ein Suchfeld, höchstens zwei Auswahlen, darunter eine Zeile mit der Summe ("214 Buchungen, Ausgaben 3.480,12 €"). Ein Filter, den ein Sprung gesetzt hat, steht als Marke mit "×" da, damit man sieht, warum die Liste kürzer ist.
10. Pfeiltasten bewegen die Auswahl, Enter öffnet, Esc geht eine Ebene zurück. Das gilt in jeder Liste, nicht nur in "Prüfen".

### Zustände und Beträge

11. Ein Wortschatz für Zustände, überall gleich und gleich breit:

| Kapsel | Punkt | Bedeutung |
|---|---|---|
| prüfen | gelb | wartet auf den Besitzer |
| doppelt? | rot | könnte schon im Archiv liegen |
| bestätigt / stimmt | grün | vom Besitzer oder vom Code bestätigt |
| gelesen | grau | abgelegt, ohne dass jemand hinsehen musste |
| fehlt | rot | etwas Erwartetes ist nicht da (Rechnung, Kontoauszug) |

12. Farbe steht nur für einen Zustand. Kein Baustein, kein Diagramm und keine Kategorie bekommt eine Schmuckfarbe.
13. Beträge stehen rechtsbündig in Tabellenziffern, deutsch geschrieben, mit "€". Ein Eingang trägt ein "+", ein Ausgang kein Zeichen.

### Leer, laden, Fehler

14. **Leer:** der Baustein `EmptyState` mit einem Satz und höchstens einer Aktion, die den ersten Schritt tut.
15. **Laden:** Was schneller als 200 ms kommt, zeigt nichts. Danach stehen graue Zeilen in der Form der Liste, nie ein Kreisel mitten im Fenster.
16. **Fehler:** ein deutscher Satz an der Stelle, an der der Inhalt stünde, mit "Erneut versuchen". Ein Fehler eines Feldes steht am Feld.
17. **Ein Problem des Archivs** (beschädigt, fehlt) ersetzt alle Ansichten durch eine einzige Seite mit dem Satz und den Sicherungen.

### Dialoge, Menüs, Rückmeldung

18. Ein Dialog fragt nur, bevor etwas gelöscht wird, oder nach einer einzelnen kurzen Eingabe. Listen werden nie in einem Dialog verwaltet.
19. Nach einer Aktion steht die Rückmeldung an der Stelle, an der sie geschah, mit "Rückgängig", wo es eines gibt. Keine Meldungen, die am Rand aufgehen und verschwinden.
20. Ein Knopf, der erst noch etwas fragt, endet auf " …". Ein Knopf, der sofort handelt, nicht.

### Was schon gilt und bleibt

- Jede Farbe, Größe und Bewegung kommt aus `tokens.css`, kein Baustein wählt eine eigene.
- Jede Bewegung steht unter reduzierter Bewegung des Systems still.
- Hell und dunkel sind gleichwertig, jede Ansicht wird in beiden gezeigt.
- Die Oberfläche rechnet nichts, jede Summe kommt aus dem Kern.

## Offene Fragen an den Besitzer

1. **"Konten" oder "Geld"?** Die Analyse von 1.0 nennt die Ansicht "Geld", gebaut heißt sie "Konten". Der Entwurf bleibt bei "Konten", weil die Ansicht mit den Konten beginnt.
2. **Bestellungen unter "Dokumente"?** Eine Bestellung ist kein Dokument, sie wartet auf eines. Die andere Möglichkeit ist eine Seite unter "Konten", neben den PayPal-Zahlungen.
3. **Kategorien in "Einstellungen" oder in "Konten"?** Der Entwurf legt sie in die Einstellungen, weil man sie selten ändert. Die Kategorie einer einzelnen Buchung wählt man weiter an der Buchung.
4. **Eingehendes Geld grün oder nur mit "+"?** Heute ist ein Eingang grün. Nach der Regel "Farbe nur für Zustände" wäre er schwarz mit "+". Der Entwurf schlägt das "+" ohne Farbe vor.
5. **Der Pfeil "Zurück" nach einem Sprung:** gewünscht, oder reicht der Pfad?
6. **Wiederkehrende Zahlungen:** eine Seite unter "Konten", ein Abschnitt in "Auswertungen", oder beides wie im Entwurf?
7. **Chat in der Seitenleiste** oder als Feld, das wie die Schnellsuche über jeder Ansicht aufgeht?

## Wie es weitergeht

1. Der Besitzer geht Landkarte und Regeln durch und beantwortet die Fragen.
2. Jede Ansicht entsteht als Bild in echter Größe, hell und dunkel, mit erfundenen Daten, wo es Sinn hat in Varianten.
3. Der Besitzer wählt. Das Ergebnis wird eine Entscheidung in `knowledge/`, die Anforderungen und die Issues für das Zusammenführen folgen daraus.
