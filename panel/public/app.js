// The panel page. Everything the server sends is shown with textContent, never as HTML.
const $ = (id) => document.getElementById(id);
// ?zoom=0.6 shrinks the page (for a small window, or a screenshot).
const zoom = Number(new URLSearchParams(location.search).get("zoom"));
if (zoom > 0.2 && zoom < 3) document.body.style.zoom = String(zoom);
let state = null;

async function post(action, body = {}) {
  const res = await fetch(`/api/${action}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  if (!res.ok) throw new Error((await res.json().catch(() => ({}))).error ?? res.statusText);
}
const act = (action, body) => post(action, body).catch((e) => show({ ...state, message: e.message, messageIsError: true }));

const WAYS = { N: "↑ northbound", S: "↓ southbound", E: "→ eastbound", W: "← westbound", fwd: "↑ along the line", back: "↓ against the line" };
const rgb = (f) => `rgb(${f.map((c) => Math.round(c * 255)).join(",")})`;

function el(tag, props = {}, ...kids) {
  const e = Object.assign(document.createElement(tag), props);
  for (const k of kids) e.append(k);
  return e;
}

function drawItems(list, bar, total, view, editable) {
  list.replaceChildren();
  bar.replaceChildren();
  const sum = view.total_width || 1;
  for (const it of view.items) {
    bar.append(el("i", { title: `${it.label} ${it.width} m`, style: `width:${(it.width / sum) * 100}%;background:${it.fill ? rgb(it.fill) : "#ccc"}` }));
    const li = el("li", {}, el("span", { className: "sw", style: `background:${it.fill ? rgb(it.fill) : "#ccc"}` }), el("span", { className: "name" }, it.label));
    if (it.way) li.append(el("span", { className: "way" }, WAYS[it.way] ?? it.way));
    if (editable) {
      const w = el("button", { className: `w${it.explicit_width ? " fixed" : ""}`, title: "Change this width (metres)" }, `${it.width} m`);
      w.onclick = () => {
        const v = prompt(`Width of "${it.label}" in metres`, String(it.width));
        if (v !== null && Number(v) > 0) act("edit", { op: ["width", it.index, Number(v)] });
      };
      li.append(w);
      const mv = (label, title, to) => {
        const b = el("button", { className: "plain tiny", title }, label);
        b.disabled = to < 0 || to >= view.items.length;
        b.onclick = () => act("edit", { op: ["move", it.index, to] });
        return b;
      };
      const del = el("button", { className: "plain tiny", title: `Remove ${it.label}` }, "✕");
      del.disabled = view.items.length < 2;
      del.onclick = () => act("edit", { op: ["delete", it.index] });
      li.append(mv("◀", "Move left", it.index - 1), mv("▶", "Move right", it.index + 1), del);
    } else {
      li.append(el("span", { className: "w" }, `${it.width} m`));
    }
    list.append(li);
  }
  total.textContent = `${view.items.length} elements, ${view.total_width} m wide${view.facing ? `, read facing ${view.facing}` : ""}`;
}

function fillList(id, items) {
  const ul = $(id);
  ul.replaceChildren(...(items ?? []).map((t) => el("li", {}, t)));
  ul.hidden = !items?.length;
}

function show(s) {
  state = s;
  $("notesCard").hidden = !s.notes?.length;
  $("notes").replaceChildren(...(s.notes ?? []).map((n) => el("li", {}, n)));
  $("dot").classList.toggle("on", !!s.connected);
  $("problem").hidden = !s.error;
  $("problem").textContent = s.error ?? "";

  const agents = $("agent");
  if (agents.options.length !== s.agents.list.length) agents.replaceChildren(...s.agents.list.map((a) => el("option", { value: a.id, textContent: a.label })));
  if (document.activeElement !== agents) agents.value = s.agents.default;

  const t = s.target;
  const pj = s.pending?.kind === "junction" ? s.pending : null; // a proposed junction
  const ctx = $("context");
  ctx.replaceChildren();
  if (!s.doc) ctx.append(el("p", { className: "muted" }, "Waiting for VectorCraft…"));
  else if (!t) ctx.append(el("strong", {}, "Nothing selected"), el("p", { className: "muted" }, "Draw a line in VectorCraft and select it, or select a road, then describe it here."));
  else if (t.kind === "line") ctx.append(el("strong", {}, "A plain line is selected"), el("p", { className: "muted" }, s.mode === "water" ? "Describe the water below: a closed shape is a lake or pond, an open line a creek, river or shore." : s.mode === "connection" ? "Describe the junction below, or just carry the road across." : "Describe it below to turn it into a road."));
  else if (t.kind === "road") ctx.append(el("strong", {}, `Road ${s.road.segment}`), el("p", { className: "muted" }, s.road.described ? `${s.road.pieces.selected} of ${s.road.pieces.total} piece${s.road.pieces.total === 1 ? "" : "s"} selected` : "No description yet (drawn as two lanes)"));
  else if (t.kind === "junction") ctx.append(el("strong", {}, `Junction ${s.junction.id}`), el("p", { className: "muted" }, s.junction.described ? "A connecting line with a description" : "A connecting line, not described yet"));
  else if (t.kind === "water") ctx.append(el("strong", {}, `Water ${t.segment}`), el("p", { className: "muted" }, s.water?.described ? "A water feature. Change it below, then Draw." : "Choose what it is, then Draw."));
  else if (t.kind === "wet") ctx.append(el("strong", {}, `Road ${t.road} meets water ${t.water}`), el("p", { className: "muted" }, s.wet?.described ? "Change how the road crosses it, then Draw." : "Choose how the road crosses the water, then Draw."));
  else if (t.kind === "hub") ctx.append(el("strong", {}, `${t.roads.length} roads meet: ${t.roads.join(", ")}`), el("p", { className: "muted" }, s.hub?.described ? "They form an intersection. Change it below, then Draw." : "Choose what happens where they meet, then Draw."));
  else if (t.kind === "crossing") ctx.append(el("strong", {}, `Roads ${t.roads[0]} and ${t.roads[1]} cross`), el("p", { className: "muted" }, s.crossing?.described ? "They form an intersection. Change it below, then Draw." : "Choose what happens where they cross, then Draw."));
  else if (t.kind === "track") ctx.append(el("strong", {}, `Track ${t.id}`), el("p", { className: "muted" }, s.track?.described ? "Railway track. Change it below, or describe the change in words." : "Not described yet: one track. Change it below or describe it in words."));
  else if (t.kind === "level") ctx.append(el("strong", {}, `Road ${t.road} crosses track ${t.track}`), el("p", { className: "muted" }, s.level?.described ? "A level crossing. Change it below." : "A road and a track are selected: make it a level crossing (the road on a panel, the rails across)."));
  else if (t.kind === "joint") ctx.append(el("strong", {}, `Lines ${t.lines[0]} and ${t.lines[1]}`), el("p", { className: "muted" }, s.joint?.described ? "These two lines are joined. Change it below, or describe the change in words." : "Two track lines are selected: say how they meet."));
  else ctx.append(el("strong", {}, "Select one line or one road"), el("p", { className: "muted" }, "The selection holds something this panel does not describe."));
  if (t && (t.kind === "road" || t.kind === "junction" || t.kind === "track") && !s.busy) {
    const b = el("button", { className: "plain", title: "Select the road's own line in VectorCraft, to move or reshape it" }, "Select the line");
    b.onclick = () => act("selectline");
    ctx.append(el("div", { className: "row" }, b, el("span", { className: "muted" }, `Click anywhere on a ${t.kind === "track" ? "track" : "road"} to select it here.`)));
  }

  // A plain line can be a road or a connection between two roads.
  $("modeCard").hidden = t?.kind !== "line";
  for (const r of document.querySelectorAll("input[name=mode]")) r.checked = r.value === s.mode;
  $("matchRow").hidden = s.mode !== "connection";

  // Crossing card: the proposed intersection, or the stored one, or the starting suggestion for two selected roads.
  const pc = s.pending?.kind === "crossing" ? s.pending : null;
  const cv = t?.kind === "crossing" && s.crossing ? { spec: pc ? pc.crossing : s.crossing.spec, summary: pc ? pc.summary : s.crossing.summary, proposed: !!pc } : null;
  $("crossingCard").hidden = !cv;
  if (cv) {
    const [ra, rb] = s.crossing.roads;
    const sp = cv.spec;
    const tr = sp.treatment;
    $("ctitle").textContent = cv.proposed ? "Proposed intersection" : s.crossing.described ? "Intersection" : "Make an intersection";
    $("casked").textContent = pc?.description ? `You asked: “${pc.description}”` : "";
    $("casked").hidden = !pc?.description;
    $("csummary").textContent = cv.summary;
    fillList("cchanges", pc ? pc.changes : null);
    $("ctreat").value = tr;
    const atGrade = ["signal", "two-way-stop", "four-way-stop", "uncontrolled", "driveway", "on-ramp", "off-ramp"].includes(tr);
    const pick = (id, blank) => {
      const sel = $(id);
      if (sel.options.length !== 3 || sel.options[1].value !== ra || sel.options[2].value !== rb) sel.replaceChildren(el("option", { value: "", textContent: blank }), el("option", { value: ra, textContent: `Road ${ra}` }), el("option", { value: rb, textContent: `Road ${rb}` }));
      return sel;
    };
    pick("cminor", "the narrower road").value = sp.minor ?? "";
    pick("cover", `Road ${ra}`).value = sp.over ?? "";
    pick("cmain", "worked out for me").value = sp.main ?? "";
    const only = (id, on) => ($(id).hidden = !on);
    only("cminorRow", tr === "two-way-stop");
    only("coverRow", tr === "overpass" || tr === "cloverleaf");
    only("cmainRow", ["median-opening", "midblock-crossing", "railroad", "on-ramp", "off-ramp"].includes(tr));
    only("cdiaRow", tr === "roundabout");
    only("copenRow", tr === "median-opening");
    only("caccelRow", tr === "on-ramp" || tr === "off-ramp");
    only("ccrossRow", atGrade && !["driveway", "on-ramp", "off-ramp"].includes(tr) || tr === "roundabout");
    only("cbeaconRow", tr === "midblock-crossing");
    only("cgoreRow", tr === "on-ramp" || tr === "off-ramp");
    $("cgore").checked = sp.gore !== false;
    only("cgatesRow", tr === "railroad");
    only("cslipRow", atGrade && !["driveway", "on-ramp", "off-ramp"].includes(tr));
    only("cpocketRow", atGrade && !["driveway", "on-ramp", "off-ramp"].includes(tr));
    only("cstubRow", atGrade);
    $("ccross").checked = !!sp.crosswalks;
    $("cbeacon").checked = !!sp.beacon;
    $("cgates").checked = sp.gates !== false;
    $("cslip").checked = !!sp.slip;
    $("cpocket").value = sp.pocket ?? "none";
    $("cpocketLenRow").hidden = (sp.pocket ?? "none") === "none";
    $("cstub").value = sp.stub ?? "keep";
    for (const [id, v, d] of [["cdia", sp.diameter, 40], ["copen", sp.opening, ""], ["caccel", sp.accel_length, 120], ["cpocketlen", sp.pocket_length, 30]]) if (document.activeElement !== $(id)) $(id).value = v ?? d;
    $("ccancel").hidden = !cv.proposed;
  }

  // Lane-change card: on a selected line of a described road. The stored lane changes, and a form to describe a new one.
  const tcard = t?.kind === "road" && s.road?.described && s.road.piece;
  $("transitionCard").hidden = !tcard;
  if (tcard) {
    const list = $("tlist");
    list.replaceChildren(...(s.road.transitions ?? []).map((e, i) => {
      const rm = el("button", { className: "plain tiny", title: "Take this lane change away" }, "✕");
      rm.onclick = () => act("removetransition", { index: i });
      return el("li", { className: "legrow" }, el("span", { className: "name" }, e.summary || "lane change"), el("code", {}, e.after || ""), rm);
    }));
    const open = s.mode === "transition";
    $("topen").hidden = open;
    $("tform").hidden = !open;
    if (open) {
      const tr = pt?.transition ?? {};
      $("tasked").textContent = pt?.description ? `You asked: “${pt.description}”` : "";
      $("tasked").hidden = !pt?.description;
      $("tchange").value = tr.change ?? "add";
      $("tside").value = tr.side ?? "right";
      $("tdir").value = tr.direction ?? "fwd";
      $("tcount").value = String(tr.count ?? 1);
      for (const [id, v] of [["tat", tr.at], ["tlen", tr.length], ["tspeed", tr.speed ?? 35]]) if (document.activeElement !== $(id)) $(id).value = v ?? "";
      $("tsummary").textContent = pt ? `${pt.summary}. After it the road is: ${pt.after}` : "Choose what changes, or describe it in the box below.";
      fillList("tchanges", null);
    }
  }

  // Hub card: three or more roads that meet at one point.
  const ph = s.pending?.kind === "hub" ? s.pending : null;
  const hv = t?.kind === "hub" && s.hub ? { spec: ph ? ph.hub : s.hub.spec, summary: ph ? ph.summary : s.hub.summary, proposed: !!ph } : null;
  $("hubCard").hidden = !hv;
  if (hv) {
    $("htitle").textContent = hv.proposed ? "Proposed intersection" : s.hub.described ? "Roads meeting at a point" : "Make an intersection of these roads";
    $("hasked").textContent = ph?.description ? `You asked: “${ph.description}”` : "";
    $("hasked").hidden = !ph?.description;
    $("hsummary").textContent = hv.summary;
    fillList("hchanges", ph ? ph.changes : null);
    $("htreat").value = hv.spec.treatment;
    $("hcross").checked = !!hv.spec.crosswalks;
    $("hstub").value = hv.spec.stub ?? "keep";
    $("hcancel").hidden = !hv.proposed;
  }

  // Water card: a selected water line or shape, or a plain line being made into water.
  const pw = s.pending?.kind === "water" ? s.pending : null;
  const wv = s.water ? { spec: pw ? pw.water : s.water.spec, summary: pw ? pw.summary : s.water.summary, proposed: !!pw } : null;
  $("waterCard").hidden = !wv;
  if (wv) {
    const sp = wv.spec;
    $("wtitle").textContent = wv.proposed ? "Proposed water" : s.water.described ? `Water ${s.water.id}` : "Make this water";
    $("wasked").textContent = pw?.description ? `You asked: “${pw.description}”` : "";
    $("wasked").hidden = !pw?.description;
    $("wsummary").textContent = wv.summary;
    fillList("wchanges", pw ? pw.changes : null);
    $("wkind").value = sp.kind ?? "auto";
    const k = sp.kind ?? "auto";
    $("wwidthRow").hidden = $("wflowRow").hidden = $("wspeedRow").hidden = !(k === "flowing" || k === "auto");
    $("wsideRow").hidden = k !== "shore" && k !== "auto";
    $("wbrRow").hidden = k === "shore";
    $("wflow").value = sp.flow ?? "fwd";
    $("wside").value = sp.side ?? "right";
    for (const [id, v] of [["wname", sp.name ?? ""], ["wwidth", sp.width ?? 8], ["wspeed", sp.speed ?? ""], ["wbl", sp.bank_left ?? 0], ["wbr", sp.bank_right ?? 0], ["wslope", sp.slope ?? 2]]) if (document.activeElement !== $(id)) $(id).value = v;
    $("wcancel").hidden = !wv.proposed;
  }

  // Road-meets-water card: one road line and one water line selected.
  const px = s.pending?.kind === "wet" ? s.pending : null;
  const xv = t?.kind === "wet" && s.wet ? { spec: px ? px.wet : s.wet.spec, summary: px ? px.summary : s.wet.summary, proposed: !!px } : null;
  $("wetCard").hidden = !xv;
  if (xv) {
    const sp = xv.spec;
    $("xtitle").textContent = xv.proposed ? "Proposed crossing" : s.wet.described ? "Road meets water" : "How does the road cross the water?";
    $("xasked").textContent = px?.description ? `You asked: “${px.description}”` : "";
    $("xasked").hidden = !px?.description;
    $("xsummary").textContent = xv.summary;
    fillList("xchanges", px ? px.changes : null);
    $("xtreat").value = sp.treatment ?? "bridge";
    const tr = sp.treatment ?? "bridge";
    const only = (id, on) => ($(id).hidden = !on);
    only("xdeckRow", tr === "bridge");
    only("xabutRow", tr === "bridge");
    for (const id of ["xshapeRow", "xcountRow", "xsizeRow"]) only(id, tr === "culvert");
    only("xfillRow", tr !== "bridge");
    only("xslopeRow", tr !== "bridge");
    $("xshape").value = sp.shape ?? "pipe";
    for (const [id, v] of [["xdeck", sp.deck_height ?? 3], ["xabut", sp.abutment ?? 4], ["xcount", sp.count ?? 2], ["xsize", sp.size ?? 1.2], ["xfill", sp.fill_height ?? 2], ["xslope", sp.slope ?? 2]]) if (document.activeElement !== $(id)) $(id).value = v;
    $("xcancel").hidden = !xv.proposed;
  }

  // Embankment card: on a selected line of a described road, the stored fills and a form for a new one.
  const ecard = t?.kind === "road" && s.road?.described && s.road.piece;
  $("embankCard").hidden = !ecard;
  if (ecard) {
    $("elist").replaceChildren(...(s.embank?.list ?? []).map((e, i) => {
      const rm = el("button", { className: "plain tiny", title: "Take this embankment away" }, "✕");
      rm.onclick = () => act("removeembank", { index: i });
      return el("li", { className: "legrow" }, el("span", { className: "name" }, e.summary || "embankment"), rm);
    }));
    const open = s.mode === "fill";
    $("eopen").hidden = open;
    $("eform").hidden = !open;
    if (open) {
      const pe = s.pending?.kind === "embank" ? s.pending : null;
      const ev = pe?.embank ?? {};
      $("easked").textContent = pe?.description ? `You asked: “${pe.description}”` : "";
      $("easked").hidden = !pe?.description;
      $("esides").value = ev.sides ?? "both";
      for (const [id, v] of [["ehgt", ev.height ?? 2], ["eslope", ev.slope ?? 2], ["efrom", ev.from ?? ""], ["eto", ev.to ?? ""], ["eramp", ev.ramp ?? 15]]) if (document.activeElement !== $(id)) $(id).value = v;
      $("esummary").textContent = pe ? pe.summary : "Choose the height and the stretch, or describe it in the box below, then Draw.";
    }
  }

  // Junction card: the proposed junction, or the selected junction's description.
  const jv = pj ? { spec: pj.junction, summary: pj.summary, proposed: true } : t?.kind === "junction" && s.junction ? { spec: s.junction.spec, summary: s.junction.summary } : null;
  $("junctionCard").hidden = !jv;
  if (jv) {
    $("jtitle").textContent = jv.proposed ? "Proposed junction" : `Junction ${s.junction.id}`;
    $("jasked").textContent = pj?.description ? `You asked: “${pj.description}”` : "";
    $("jasked").hidden = !pj?.description;
    $("jsummary").textContent = jv.summary;
    fillList("jchanges", pj ? pj.changes : null);
    const legs = $("jlegs");
    legs.replaceChildren();
    jv.spec.legs.forEach((l, i) => {
      const rm = el("button", { className: "plain tiny", title: "Remove this leg" }, "✕");
      rm.onclick = () => act("junction", { patch: { removeLeg: i } });
      legs.append(el("li", { className: "legrow" }, el("span", { className: "name" }, `${l.side} leg`), el("code", {}, `${l.cross_section} · ${l.length} m`), rm));
    });
    $("jcontrol").value = jv.spec.control;
    $("jcross").checked = !!jv.spec.crosswalks;
    $("jcancel").hidden = !jv.proposed;
  }

  $("road").hidden = !s.road || !!pj || !!cv || !!hv;
  const pk = ["water", "wet", "embank", "tracks", "yard", "joint", "level"].includes(s.pending?.kind); // a proposed water feature, crossing, embankment, track or joint
  if (s.road && !pj) drawItems($("items"), $("bar"), $("total"), s.road, !s.pending);

  const pt = s.pending?.kind === "transition" ? s.pending : null; // a proposed lane change
  const pr = s.pending && !pj && !pc && !ph && !pt && !pk ? s.pending : null; // a proposed road
  $("proposal").hidden = !pr;
  fillList("pchanges", pr ? pr.changes : null);
  $("asked").textContent = pr?.description ? `You asked: “${pr.description}”` : "";
  $("asked").hidden = !pr?.description;
  if (pr) {
    drawItems($("pitems"), $("pbar"), $("ptotal"), pr, true);
    $("shorthand").value = pr.canonical;
  } else if (s.road && document.activeElement !== $("shorthand")) $("shorthand").value = s.road.text;

  // Track, yard and joint cards (built from the CLI's schemas). A proposal is shown in the card and in "what changes".
  const onTrack = s.track && (t?.kind === "track" || (t?.kind === "line" && s.mode === "track"));
  $("trackCard").hidden = !onTrack;
  if (onTrack) {
    const pt2 = ["tracks", "yard"].includes(s.pending?.kind) ? s.pending : null;
    const tr = pt2 ?? s.track;
    $("tsummary").textContent = pt2 ? `Proposed: ${pt2.summary}` : s.track.described ? s.track.summary : "Not described yet: one track.";
    const needs = tr.kind === "yard" && tr.min_line_m ? `This yard needs at least ${tr.min_line_m} m of line.` : "";
    $("tneeds").textContent = needs;
    $("tneeds").hidden = !needs;
    fillList("tchanges", pt2?.changes);
    showCard("t", $("tfields"), s.track.card, pt2?.spec ?? null, (patch) => act("track", { patch }));
    $("tcancel").hidden = !pt2;
  }
  $("levelCard").hidden = !(s.level && t?.kind === "level");
  if (s.level && t?.kind === "level") {
    const pl = s.pending?.kind === "level" ? s.pending : null;
    $("lsummary").textContent = pl ? `Proposed: ${pl.summary}` : s.level.described ? `Stored: ${s.level.spec.treatment === "gated" ? "a gated level crossing" : "a level crossing"}` : "Not stored yet: a level crossing.";
    fillList("lchanges", pl?.changes);
    showCard("l", $("lfields"), s.level.card, pl?.spec ?? null, (patch) => act("level", { patch }));
    $("lcancel").hidden = !pl;
  }
  $("jointCard").hidden = !(s.joint && t?.kind === "joint");
  if (s.joint && t?.kind === "joint") {
    const pj2 = s.pending?.kind === "joint" ? s.pending : null;
    $("jfound").textContent = s.joint.found?.ok ? `From the drawing: ${s.joint.found.summary}` : s.joint.found?.error ?? "";
    $("jsummary").textContent = pj2 ? `Proposed: ${pj2.summary}` : s.joint.described ? s.joint.summary : "Not described yet.";
    fillList("jchanges", pj2?.changes);
    showCard("j", $("jfields"), s.joint.card, pj2?.spec ?? null, (patch) => act("joint", { patch }));
    $("jcancel2").hidden = !pj2;
  }

  const scope = $("scope");
  const multi = s.road && !pj && s.road.pieces.total > s.road.pieces.selected;
  scope.hidden = !multi;
  if (multi) $("scope-all").textContent = `All ${s.road.pieces.total} pieces of this road`;

  const wordsFor = t?.kind === "crossing" ? "Describe the intersection, or the change you want (lighted, stop signs, left turn pockets, driveway, ramp, railroad, roundabout …)" : t?.kind === "hub" ? "Describe the intersection, or the change you want (lighted, all-way stop, crosswalks …)" : t?.kind === "water" || (t?.kind === "line" && s.mode === "water") ? "Describe the water (a creek 6 m wide with steep banks, ocean on the right, a pond …)" : t?.kind === "joint" ? "Describe how these two lines meet (they cross; b is a siding off a; they join end to end)" : t?.kind === "track" || (t?.kind === "line" && s.mode === "track") ? "Describe the track (double track; a yard with 8 spurs on the left and 4 on the right, merging back; a crossover …)" : t?.kind === "wet" ? "Describe how the road crosses the water (bridge with 8 m clearance, three 1.5 m culverts, causeway …)" : t?.kind === "road" && s.mode === "fill" ? "Describe the embankment (4 m of fill between 200 and 450 m, 3:1 slopes …)" : null;
  const junctionish = t?.kind === "junction" || (t?.kind === "line" && s.mode === "connection");
  $("wordsLabel").textContent = (t?.kind === "road" && s.mode === "transition" ? "Describe the lane change (for example: the right lane ends after 300 m)" : null) ?? wordsFor ?? (junctionish ? "Describe the junction, or the change you want" : "Describe the road, or the change you want");
  const canAct = !!t && t.kind !== "other" && !s.busy;
  $("propose").disabled = !canAct || !$("words").value.trim();
  $("draw").disabled = !canAct;
  $("draw").textContent = s.busy === "drawing" ? "Drawing…" : s.pending ? "Draw this" : (t?.kind === "crossing" && !s.crossing?.described) || (t?.kind === "hub" && !s.hub?.described) ? "Draw this intersection" : (t?.kind === "water" && !s.water?.described) || (t?.kind === "line" && s.mode === "water") || (t?.kind === "wet" && !s.wet?.described) || (t?.kind === "line" && s.mode === "track") || (t?.kind === "track" && !s.track?.described) || (t?.kind === "joint" && !s.joint?.described) || (t?.kind === "level" && !s.level?.described) ? "Draw this" : "Redraw";
  $("thinking").hidden = s.busy !== "thinking";
  const msg = $("message");
  msg.textContent = s.message ?? "";
  msg.classList.toggle("err", !!s.messageIsError || /could not|cannot|unknown|select /i.test(s.message ?? ""));
  showSettings(s);
}

$("propose").onclick = () => act("propose", { description: $("words").value.trim(), agent: $("agent").value });
$("words").oninput = () => state && show(state);
$("words").onkeydown = (e) => { if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && !$("propose").disabled) $("propose").click(); };
$("draw").onclick = () => act("draw", { scope: document.querySelector("input[name=scope]:checked")?.value });
$("cancel").onclick = () => act("cancel");
$("jcancel").onclick = () => act("cancel");
$("tcancel").onclick = () => act("cancel");
$("lcancel").onclick = () => act("cancel");
$("jcancel2").onclick = () => act("cancel");
$("ccancel").onclick = () => act("cancel");
const cpatch = (patch) => act("crossing", { patch });
$("ctreat").onchange = () => cpatch({ treatment: $("ctreat").value });
$("ccross").onchange = () => cpatch({ crosswalks: $("ccross").checked });
$("cminor").onchange = () => cpatch({ minor: $("cminor").value || null });
$("cover").onchange = () => cpatch({ over: $("cover").value || null });
$("cstub").onchange = () => cpatch({ stub: $("cstub").value });
$("cmain").onchange = () => cpatch({ main: $("cmain").value || null });
$("cbeacon").onchange = () => cpatch({ beacon: $("cbeacon").checked });
$("cgates").onchange = () => cpatch({ gates: $("cgates").checked });
$("cslip").onchange = () => cpatch({ slip: $("cslip").checked });
$("cgore").onchange = () => cpatch({ gore: $("cgore").checked });
$("cpocket").onchange = () => cpatch({ pocket: $("cpocket").value });
$("cpocketlen").onchange = () => cpatch({ pocket_length: Number($("cpocketlen").value) || 30 });
$("copen").onchange = () => cpatch({ opening: Number($("copen").value) || null });
$("caccel").onchange = () => cpatch({ accel_length: Number($("caccel").value) || 120 });
const hpatch = (patch) => act("hub", { patch });
$("htreat").onchange = () => hpatch({ treatment: $("htreat").value });
$("hcross").onchange = () => hpatch({ crosswalks: $("hcross").checked });
$("hstub").onchange = () => hpatch({ stub: $("hstub").value });
$("hcancel").onclick = () => act("cancel");
const num = (id) => ($(id).value === "" ? null : Number($(id).value));
const wpatch = () => act("water", { patch: { kind: $("wkind").value, name: $("wname").value || null, width: num("wwidth"), flow: $("wflow").value, speed: num("wspeed"), side: $("wside").value, bank_left: num("wbl"), bank_right: num("wbr"), slope: num("wslope") } });
for (const id of ["wkind", "wname", "wwidth", "wflow", "wspeed", "wside", "wbl", "wbr", "wslope"]) $(id).onchange = wpatch;
$("wcancel").onclick = () => act("cancel");
const xpatch = () => act("wet", { patch: { treatment: $("xtreat").value, deck_height: num("xdeck"), abutment: num("xabut"), shape: $("xshape").value, count: num("xcount"), size: num("xsize"), fill_height: num("xfill"), slope: num("xslope") } });
for (const id of ["xtreat", "xdeck", "xabut", "xshape", "xcount", "xsize", "xfill", "xslope"]) $(id).onchange = xpatch;
$("xcancel").onclick = () => act("cancel");
$("eopen").onclick = () => act("mode", { mode: "fill" });
$("edone").onclick = () => act("mode", { mode: "road" });
const epatch = () => act("embank", { patch: { height: num("ehgt"), slope: num("eslope"), from: num("efrom"), to: num("eto"), sides: $("esides").value, ramp: num("eramp") } });
for (const id of ["ehgt", "eslope", "efrom", "eto", "esides", "eramp"]) $(id).onchange = epatch;
$("topen").onclick = () => act("mode", { mode: "transition" });
$("tdone").onclick = () => act("mode", { mode: "road" });
const tpatch = () => act("transition", { patch: { change: $("tchange").value, side: $("tside").value, direction: $("tdir").value, count: Number($("tcount").value), at: $("tat").value === "" ? null : Number($("tat").value), length: $("tlen").value === "" ? null : Number($("tlen").value), speed: Number($("tspeed").value) || 35 } });
for (const id of ["tchange", "tside", "tdir", "tcount", "tat", "tlen", "tspeed"]) $(id).onchange = tpatch;
$("cdia").onchange = () => cpatch({ diameter: Number($("cdia").value) || 40 });
$("match").onclick = () => act("match");
for (const r of document.querySelectorAll("input[name=mode]")) r.onchange = () => act("mode", { mode: r.value });
$("jcontrol").onchange = () => act("junction", { patch: { control: $("jcontrol").value } });
$("jcross").onchange = () => act("junction", { patch: { crosswalks: $("jcross").checked } });
$("use").onclick = () => act("text", { text: $("shorthand").value });
$("agent").onchange = () => act("agent", { id: $("agent").value });

const events = new EventSource("/events");
events.onmessage = (m) => show(JSON.parse(m.data));
events.onerror = () => $("dot").classList.remove("on");

// ---- Document settings: a card built from the CLI's schema, read only while it is open.
let settingsBuilt = "";
function fieldControl(f, send, id) {
  if (f.type === "choice") {
    const s = el("select", { id }, ...f.options.map((o) => el("option", { value: o, textContent: o })));
    s.onchange = () => send(s.value);
    return s;
  }
  const numeric = f.type === "integer" || f.type === "number";
  const i = el("input", { id, type: numeric ? "number" : "text" });
  if (f.min !== undefined) i.min = String(f.min);
  if (f.max !== undefined) i.max = String(f.max);
  if (f.type === "integer") i.step = "1";
  // An empty number is sent as null, which the panel leaves out of the description.
  i.onchange = () => send(numeric ? (i.value === "" ? null : Number(i.value)) : i.value);
  return i;
}
const settingControl = (f) => fieldControl(f, (v) => act("settings", { patch: { [f.key]: v } }), `f_${f.key}`);

// A card built from a schema's fields: made once per set of fields, then only the values are updated. `spec` is a proposal (its missing fields are empty),
// or null to show the stored values from the card model.
const cardBuilt = {};
function showCard(prefix, container, card, spec, send) {
  const keys = card.fields.map((f) => f.key).join(",");
  if (cardBuilt[prefix] !== keys) {
    cardBuilt[prefix] = keys;
    container.replaceChildren(...card.fields.map((f) => el("div", { className: "field" }, el("label", { htmlFor: `${prefix}_${f.key}`, textContent: f.label }), fieldControl(f, (v) => send({ [f.key]: v }), `${prefix}_${f.key}`), el("p", { className: "help", textContent: f.help }))));
  }
  for (const f of card.fields) {
    const c = $(`${prefix}_${f.key}`);
    if (!c || document.activeElement === c) continue;
    const v = spec ? spec[f.key] : f.value;
    c.value = String(v === true ? "on" : v === false ? "off" : v ?? "");
  }
}

function showSettings(s) {
  const sc = s.settings;
  $("swait").hidden = !!sc || !$("settingsCard").open;
  $("sproblem").hidden = !sc?.problem;
  $("sproblem").textContent = sc?.problem ?? "";
  if (sc) {
    const fields = sc.card.fields;
    const keys = fields.map((f) => f.key).join(",");
    if (keys !== settingsBuilt) {
      settingsBuilt = keys;
      $("sfields").replaceChildren(...fields.map((f) => el("div", { className: "field" }, el("label", { htmlFor: `f_${f.key}`, textContent: f.label }), settingControl(f), el("p", { className: "help", textContent: f.help }))));
    }
    const values = s.settingsPending?.settings ?? sc.values;
    for (const f of fields) {
      const c = $(`f_${f.key}`);
      if (c && document.activeElement !== c) c.value = String(values[f.key] ?? f.value);
    }
    $("ssummary").textContent = s.settingsPending ? `Proposed: ${s.settingsPending.summary}` : sc.summary;
  } else {
    $("sfields").replaceChildren();
    settingsBuilt = "";
    $("ssummary").textContent = "";
  }
  $("sdraw").disabled = !s.settingsPending || !!s.busy;
  $("scancel").hidden = !s.settingsPending;
}
$("settingsCard").ontoggle = () => act($("settingsCard").open ? "opensettings" : "closesettings");
$("sdraw").onclick = () => act("drawsettings");
$("scancel").onclick = () => act("cancelsettings");
