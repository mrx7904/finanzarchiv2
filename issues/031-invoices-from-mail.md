# #31 Invoices from mail

**Goal:** Every invoice, receipt and payslip that comes by mail is in the archive, and every mail is looked at once.
**Requirements**: FA-35 · **Depends on**: #29, #14

## Done when

- [ ] A prefilter by code: a PDF attachment, or a word of an invoice in subject or sender. Only those mails go to the local model, which answers from sender, subject, names of attachments and the start of the text. It is never asked how sure it is.
- [ ] Every mail looked at is a row of `mail_messages` under the hash of its Message-ID, with the verdict and no body. A mail is never asked about twice, and what the owner dismissed never comes back.
- [ ] A PDF attachment goes through the intake with the mail as its source. A mail that is the invoice itself becomes a PDF of its text with sender, date and subject on top, made without loading anything from outside (no image, no link followed).
- [ ] Measured on one month of the owner's mail, counts only: mails looked at, taken, left out, seconds.
