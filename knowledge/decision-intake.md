# Decision: one intake queue in the database, worked by threads of the core

**Decided:** 2026-11-25 (#05) · applies to `src-tauri/src/intake/`, `migrations/0003_intake_jobs.sql`, the import commands, `src/intake/`

## Decision

- Every way in (dialog and drop now, folders and mail in stage 4) only creates jobs: one row of `intake_jobs` per file, folders walked through, system files left out. The import commands return how many files went into the queue, the window learns the rest from the queue.
- A job has a stage: `queued`, then `stored`, `text`, `classified`, `read`, `model` (only while the local model gives no answer, with the German reason in `error`, migration 0008), `checked`, and one of the ends `filed`, `waiting`, `failed`. Each stage is committed on its own, so a restart resumes at the stage a job reached. What `text` to `model` do stands in `decision-reading.md`.
- Storing a file is one transaction: the row of the document and the new stage of the job are committed together. The file is written under its hash before, and a file that a killed attempt left there is found and kept (its name is its content). So a kill at any moment leaves either both or neither, never a document without its job or a job that counts a file twice.
- A refused file ends `failed` with its German reason and stays listed under "Eingang" until the owner hides it. The same bytes a second time end `filed` with `known = 1` and point to the existing document.
- Workers are threads of the core. Reading and checking a file need no database and run side by side, writing goes through the one connection of the archive. Which worker takes which job is held in memory: the app runs once per data folder, and a restart frees every claim.
- Pausing stops a worker between two stages. The running job keeps the stage it reached.
- The number of workers is the setting `intake.workers`, default `DEFAULT_WORKERS = 2` in `intake/jobs.rs`, nowhere else.
- The window hears `IntakeChanged` after every change and reloads at most every 150 ms (`useIntakeChanged`), so two hundred files mean a handful of reloads.

## Evidence

On 2026-11-25 on Windows 11:

- Thread kill test: three seeds, 200 files each (every tenth a PDF), a worker stopped at a random one of four points and the archive opened afresh, again and again. Each round ended with 200 documents, 200 files under their hash and 200 jobs filed as new, after more than ten kills.
- Process kill test: the test binary starts itself as a child that works the queue with two workers, and the parent kills that process (`Child::kill`) after 20 to 270 ms, until a child finishes. 6 kills, 7 starts, 200 of 200 once.
- Both tests were shown able to fail: with the document committed before the job, the thread test counted 195 of 200 and the process test 196, 192 and 196 in three runs.
- The release build resumed 200 queued jobs at its start and showed "Heute 200 aufgenommen".
- `measure_workers` (release, 200 files of a few hundred bytes, every tenth a PDF): 1 worker 386 ms, 2 workers 327 ms, 4 workers 310 ms. Storing small files is bound by the one database connection, and PDFium works one call at a time (`thread_safe`). Two workers take most of the gain. Measure again when stage 2 adds reading by the model, which waits on Ollama and will profit from more workers.

## Not verified

- Dropping files on the window (built on the window event, needs a real drag).
- Driving the folder dialog by UI Automation worked once in #04 and failed several times here: the dialog's name field was not found in time. The kill tests above replace that way of testing.
