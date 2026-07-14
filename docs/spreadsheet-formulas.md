# PAS Reconciliation Spreadsheet — Review & Formula Extraction

Source workbook: `PAS Rec with EMR (Excel LibreOffice Calc) TEMPLATE.xlsx` (Doctors of BC / PSP, v. Jul 7 2026). Built to run in both Excel and LibreOffice Calc.

## What the workbook does

A **patient-panel reconciliation tool** that compares a clinic's **EMR patient panel** against the **Provincial Attachment System (PAS)** patient list, matching patients by their **PHN** (Personal Health Number). It produces three "to review" worklists plus status/duplicate statistics.

## Sheet-by-sheet architecture

| # | Sheet | Role |
|---|-------|------|
| 1 | `1 Guide` | Instructions + the two **input cells** (`F5`, `F7`) where you pick which column holds the PHN |
| 2 | `2 EMR Panel` | **Paste zone** — raw EMR export pasted at A1 |
| 3 | `3 PAS Patient List` | **Paste zone** — raw PAS CSV pasted at A1 |
| 7 | `7 Patients in EMR` | Engine: pulls from sheet 2, matches against sheet 8 |
| 8 | `8 Patients in PAS` | Engine: pulls from sheet 3, matches against sheet 7, auto-detects status/date columns |
| 4 | `4 EMR No Match - To Review` | Output: in EMR but not in PAS |
| 5 | `5 PAS Match - To Review` | Output: matched but Pending/Not-MRP/Deceased/Removed |
| 6 | `6 PAS No Match - To Review` | Output: in PAS but not in EMR |

The paste sheets (2, 3) are dumb — they hold raw data. **All logic lives in sheets 7, 8 and the three output sheets.** Note the **7-row offset**: engine row 8 reads paste-sheet row 1 (e.g. `'7 Patients in EMR'!D8` → `'2 EMR Panel'!A1`).

## The two inputs

Data-validation dropdowns (LibreOffice stores these as `x14:list` extensions):

- **`'1 Guide'!F5`** → list source `'7 Patients in EMR'!$D$7:$X$7` (letters A–U). Which column of the EMR panel holds the PHN.
- **`'1 Guide'!F7`** → list source `'8 Patients in PAS'!$I$7:$R$7` (letters A–J). Which column of the PAS list holds the PHN. Default `B`.

These two letters propagate everywhere via the indirect-lookup trick below.

## Engine sheet 7 — `7 Patients in EMR`

Header region (rows 1–7) holds labels/refs; **row 8 is the template row copied down to row 3007**.

### Selected-column echo (A4)
```
A4: =IF('1 Guide'!$F$5="","",'1 Guide'!$F$5)        → e.g. "C"
```

### Summary counters (row 2)
```
F2: =COUNTIF($A:$A,"Patient Match")                   matched count
G2: =COUNTIF($A:$A,"No Match in PAS - To Review")     unmatched count
I2: =IF(OR($A$4="",'8 Patients in PAS'!$A$4=""),0,
       COUNTA(UNIQUE(INDIRECT("'2 EMR Panel'!"&$A$4&"1:"&$A$4&3000),,0))-1)   unique PHNs
J2: [ARRAY] =IF(OR($A$4="",'8'!$A$4=""),0,
            SUM(--(COUNTIF(INDIRECT("'2 EMR Panel'!"&$A$4&"1:"&$A$4&3000),
                   UNIQUE(INDIRECT("'2 EMR Panel'!"&$A$4&"1:"&$A$4&3000)))>1)))  duplicate PHNs
```

### Per-row formulas (row 8, filled down to 3007)

`D8:X8` — pull each EMR column (A–U) into a normalized grid. The **selected PHN column is coerced to a number** with spaces, hyphens and non-breaking spaces stripped; everything else stays as text:
```
D8: =IF('2 EMR Panel'!A1="","-",
       IF(D$7=$A$4,
          IFERROR(VALUE(SUBSTITUTE(SUBSTITUTE(SUBSTITUTE('2 EMR Panel'!A1," ",""),"-",""),CHAR(160),"")),
                  '2 EMR Panel'!A1),
          '2 EMR Panel'!A1))
```

`C8` — extracts the **PHN** for this row using `INDEX` on the whole D:X grid, with the column chosen dynamically by ASCII code of the selected letter (`CODE("C")-64` = 3):
```
C8: [ARRAY] =IFERROR(INDEX($D:$X, ROW(), CODE($A$4)-64),"-")
```

`A8` — the **classification**: needs ≥2 non-dash fields and a real PHN, then `VLOOKUP`s the PHN against sheet 8's deduped PHN column (E):
```
A8: =IF(OR($A$4="",'8 Patients in PAS'!$A$4=""),"-",
       IF(COUNTIF(D8:X8,"<>-")<2,"-",
          IF(C8="-","No Match in PAS - To Review",
             IF(ISERROR(VLOOKUP(C8,'8 Patients in PAS'!E:E,1,FALSE)),
                "No Match in PAS - To Review","Patient Match"))))
```

`B8` — running sequential number assigned only to "No Match" rows (drives the output sheet's VLOOKUP key):
```
B8: =IF($A8="-","-",IF($A8<>"No Match in PAS - To Review","-",
        COUNTIF(A$8:A8,"No Match in PAS - To Review")))
```

## Engine sheet 8 — `8 Patients in PAS`

Richer, because the PAS export has **status** and **MRP-updated date** columns whose position varies, so the sheet auto-detects them.

### Selected-column echo
```
A4: =IF('1 Guide'!$F$7="","",'1 Guide'!$F$7)
```

### Status counters (row 2)
```
L2/M2/N2/O2/P2: COUNTIF($B:$B,"Confirmed"|"Pending"|"Not the MRP"|"Deceased"|"Removed")
Q2: =SUM(L2:P2)        total of all statuses
```

### Auto-detect "PAS MRP Updated" and "PAS MRP Status" columns (rows 5–6)
Each column I:R scans rows 8–507 for a header containing the text:
```
I5: =IF(COUNTIF(I8:I507,"*PAS MRP Updated*")>0, COLUMN()-8, 0)
B5: =IF(OR(MAX(I5:R5)=0,MAX(I5:R5)>10), 1, MAX(I5:R5))   → picks the detected col, default 1
```
Same pattern in row 6 for `"*PAS MRP Status*"`. `B5`/`B6` end up holding the column-index (1–10) of those two fields.

### Per-row formulas (row 8, filled down)

`I8:R8` — pull PAS columns A–J (the selected PHN column is coerced to a number, like sheet 7):
```
I8: =IF('3 PAS Patient List'!A1="","-",
       IF(I$7=$A$4, IFERROR(VALUE('3 PAS Patient List'!A1),'3 PAS Patient List'!A1),
                      '3 PAS Patient List'!A1))
```

`F8` — the raw selected PHN (array INDEX):
```
F8: [ARRAY] =IFERROR(INDEX($I:$R, ROW(), CODE($A$4)-64),"-")
```

`G8` — the MRP-updated value from the auto-detected column:
```
G8: [ARRAY] =INDEX($I$8:$R$10007, ROW()-7, $B$5)
```

`H8` — parses `G8` into a real date (handles both serial numbers and `D/M/YYYY` strings):
```
H8: =IF(ISNUMBER(G8), G8,
       IFERROR(DATE(RIGHT(G8,4), LEFT(G8,FIND("/",G8)-1),
                    MID(G8, FIND("/",G8)+1,
                        FIND("/",G8,FIND("/",G8)+1)-FIND("/",G8)-1)),"-"))
```

`B8` — pulls the **status** text ("Confirmed"/"Pending"/"Not the MRP"/"Deceased"/"Removed") from the auto-detected status column:
```
B8: [ARRAY] =IF(OR($A$4="",'7'!$A$4=""),"-",
           IF(E8="-","-",
              IFERROR(INDEX($I$8:$R$10007, ROW()-7, $B$6),"-")))
```

`E8` — **deduplication**: keeps the PHN only on the row with the *latest* MRP-updated date among duplicates (this is the key the EMR sheet VLOOKUPs against):
```
E8: =IFERROR(IF(COUNTIFS($F$8:$F$10007, F8, $H$8:$H$10007, ">"&H8)+1 = 1, F8, "-"),"-")
```

`A8` — classification, mirrored from sheet 7 but checking the deduped `E8` against the EMR PHNs:
```
A8: =IF(OR($A$4="",'7'!$A$4=""),"-",
       IF(B8="-","-",
          IF(COUNTIF(I8:R8,"<>-")<2,"-",
             IF(ISERROR(VLOOKUP(E8,'7 Patients in EMR'!C:C,1,FALSE)),
                "No Match in EMR - To Review","Patient Match"))))
```

`C8` / `D8` — running sequence numbers feeding output sheets 6 and 5:
```
C8: =IF($A8="-","-",IF($A8="Patient Match","-",
        IF(OR($B8="Deceased",$B8="Removed"),"-",
           COUNTIFS(A$8:A8,"No Match in EMR - To Review",
                    B$8:B8,"<>Deceased",B$8:B8,"<>Removed"))))      → for sheet 6
D8: =IF($A8="-","-",IF($A8="No Match in EMR - To Review","-",
        IF($B8="Confirmed","-",
           COUNTIFS(B$8:B8,"<>Confirmed",A$8:A8,"Patient Match"))))  → for sheet 5
```

## Output sheets 4 / 5 / 6 — the "To Review" lists

All three are identical in shape: column A holds 1, 2, 3…; a `VLOOKUP(A, engine_range, col_index)` pulls each patient's fields. The only differences are **which engine sheet/range** and **the starting column index**.

```
A6,A7,…   : literal 1, 2, 3, …  (the lookup key)
B6        : =IF(COUNTIF(C6:J6,"<>-")>0, 1, "-")          row-has-data flag
H2        : =COUNTIF($B$6:$B$1005,1)                       visible count
I1        : ="There are "&MAX($A$6:$A$505)&" or more patients, please reach out to PSP@doctorsofbc.ca for support"
```

The data-pull formula per sheet (row 6, filled across C:J and down):

| Sheet | Lookup range | Col indices (C4:J4) | Example formula |
|---|---|---|---|
| **4** EMR No Match | `'7 Patients in EMR'!$B$7:$X$6007` | 2…9 | `=IF('7'!$F$2=0,"-",IFERROR(VLOOKUP($A6,'7 Patients in EMR'!$B$7:$X$6007,C$4,FALSE),"-"))` |
| **5** PAS Match | `'8 Patients in PAS'!$D$7:$R$6007` | 6…13 | `=IF('7'!$F$2=0,"-",IFERROR(VLOOKUP($A6,'8 Patients in PAS'!$D$7:$R$6007,C$4,FALSE),"-"))` |
| **6** PAS No Match | `'8 Patients in PAS'!$C$7:$R$6007` | 7…14 | `=IF('7'!$F$2=0,"-",IFERROR(VLOOKUP($A6,'8 Patients in PAS'!$C$7:$R$6007,C$4,FALSE),"-"))` |

The column-index offsets account for where the sequence-number column sits relative to the start of each lookup range (B / D / C respectively), so C always lands on the PHN and the following columns pull name fields.

## Guide sheet summary panel (cells E10:G13)

Feeds the dashboard counts and warns if a list exceeds 1000 patients:
```
F10: ='7 Patients in EMR'!F2                          matched total
F11: =IF(F10=0,0,MAX('7 Patients in EMR'!$B$8:$B$10007))
G11: =IF(F11>MAX('4 EMR No Match - To Review'!$A$6:$A$1005),
         "Note: only first "&MAX(...)&" on worksheet","")
```
(Analogous for F12/G12 → sheet 5, F13/G13 → sheet 6.) Plus PHN-format sanity checks in `G5`/`G7` (warns if a value isn't a 10-digit number starting with 9).

## How the matching actually works (summary)

1. User pastes EMR + PAS data and picks the PHN column in each.
2. Both engines rebuild a normalized grid and isolate the PHN as a **number** (so `9876543210` text and `9876 543 210` both match).
3. **Sheet 8 deduplicates PAS PHNs**, keeping only the record with the most recent "MRP Updated" date (`E8`).
4. Each side classifies every row by `VLOOKUP`ing its PHN against the other side's PHN column.
5. Running counters (`B`/`C`/`D`) assign 1, 2, 3… to each category.
6. The three output sheets `VLOOKUP` those sequence numbers to render clean, reviewable lists capped at 1000 rows.

## Things to flag if rebuilding elsewhere

- The whole design leans on `INDIRECT` for column selection — volatile, recalc-heavy on big panels.
- Row limits are hard-coded throughout (3007 in engines, 10007 in some ranges, 6007 in output lookups, 1000-row cap on review lists).
- The dedup tiebreaker silently drops all-but-newest duplicates without surfacing them on any review sheet.
