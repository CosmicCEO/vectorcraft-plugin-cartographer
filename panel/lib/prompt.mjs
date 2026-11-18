// The instructions every agent gets. Kept compact on purpose: a small on-device model must be able to follow them.

export function systemPrompt(lib) {
  const els = lib.elements.map((e) => `${e.id} (${e.width} m${e.directional ? ", has a direction" : ""}): ${e.label}`).join("\n");
  return `You turn a plain-language description of a road's cross-section into one line of "Cartographer shorthand". Reply with that one line and nothing else: no quotes, no explanation, no code fence.

SHORTHAND
- Items are separated by " | " and read LEFT TO RIGHT as seen facing the direction given after "facing".
- Start with "facing N:" (or S, E, W) when the description says which way the road runs, and list from the left side to the right side as seen facing that way. If no orientation is given, leave "facing" out and list left to right along the drawn line.
- An item is an element name, then optional pieces in any order: a direction (N S E W, or fwd / back) for elements that have one (travel lanes, bike lanes, turn pockets), a width ("3" = metres, "10ft"), a repeat ("x2").
- US right-hand traffic: facing north, the lanes on the LEFT (west) side run south (S) and the lanes on the RIGHT (east) side run north (N). In the same way, facing along the line, the lane on the RIGHT is "fwd" (goes the way the line goes) and the lane on the LEFT is "back" (comes toward you): a two-lane road is "lane back | lane fwd".
- Use only these elements (default width in brackets). If something is not listed, use the closest one; never invent a name:
${els}

EXAMPLES
Description: road runs north-south; 1 bike lane, 1 apron, 2 south lanes, a 3 m grassy curbed margin, 2 north lanes, apron, bike lane
facing N: bike-lane | apron | lane S x2 | margin 3 | lane N x2 | apron | bike-lane

Description: a quiet two-lane street with parking on both sides
parking-lane | lane back | lane fwd | parking-lane

EDITING
When a current cross-section is given, apply the requested change to it and return the COMPLETE updated line. Keep everything that was not mentioned exactly as it was (same order, same widths). "Delete/remove X" removes that item; "add X" inserts it where it belongs in left-to-right order; "make X wider/narrower" sets that item's width.`;
}

export function userPrompt({ description, current }) {
  return current ? `Current cross-section:\n${current}\n\nChange requested: ${description}` : `Description: ${description}`;
}

/** Pulls the shorthand line out of whatever the model said (it may wrap it in quotes or a code fence despite instructions). */
export function extractLine(reply) {
  const cleaned = String(reply).replace(/```[a-z]*\n?/gi, "").trim();
  const lines = cleaned.split("\n").map((l) => l.trim().replace(/^["'`]+|["'`]+$/g, "")).filter(Boolean);
  return lines.find((l) => /^facing\b/i.test(l)) ?? lines.find((l) => l.includes("|")) ?? lines[0] ?? "";
}

export function junctionSystemPrompt(lib) {
  const els = lib.elements.map((e) => e.id).join(", ");
  return `You turn a plain-language description of a road junction into one JSON object. Reply with the JSON object only: no explanation, no code fence.

The junction is where a short connecting line joins two road pieces; the main road carries on through it. The JSON describes what crosses it:
{"legs": [{"side": "...", "cross_section": "...", "length": 40}], "control": "none|signal|stop", "crosswalks": false, "radius": 6}
- legs: 0, 1 or 2 side roads. No legs means just carry the road across the gap. One leg is a T-junction, two legs (one each side) a four-way junction.
- side: "west", "east", "north" or "south" if the description gives compass directions, otherwise "left" or "right" as seen travelling along the main road. A leg must cross the main road, so never use the direction the main road runs.
- cross_section: the side road written in the Cartographer shorthand: elements separated by " | ", each an element name with optional direction (fwd, back), width in metres, or repeat (x2). Elements: ${els}. A two-lane road is "lane back | lane fwd" (right-hand traffic: facing outward along the leg, the lane on the right goes outward, the lane on the left comes in toward the junction); a "rural road" is two lanes, possibly with shoulders ("shoulder | lane back | lane fwd | shoulder").
- length: how far the leg is drawn, in metres (default 40).
- control: "signal" for traffic lights or signals, "stop" for stop signs on the side roads, otherwise "none".
- crosswalks: true if there are crosswalks, cross walks or pedestrian crossings.
- radius: corner radius in metres (default 6).
Do not add a turn pocket or turn lane here: describe only the legs, control and crosswalks.

EXAMPLE
Description: This is a 4 way intersection. The west leg is a two lane rural road, the east is a two lane rural road. There are traffic lights and cross walks at each side.
{"legs": [{"side": "west", "cross_section": "lane back | lane fwd", "length": 40}, {"side": "east", "cross_section": "lane back | lane fwd", "length": 40}], "control": "signal", "crosswalks": true, "radius": 6}

EDITING
When a current junction is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function junctionUserPrompt({ description, current }) {
  return current ? `Current junction:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `Description: ${description}`;
}

/** Pulls the first balanced JSON object out of a reply (models sometimes add words or a code fence). */
export function extractJson(reply) {
  const text = String(reply).replace(/```[a-z]*\n?/gi, "");
  const start = text.indexOf("{");
  if (start < 0) return "";
  let depth = 0;
  let inString = false;
  for (let i = start; i < text.length; i++) {
    const c = text[i];
    if (inString) {
      if (c === "\\") i++;
      else if (c === '"') inString = false;
    } else if (c === '"') inString = true;
    else if (c === "{") depth++;
    else if (c === "}" && --depth === 0) return text.slice(start, i + 1);
  }
  return "";
}

export function crossingSystemPrompt() {
  return `You turn a plain-language description of how two roads meet or cross into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"treatment": "signal", "crosswalks": false, "radius": null, "minor": null, "over": null, "main": null, "diameter": 40, "stub": "keep", "pocket": "none", "pocket_length": 30, "pocket_taper": 15, "slip": false, "opening": null, "gore": true, "accel_length": 120, "beacon": false, "gates": true}
- treatment, one of: "signal" (lighted intersection, traffic lights); "two-way-stop" (stop signs on the minor road only); "four-way-stop" (stop signs everywhere); "uncontrolled"; "driveway" (a small side entrance); "median-opening" (a gap in a median, or a U-turn crossover, on a divided road); "on-ramp" (a ramp joining with an acceleration lane, a merge); "off-ramp" (a ramp leaving, an exit, with a deceleration lane); "midblock-crossing" (a pedestrian crossing in the middle of a road, not at an intersection); "railroad" (a railway crossing, level crossing); "overpass" (one road bridges the other; an underpass is the same thing seen from below); "roundabout"; "cloverleaf" (interchange with loop ramps).
- crosswalks: true if the description mentions crosswalks, cross walks or pedestrian crossings at an intersection.
- radius: kerb corner radius in metres, or null for the usual (6 m, a driveway 1.5 m, a ramp none).
- minor: for a two-way stop, the id of the road that stops; null means the narrower road.
- over: for an overpass or cloverleaf, the id of the road that goes over; null means the first.
- main: for a median opening, mid-block crossing, railroad or ramp, the id of the road that stays (the divided road, the road being crossed, the through road); null lets the program decide.
- diameter: for a roundabout, the outside width in metres.
- stub: if one road runs only a short way past the crossing, "keep" draws it as a dead end (four-way) and "trim" cuts it off at the other road (three-way, T intersection). Use "trim" when asked to trim, cut off or remove the stub, or for a three-way or T.
- pocket: turn lanes on the approaches: "none", "left", "right" or "both". pocket_length is how long the full-width lane is (metres) and pocket_taper the length of the taper. "Twice as long" doubles pocket_length.
- slip: true for right-turn slip lanes with a triangular island.
- opening: the length of a median opening in metres; null means as long as the crossing road is wide.
- accel_length: the length of a ramp's acceleration or deceleration lane in metres. gore: false to leave out the chevron-marked gore where a ramp parts from or joins the road. A merge or converge is an "on-ramp"; a diverge or exit is an "off-ramp"; left or right is decided by which side of the road the ramp line lies on.
- beacon: true for flashing beacons at a mid-block crossing. gates: false for a railroad crossing without gates.
The two roads have the ids given in the request; "minor", "over" and "main" must be one of those two ids or null. Leave out "roads": the program adds it.

EXAMPLES
Description: lighted intersection with crosswalks
{"treatment": "signal", "crosswalks": true, "radius": null, "minor": null, "over": null, "main": null, "diameter": 40, "stub": "keep", "pocket": "none", "pocket_length": 30, "pocket_taper": 15, "slip": false, "opening": null, "gore": true, "accel_length": 120, "beacon": false, "gates": true}

Description: lighted three-way, trim off the little stub, with left turn pockets
{"treatment": "signal", "crosswalks": true, "radius": null, "minor": null, "over": null, "main": null, "diameter": 40, "stub": "trim", "pocket": "left", "pocket_length": 30, "pocket_taper": 15, "slip": false, "opening": null, "gore": true, "accel_length": 120, "beacon": false, "gates": true}

Description: railroad crossing with gates
{"treatment": "railroad", "crosswalks": false, "radius": null, "minor": null, "over": null, "main": null, "diameter": 40, "stub": "keep", "pocket": "none", "pocket_length": 30, "pocket_taper": 15, "slip": false, "opening": null, "gore": true, "accel_length": 120, "beacon": false, "gates": true}

EDITING
When a current intersection is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function hubSystemPrompt() {
  return `You turn a plain-language description of an intersection where three or more roads meet at one point into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"treatment": "signal", "crosswalks": false, "radius": null, "minor": null, "stub": "keep"}
- treatment: "signal" (traffic lights); "two-way-stop" (the narrowest road, or the road named in minor, has stop signs); "four-way-stop" (all-way stop, stop signs on every road); "uncontrolled".
- crosswalks: true if crosswalks are mentioned. radius: kerb corner radius in metres, or null for 6.
- minor: the id of the road that stops for a two-way stop, or null.
- stub: "trim" cuts off short stubs of road past the meeting point, "keep" (default) leaves them as dead ends.
The roads have the ids given in the request. Leave out "roads": the program adds it.

EXAMPLE
Description: five-way with traffic lights and crosswalks
{"treatment": "signal", "crosswalks": true, "radius": null, "minor": null, "stub": "keep"}

EDITING
When a current intersection is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function hubUserPrompt({ description, current, roads }) {
  const head = `The roads are "${roads.join('", "')}".`;
  return current ? `${head}\nCurrent intersection:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `${head}\nDescription: ${description}`;
}

export function crossingUserPrompt({ description, current, roads }) {
  const head = `The two roads are "${roads.join('" and "')}".`;
  return current ? `${head}\nCurrent intersection:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `${head}\nDescription: ${description}`;
}

export function transitionSystemPrompt() {
  return `You turn a plain-language description of a change in the number of lanes along a road into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"change": "add", "side": "right", "direction": "fwd", "count": 1, "at": null, "length": null, "speed": 35}
- change: "add" (a lane is added, the road widens) or "drop" (a lane ends, the road narrows).
- side: "left" or "right" as the driver sees it, driving in the direction of the lanes concerned. In right-hand traffic "left" is the side toward the centre of the road and "right" is the side toward the edge of the road. "Merge left" and "lane ends on the left" are drop on the left; "merge right" is drop on the right; "new lane on the right" is add on the right.
- direction: "fwd" for the lanes that run the way the line was drawn (the right half of the line), "back" for the lanes against it. Use "fwd" unless the description says the other direction.
- count: how many lanes, 1 to 3.
- at: metres from the start of the line where the taper begins; null means the middle of the line.
- length: the taper length in metres; null means the MUTCD formula from the speed.
- speed: the speed limit in mph, used for the taper length (default 35).
Add "remove": true only to take a transition away.

EXAMPLES
Description: the right lane ends after 300 metres
{"change": "drop", "side": "right", "direction": "fwd", "count": 1, "at": 300, "length": null, "speed": 35}

Description: a second lane is added on the left, 55 mph
{"change": "add", "side": "left", "direction": "fwd", "count": 1, "at": null, "length": null, "speed": 55}

EDITING
When a current transition is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function transitionUserPrompt({ description, current, crossSection }) {
  const head = `The road's cross-section before the change: ${crossSection}`;
  return current ? `${head}\nCurrent transition:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `${head}\nDescription: ${description}`;
}

export function waterSystemPrompt() {
  return `You turn a plain-language description of a water feature drawn on a map into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"kind": "flowing", "name": null, "width": 8, "flow": "fwd", "speed": null, "side": null, "bank_left": 1.5, "bank_right": 1.5, "slope": 2}
- kind: "still" (a closed shape: lake, pond, reservoir), "flowing" (an open line: creek, stream, river, culvert channel) or "shore" (an open line that is the edge of the sea or a lake, with water on one side).
- name: the water's name, or null.
- width: the water's width in metres for flowing water (a creek 3 to 8, a river 20 to 100); leave out for still water and shores.
- flow: "fwd" if the water runs the way the line was drawn, "back" if against it, "none" if it does not flow. Flowing water only.
- speed: current speed in metres per second, or null.
- side: "left" or "right" of the line (facing the way it was drawn) where the water is, for a shore only.
- bank_left, bank_right: height of the bank above the water in metres on each side of the line (for still water both give the bank height all round; for a shore only bank_left, the land side, is used). 0 means no bank is drawn.
- slope: bank side slope as horizontal metres per metre of height (2 means 2:1, flat is 1 to 6).
Leave out anything that is not mentioned.

EXAMPLES
Description: a creek about 6 m wide with steep banks 2 m high
{"kind": "flowing", "width": 6, "bank_left": 2, "bank_right": 2, "slope": 1.5}

Description: ocean on the right of the line
{"kind": "shore", "side": "right"}

Description: a pond with gentle banks
{"kind": "still", "bank_left": 1, "bank_right": 1, "slope": 4}

EDITING
When a current description is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function embankmentSystemPrompt() {
  return `You turn a plain-language description of a road built up on fill (an embankment) into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"height": 3, "slope": 2, "from": null, "to": null, "sides": "both", "ramp": 15}
- height: how high the road stands above the ground beside it, in metres (default 2).
- slope: side slope as horizontal metres per metre of height (2 means 2:1, steeper is 1.5, flatter is 3; range 1 to 6).
- from, to: metres from the start of the line where the fill starts and ends; null means the start or end of the line.
- sides: "both", "left" or "right" (as seen facing the way the line was drawn).
- ramp: metres over which the slope grows from nothing at each end (default 15).
Add "remove": true only to take an embankment away.

EXAMPLES
Description: road on 4 m of fill between 200 and 450 m, 3:1 slopes
{"height": 4, "slope": 3, "from": 200, "to": 450, "sides": "both", "ramp": 15}

Description: low embankment on the left side only
{"height": 1.5, "slope": 2, "sides": "left"}

EDITING
When a current embankment is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function wetSystemPrompt() {
  return `You turn a plain-language description of how a road crosses water into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"treatment": "bridge", "deck_height": 3, "abutment": 4, "shape": "pipe", "count": 2, "size": 1.2, "fill_height": 2.5, "slope": 2}
- treatment: "bridge" (the road is carried over the water on a deck), "culvert" (pipes or boxes under the road, which stands on fill; creeks and rivers only), "causeway" (the road on fill straight across the water).
- bridge: deck_height is the clearance above the water in metres; abutment is how far the deck runs past the banks, in metres.
- culvert: count pipes (1 to 6) of size metres (diameter for a pipe, width for a box), shape "pipe" or "box"; fill_height and slope describe the fill over it.
- causeway: fill_height and slope.
Leave out anything that is not mentioned. Leave out "road" and "water": the program adds them.

EXAMPLES
Description: three 1.5 m pipes under a high embankment
{"treatment": "culvert", "shape": "pipe", "count": 3, "size": 1.5, "fill_height": 4}

Description: bridge with 8 m clearance
{"treatment": "bridge", "deck_height": 8}

EDITING
When a current description is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was.`;
}

export function featureUserPrompt({ description, current, head }) {
  const h = head ? `${head}\n` : "";
  return current ? `${h}Current description:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `${h}Description: ${description}`;
}

// ---- tracks (N-scale model railroad), ported from Track Layer

export function trackSystemPrompt() {
  return `You turn a plain-language description of N-scale model railroad track into one JSON object. Reply with the JSON object only: no explanation, no code fence.

The track is built along a line the user drew. There are two kinds, chosen with "kind":

1. Parallel tracks side by side along the line:
{"kind": "tracks", "tracks": 2, "spacing": "13ft", "ballast": true, "ties": true}
- tracks: how many, 1 to 12. spacing: distance between track centres, as a number of millimetres or text such as "25mm", "1in" or "13ft" (13 ft is the usual; "ft" means real-world feet), or an era: "old-time" (12 ft), "classic" (13 ft, the default) or "modern" (14 ft).
- ballast: false to leave out the roadbed; ties: false to leave out the ties.

2. A yard: through tracks (mains) along the line, a ladder of turnouts off each side, spurs (sidings) side by side, and optionally a mirror-image ladder that brings the spurs back to the mains:
{"kind": "yard", "mains": 2, "left": 8, "right": 4, "ends": "merge", "frog": 6, "spacing": "13ft", "at": 0, "length": 2400}
- mains: through tracks along the drawn line, 1 to 4. left, right: spurs on that side, 0 to 16, as seen travelling from the start of the line to its end.
- ends: "merge" when the spurs join the mains again at the far end ("merge back", "come back together", "loop back"); "stub" when the spurs end in the yard (the default).
- frog: turnout number, a whole number from 4 to 20 (6 is typical; "No. 8 turnouts" is 8). spacing as above.
- at and length: where the yard starts and how long it is, in millimetres or text such as "1m", "2ft". Leave them out to use the whole line.
- crossovers: a list of crossovers between neighbouring mains, [{"from": 1, "to": 2, "at": "1m"}]. Mains are numbered 1, 2, 3 ... from the left as seen travelling along the line. "from" is the main where the crossover starts as the line is followed, "to" the main it reaches. Leave out "at" to put it in the middle. A crossover needs at least two mains.

Leave out every field the description does not mention (never write null). If it mentions a yard, spurs, sidings, a ladder, turnouts, switches or crossovers use "yard" (with mains = the number of through tracks); otherwise "tracks". "Double track" is tracks: 2; "triple track" is 3.

EXAMPLES
Description: two mainlines enter a yard with 8 spurs to the left and four to the right, they merge back down to two at the end
{"kind": "yard", "mains": 2, "left": 8, "right": 4, "ends": "merge"}

Description: a single main with three sidings on the right, number 8 turnouts
{"kind": "yard", "mains": 1, "right": 3, "frog": 8}

Description: double track with a crossover in the middle and a siding on the right
{"kind": "yard", "mains": 2, "right": 1, "crossovers": [{"from": 1, "to": 2}]}

Description: double track with a crossover from the left track to the right track 40 cm along
{"kind": "yard", "mains": 2, "crossovers": [{"from": 1, "to": 2, "at": "400mm"}]}

Description: triple track, 28 mm apart, no roadbed
{"kind": "tracks", "tracks": 3, "spacing": "28mm", "ballast": false}

EDITING
When a current description is given, apply the requested change and return the COMPLETE updated object, keeping everything not mentioned exactly as it was. "Add two more spurs on the left" adds two to the current left count.`;
}

export function trackUserPrompt({ description, current }) {
  return current ? `Current description:\n${JSON.stringify({ kind: current.kind, ...current.spec })}\n\nChange requested: ${description}` : `Description: ${description}`;
}

export function jointSystemPrompt() {
  return `You turn a plain-language description of how two railway lines meet into one JSON object. Reply with the JSON object only: no explanation, no code fence.

{"kind": "crossing", "main": "a"}
{"kind": "branch", "frog": 6}
{"kind": "join"}
- kind: "crossing" when the two tracks cross each other (a diamond). "branch" when one line is a siding, spur or turnout leaving the other. "join" when two lines continue into each other end to end.
- main (crossing): the id of the line whose track runs through the crossing (the other line's ties stop at its edge). Leave it out unless the description says which is the main, which is more important, or which gives way. The two lines have the ids given in the request: "main" must be one of them. (A branch's main is worked out from the drawing: leave it out.)
- frog (branch): the turnout number, a whole number from 4 to 20, if the description names one ("a number 8 turnout"). Leave it out otherwise: the program picks the one that fits.

EXAMPLES
Description: they cross
{"kind": "crossing"}

Description: line b crosses over the main line a, a is the main
{"kind": "crossing", "main": "a"}

Description: b is a siding off a
{"kind": "branch"}

Description: b leaves a with a number 8 turnout
{"kind": "branch", "frog": 8}

Description: the two lines continue into each other
{"kind": "join"}

EDITING
When a current description is given, apply the requested change and return the COMPLETE updated object.`;
}

export function jointUserPrompt({ description, current, lines, found }) {
  const head = `The two lines are "${lines[0]}" and "${lines[1]}".${found?.ok ? ` From the drawing: ${found.summary}.` : ""}`;
  return current ? `${head}\nCurrent description:\n${JSON.stringify(current)}\n\nChange requested: ${description}` : `${head}\nDescription: ${description}`;
}
