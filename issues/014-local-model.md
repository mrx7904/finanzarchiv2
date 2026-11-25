# #14 The local model reads what code cannot

**Goal:** The fields code left empty are read by a local model on Ollama, chosen by measurement, without the model ever being asked how sure it is.
**Requirements**: FA-19 · **Depends on**: #11

## Done when

- [x] The current Ollama release and its API for structured output (`format` with a JSON schema), `keep_alive` and `think` are checked at the primary source with the date. Whether `ollama-rs` covers them or a thin client on `reqwest` is better is decided in `decision-stack.md`.
- [x] Ollama is reached only at `OLLAMA_HOST` (default `127.0.0.1:11434`), the row in "Outside connections" of `conventions.md` says so. A test proves no other host is called.
- [x] The model copies amounts and dates as printed and names the line of each value. Code parses them. A value whose line is not in the text is dropped with a finding.
- [x] Without Ollama, or while it does not answer within the time limit, the job waits in a stage of its own with a German reason, and goes on when Ollama answers. Tested with a fake server.
- [x] Candidates (at least `qwen3.5:9b` and `gemma4:12b-it-qat`, and the newest at ollama.com/library on the day) are measured with `npm run measure -- --reader model` on the generated corpus and on the owner's invoices: right fields, seconds per document, memory. Each measurement under 15 minutes, the model unloaded afterwards. The pick and the numbers stand in `decision-model.md`.

## Out of scope

- Reading images, belongs to #15
- A model picker in the settings, belongs to stage 6

Status 2026-11-25: `qwen3.5:9b`, `qwen3.5:9b-q8_0`, `gemma4:12b-it-qat` and `mimo-v2.6:9b` are measured, `gemma4:12b-it-qat` reads (`decision-model.md`). The newest, `mimo-v2.6:9b`, was measured after the owner allowed its download and finds less.
