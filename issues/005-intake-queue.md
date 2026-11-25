# #05 One intake queue that survives a restart

**Goal:** Every import becomes one job in `intake_jobs` that moves through fixed stages, resumes after a restart where it stood, can be paused and runs several jobs at once.
**Requirements**: FA-09 · **Depends on**: #04

## Done when

- [ ] The stages are `stored`, `text`, `classified`, `read`, `checked`, `filed` and `waiting` as in `decision-data-model.md`. Stages not built yet pass through.
- [ ] Killing the app in the middle of 200 generated files and starting it again ends with 200 documents, none twice and none lost, proved by an integration test that stops the worker at random points.
- [ ] Pausing stops new work at once, the running job keeps its last stage.
- [ ] The number of workers is one setting with one default. A measurement of 200 files with 1 and with 4 workers stands in the commit.
- [ ] The UI shows "Eingang" with the jobs that run, wait or failed, each with its reason.

## Out of scope

- The readers behind the stages, they belong to stage 2
