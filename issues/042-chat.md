# #42 The chat over the archive

**Goal:** The owner asks the archive in words and gets answers whose numbers are right.
**Requirements**: FA-45 · **Depends on**: #33, #34, #35, #41, #44, #45

The view follows the map and the rules of [`knowledge/decision-views.md`](../knowledge/decision-views.md), its picture is in `src/entwurf/` (#50).

## Done when

- [x] A view "Chat". The local model reaches data only through fixed tools that reuse the core's own reads (income and spending, categories, parties, bookings, documents, salary, recurring payments, taxes), never SQL from the model.
- [x] The tools run on a connection that opens the archive read-only.
- [x] Code formats every amount in a tool result, checks every amount of the answer against them, and the sources under an answer are the documents and bookings the tools returned. A question that finds nothing gets a fixed sentence.
- [x] The history in chats the owner can delete.
- [ ] Measured on a set of questions with the owner's archive copy: answers right, numbers right, time per answer.

**Status:** the core is done on 2026-11-27 (`knowledge/decision-chat.md`): lookups, read-only connection, the checks and the chats. The view is built on 2026-11-27 as a list with detail: the chats at the left, the chosen one at the right with the question field at its foot, "Neuer Chat" and "Chat löschen …" in the top bar, "Sieht in deinem Archiv nach …" while the model works, and the sources under an answer, each opening its document or the bookings. Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings. Open: the measurement with the real model, which waits for Ollama to run, and the owner's look on the real archive.
