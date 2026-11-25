# Decision: which local model reads

**Decided:** 2026-11-25, by measurement (#14) · applies to `readers::model::DEFAULT_MODEL`, the setting `model.name`

## Decision

- The model that reads the fields code left empty is `gemma4:12b-it-qat` on Ollama (Q4_0, 11.9 B parameters). The owner can choose another with the setting `model.name`. A picker in the settings comes with stage 6.
- Each measurement asks the model for every field of a document, not only the missing ones, so models compare on the same work. On the owner's documents there is no truth yet, so a field counts as found when code took the model's value (it stands in the line the model named and parses), and the model agrees with code where both found the field.

## Measured on 2026-11-25

RTX 5080 with 16 GB, Ollama 0.40.16, `npm run measure -- --reader model --model <name>`, each run well under the 15 minutes of the owner's GPU budget, the model unloaded afterwards (`keep_alive` 0, checked with `/api/ps`).

| Model | Corpus, 37 documents | s per document | Owner, 18 invoices: gross found | date found | findings (19 documents) | GPU |
|---|---|---|---|---|---|---|
| `qwen3.5:9b` (Q4_K_M) | every field right | 2.8 | 8 | 11 | 11 | 7.7 GB |
| `qwen3.5:9b-q8_0` | every field right | 4.6 | 16 | 16 | 11 | 11.1 GB |
| `gemma4:12b-it-qat` | every field right | 4.2 | 16 | 17 | 6 | 8.1 GB |
| `mimo-v2.6:9b` (measured after the download the owner allowed) | every field right | 2.5 | 14 | 11 | 10 | 7.7 GB |

- Generated corpus: all four get every field of the 25 invoices with text and the 12 payslips right (`measurements/synthetic-model-*.json`). The corpus is too regular to tell them apart, the owner's documents do.
- Owner's documents (a copy of the database, counts only, `measurements/private/`): where the model and code both found the gross, the invoice date, the number, the VAT or the IBAN, they agreed every time for all four models. They agree on the sender only 3 to 6 times of 14 to 17: code takes the first line of the page, which on the owner's invoices is often not the sender. That is the field the model is there for. The net agrees 2 of 4.
- `gemma4:12b-it-qat` finds the most with the fewest findings (values the model named that did not stand in their line) and needs 3 GB less than the 8-bit Qwen. `qwen3.5:9b` is the fastest but misses the gross on half the invoices.
- Only `mimo-v2.6:9b` found due dates on the owner's invoices (4, one of them also found by code). They mostly print none.
- `mimo-v2.6:9b`, the newest suitable model at ollama.com/library on 2026-11-25 (Xiaomi, built on Qwen3.5-9B, text and image, 6.6 GB), is fast but finds less than gemma4 on the owner's invoices and gives no value of the one receipt.
