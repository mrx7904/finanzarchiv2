# Decision: the chat looks up through fixed lookups on a read-only connection, code checks its numbers

**Decided:** 2026-11-27 (#42, FA-45), taken over from 1.0 (`decision-chat.md` there) · applies to `src-tauri/src/chat.rs`, `src-tauri/src/commands/chat.rs`, the tables `chats` and `chat_messages` (migration 0026)

## Decision

- **Nine fixed lookups** that reuse the core's own reads, never SQL from the model: `einnahmen_ausgaben` (#33), `kategorie` and `partei` (#33, #43), `buchungen`, `dokumente` (name and the text of the pages), `gehalt` (#35), `regelmaessig` (#34), `steuern` (#41), `kontostand` (#36). An unknown category returns the known ones, so the model can ask again.
- **A read-only connection.** The lookups run on the archive file opened with `SQLITE_OPEN_READ_ONLY` (`chat::read_only`): SQLite refuses every write, whatever a lookup does. The archive is locked only to read the history and to keep the question and the answer, never while the model works.
- **The model picks, code decides.** Each round the model answers in a JSON schema: a lookup with its tool, year, month and words, or the answer. At most four lookups, then the schema allows only the answer. This uses the one `LocalModel::ask` of the readers, so the chat goes through the same client and its limits.
- **Code writes and checks the numbers.** Every amount of a lookup is formatted by code ("1.234,56 €"). Every amount of the answer must be one of them, compared without sign. A miscopied amount gets one round of correction, a second one replaces the answer by "Die Zahlen dazu kann ich nicht sicher wiedergeben. Die Quellen stehen unten." An amount without any lookup gets one reminder to look up, then the same sentence.
- **Nothing found is a fixed sentence.** When every lookup found nothing, the answer is "Dazu habe ich im Archiv nichts gefunden.", whatever the model wrote.
- **The sources are what the lookups returned**, the documents and bookings, never what the model names.
- **Taxes carry the notice.** An answer that used `steuern` ends with `taxes::CHECK_NOTICE` by code.
- **The history in chats.** The last six turns go with a question, so "Und 2025?" works. A chat is titled by its first question and deleted whole with its messages.
- **To the window:** `list_chats`, `chat_messages`, `ask_chat` (async, the model on a worker thread), `delete_chat`. The view "Chat" is built with the frame (#44, #45), after `decision-views.md`.

## Measured on 2026-11-27

`chat::tests` with a model that plays given steps: the read-only connection refuses a write, an answer that copies its amount with its booking as source, one correction and then the sentence without numbers, the fixed sentence when nothing was found, a number without a lookup refused, four lookups at most, the tax notice and the history in the prompt, an unknown category, chats kept and deleted whole, the check without sign.

`examples/measure.rs --reader chat --archive <copy>` asks ten questions whose right amount the core computes on the same copy (income and spending of a year and a month, three categories, the balance of all accounts, two with nothing to find), and counts the answers that name it, with the time per answer. **Not run yet:** Ollama was not running on 2026-11-27 and is not started on the side.

**Open:** that measurement with the owner's archive copy, and the view.
