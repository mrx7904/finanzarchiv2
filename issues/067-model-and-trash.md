# #67 The model, its setup and the Papierkorb in Einstellungen

**Goal:** The owner chooses the model, is led through a missing Ollama, and empties the Papierkorb, all in Einstellungen.
**Requirements**: FA-67 · **Depends on**: #49

As of 0.7.0 the page "Modell" only shows a name, an address and a state. `model.name` and `intake.workers` are settings of the core without a command. Without Ollama the page says "nicht erreichbar" and nothing more, while documents wait for the model (FA-19). The dialogs of deleting say the file stays in the Papierkorb "bis du ihn leerst", and nothing empties it.

## Done when

- [ ] The page "Modell" lists the models installed in Ollama and lets the owner choose one. The measured default is marked (`knowledge/decision-model.md`). A change counts for the documents read from then on and is told in one line.
- [ ] Ollama out of reach, or the chosen model not installed, shows what to do in at most three steps with the command to copy, and "Erneut prüfen". The app installs nothing by itself. To verify: the endpoints of Ollama that list and describe models, at the documentation of Ollama on the day of the work.
- [ ] The page says how many documents wait for the model, and they go on by themselves once it answers.
- [ ] The number of workers of the intake can be set from 1 to 4, with the measured default named (`knowledge/decision-intake.md`), and counts after a restart or at once, which is decided and recorded here.
- [ ] The page "Archiv" shows the Papierkorb with the count and the size of its files, "Im Ordner zeigen" and "Papierkorb leeren …". The dialog names the count and that it cannot be undone. A document deleted by mistake can be taken in again from the folder until then.
- [ ] No new connection to the outside: the client still talks to `OLLAMA_HOST` only (`knowledge/conventions.md`, "Outside connections").
- [ ] Tests: the list of models against a fake server, a model that is missing, the setting kept over a restart, the Papierkorb emptied and counted.

## Out of scope

- Restoring a snapshot, moving the archive, updates: stage 6.
- Downloading a model from inside the app.
