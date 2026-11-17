# N-scale dimensions and rules used by Track Layer

Generated from library/standards-rail.json (2026-11-13). Do not edit by hand: `node scripts/standards.mjs md`.

status: verified = read in the NMRA document named in source (a number in the document, or a rule the document's numbers confirm); unverified = from general knowledge, a search snippet, or a drawing choice, do not present as a standard. Lengths are millimetres at 1:160 (model) unless the name says prototype. Dimensions in the plug-in are typical, not surveyed.

## Sources

- **rp11**: [NMRA Recommended Practice RP-11, Curvature and Rolling Stock (March 2018)](https://nmra.com/sites/default/files/standards/sandrp/General/RP/rp-11_2018.03.03r.pdf), National Model Railroad Association, retrieved 2026-11-13
- **rp72**: [NMRA Recommended Practice RP-7.2, Curved Track Centers (July 2017)](https://www.nmra.com/sites/default/files/standards/sandrp/General/RP/rp-7.2_curved_track_centers_july_2017.pdf), National Model Railroad Association, retrieved 2026-11-13
- **rp12**: [NMRA Recommended Practice RP-12, Turnouts, General (revised February 2015)](https://nmra.com/sites/default/files/standards/sandrp/General/RP/rp-12_2015.02.pdf), National Model Railroad Association, retrieved 2026-11-13
- **s32**: [NMRA Standard S-3.2, Track, Standard Scale (28 Jun 2026; the PDF's own title says Draft)](https://www.nmra.org/sites/default/files/standards/sandrp/Mech/S/s-3.2_track_standard_scale.pdf), National Model Railroad Association, retrieved 2026-11-13
- **rp71**: [NMRA Recommended Practice RP-7.1, Tangent Track Centers and Clearance Diagrams (rev. January 2019)](https://www.nmra.org/sites/default/files/standards/sandrp/General/RP/rp-7.1_tangent_track_centers_and_clearance_diagrams_2019.01.pdf), National Model Railroad Association, retrieved 2026-11-13
- **rp1235**: [NMRA Recommended Practice RP-12.35, N Scale Straight Switch Turnout (revised February 2015)](https://www.nmra.org/sites/default/files/standards/sandrp/General/RP/rp-12s_pdf_files_12.1_to_12.54.zip), National Model Railroad Association, retrieved 2026-11-13
- **rp151**: [NMRA Recommended Practice RP-15.1, Rail (4/24/2000)](https://www.nmra.org/sites/default/files/standards/sandrp/General/RP/rp-15.1.pdf), National Model Railroad Association, retrieved 2026-11-13

## Entries

### N scale proportion (`scale`, verified)

Source: s32, 2.10 N Scale (Ratio 1:160).

> 2.10 N Scale (Ratio 1:160)

### N track gauge (between railheads) (`gauge`, verified)

Source: s32, 2.10.1 Standard Scale, N Standard Gauge: G (Min, Tgt, Max).

> G 0.353 0.355 0.359 inch; 9.0 9.0 9.1 mm

The plug-in draws the target, 0.355 in = 9.017 mm. S-3.2 also gives check gauge 0.323 / 0.325 / 0.326 in, span 0.293 / 0.295 / 0.296, flangeway at frog 0.027 / 0.028 / 0.030, switch point spread 0.312 / 0.314 / 0.316 and wheel width code 72.

### Minimum radius and minimum turnout frog, N scale, locomotive-hauled trains (`radius-frog`, verified)

Source: rp11, Locomotive Hauled or MU Trains: Model Radius (mm) and Minimum Turnout Frog.

> N Scale 191 225 248 279 305 318 337 368 410 441 498 546 610 / Minimum Turnout Frog 4 4 1/2 5 5 5 6 6 6 6 6 6 7 8

Equipment should be restricted from layout parts with less than the specified minimum radius and frog. Note 9: diverging routes to be negotiated at high speed should be No. 8 for classes N and greater. Note 11: wye turnouts may be one frog number less. Note 12: curved turnouts go up one frog number if the inner curve is at minimum radius.

### Minimum radius and frog, N scale, traction (streetcar, interurban) (`radius-traction`, verified)

Source: rp11, Traction: Track Dimensions, Model Radius (mm), classes A to E.

> N Scale 70 95 124 149 191 / Min. Turnout Frog 2.5 3 3 4 4

### Minimum parallel track centres on tangent track, by era (prototype feet) (`tangent-centers`, verified)

Source: rp71, Modeling eras and recommended minimum tangent track centers.

> Old-Time/Narrow Gauge Before 1920 12 feet; Classic 1920 to 1969 13 feet; Early Modern 1969 to 1983 14 feet; Modern After 1983 14 feet

Also stated in RP-7.2. Scaled to N by dividing by 160 (arithmetic, not an NMRA table value): 12 ft = 22.86 mm, 13 ft = 24.77 mm, 14 ft = 26.67 mm. The tables of larger centres for curves (RP-7.2) are pictures and were not read: use these on straight track only.

### Track centres the N turnout crossover data assume (`crossover-basis`, verified)

Source: rp1235, Data for crossovers: point of frog to point of frog on parallel tracks.

> For Track Centers of: 0.975 (13 prototype feet) ... For Track Center Increment of: 0.075 (1 prototype foot)

0.975 in = 24.765 mm, the plug-in's default spacing, so its crossovers match these tables. 0.075 in = 1.905 mm per prototype foot, which is 304.8 / 160.

### Frog number to frog angle (`frog-angle`, verified)

Source: rp1235, (18) Frog Angle (deg.).

> FROG NUMBERS 4 5 6 7 8 ... / Frog Angle (deg.) 14.250 11.421 9.527 8.171 7.153 ...

The table lists 4 to 20. The rule angle = 2 atan(1 / (2 N)) reproduces every listed value to 0.001 degree (checked in core/tests/standards.rs).

### Tie size and spacing (`tie`, unverified)

Scaled from a typical prototype tie (8 ft 6 in by 9 in) at 19.5 in centres, divided by 160: about 16.2 mm long, 1.4 mm wide, 3.1 mm pitch. From memory, a drawing choice. NMRA S-3.2, RP-10, RP-12 and RP-15.1 give no tie dimensions.

### Rail drawn width and height (`rail`, unverified)

RP-15.1 Table 1 lists rail codes; Code 55 has A = 0.055 in (height, 1.4 mm). The column headings are in a drawing that was not readable as text, so the head width is not confirmed: the plan view draws the railhead 0.6 mm wide, a drawing choice.

### N scale straight switch turnout: lead, closure radius, switch rail, gage-line offsets (`turnout-n-straight`, verified)

Source: rp1235, RP-12.35 pages 1 and 2, lines (2), (4), (5), (8) to (18), (24) to (26).

> (8) Lead 2.806 3.175 3.544 4.634 5.075 ... ; (11) Curved Rail Radius 7.333 12.245 18.704 23.823 31.944 ... (frog numbers 4 5 6 7 8 ...)

Whole table in `tables.rp12_35` (inches). The plug-in draws the diverging route as the switch rail (straight, at the heel angle), then a closure arc of the listed radius turning to the frog angle, then straight; core/tests/turnouts.rs checks that this reproduces the listed lead and the three gage-line points to 0.1 mm. RP-12.36 (curved switch) is a different design and is not used.

## Table rp12_35: N scale straight switch turnout dimensions, frog numbers 4 to 20

Source: rp1235. Units: inch (angles in degrees). Fields:

- `switch_rail`: (2) switch rail length
- `heel_spread`: (4) switch heel spread
- `heel_angle_deg`: (5) switch heel angle
- `lead`: (8) lead from the point of switch to the theoretical point of frog
- `closure_straight`: (9) closure straight rail length
- `closure_curved`: (10) closure curved rail length
- `radius`: (11) closure curved rail radius
- `y1`: (12) first gage-line point offset
- `x1`: (13) first point distance
- `y2`: (14) mid point offset
- `x2`: (15) mid point distance
- `y3`: (16) third point offset
- `x3`: (17) third point distance
- `frog_angle_deg`: (18) frog angle
- `pf_to_icl`: (24) point of frog to intersection of centre lines
- `crossover_straight`: (25) straight track distance, point of frog to point of frog, 13 ft centres
- `crossover_crossover`: (26) crossover track distance, 13 ft centres

frog | lead | radius | switch_rail | heel_angle_deg | frog_angle_deg
--- | --- | --- | --- | --- | ---
4 | 2.806 | 7.333 | 1.11 | 3.211 | 14.25
5 | 3.175 | 12.245 | 1.104 | 3.228 | 11.421
6 | 3.544 | 18.704 | 1.119 | 3.185 | 9.527
7 | 4.634 | 23.823 | 1.677 | 2.125 | 8.171
8 | 5.075 | 31.944 | 1.74 | 2.048 | 7.153
9 | 5.394 | 40.333 | 1.749 | 2.037 | 6.36
10 | 5.875 | 51.524 | 1.843 | 1.933 | 5.725
11 | 6.855 | 60.672 | 2.301 | 1.549 | 5.205
12 | 7.212 | 73.038 | 2.325 | 1.532 | 4.772
13 | 7.599 | 87.546 | 2.365 | 1.506 | 4.405
14 | 7.986 | 103.626 | 2.41 | 1.478 | 4.091
15 | 9.431 | 112.621 | 3.193 | 1.116 | 3.818
16 | 9.8 | 131.38 | 3.201 | 1.113 | 3.58
17 | 10.158 | 149.407 | 3.23 | 1.103 | 3.369
18 | 10.516 | 168.858 | 3.261 | 1.092 | 3.182
19 | 10.925 | 192.183 | 3.311 | 1.076 | 3.015
20 | 11.334 | 217.256 | 3.364 | 1.059 | 2.864
