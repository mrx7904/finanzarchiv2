# #43 Categories by the model

**Goal:** Every party has a category without the owner assigning 160 of them, so the evaluations tell where the money goes.
**Requirements**: FA-46 · **Depends on**: #25, #14

## Done when

- [x] The local model chooses one of the categories, or none, for every party without one, from its name, the count and rhythm of its bookings, a typical amount, its kinds of booking, up to six purposes and the senders of its documents. Payment services (PayPal, Klarna) get none, their payee tells.
- [x] A category the owner chose stays (`category_by`), the model never changes it. A category the owner removed stays removed. A party is asked once.
- [x] Runs in the background whenever the model is there, never blocking the window. The window shows no explanation, only the category.
- [x] Measured on invented parties with known categories (right of all), then on a copy of the owner's archive (counts per category), within the GPU budget.

**Status:** done on 2026-11-26, measured in `knowledge/decision-parties.md`.
