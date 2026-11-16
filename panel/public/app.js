// The panel page. Everything the server sends is shown with textContent or as a form value, never as HTML.
const $ = (id) => document.getElementById(id);
let state = null;
let built = ""; // the field keys the form was built for

async function post(action, body = {}) {
  const res = await fetch(`/api/${action}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  if (!res.ok) throw new Error((await res.json().catch(() => ({}))).error ?? res.statusText);
}
const act = (action, body) => post(action, body).catch((e) => show({ ...state, message: e.message, messageIsError: true }));

function el(tag, props = {}, ...kids) {
  const e = Object.assign(document.createElement(tag), props);
  for (const k of kids) e.append(k);
  return e;
}

/** One control per schema field; a change sends that one value. */
function control(f) {
  const send = (v) => act("settings", { patch: { [f.key]: v } });
  if (f.type === "choice") {
    const s = el("select", { id: `f_${f.key}` }, ...f.options.map((o) => el("option", { value: o, textContent: o })));
    s.onchange = () => send(s.value);
    return s;
  }
  const numeric = f.type === "integer" || f.type === "number";
  const i = el("input", { id: `f_${f.key}`, type: numeric ? "number" : "text" });
  if (f.min !== undefined) i.min = String(f.min);
  if (f.max !== undefined) i.max = String(f.max);
  if (f.type === "integer") i.step = "1";
  i.onchange = () => send(numeric ? Number(i.value) : i.value);
  return i;
}

function show(s) {
  state = s;
  $("dot").classList.toggle("on", !!s.connected);
  $("problem").hidden = !s.error;
  $("problem").textContent = s.error ?? "";
  $("settingsCard").hidden = !s.settings;
  if (s.settings) {
    const fields = s.settings.card.fields;
    const keys = fields.map((f) => f.key).join(",");
    if (keys !== built) {
      built = keys;
      $("fields").replaceChildren(...fields.map((f) => el("div", { className: "field" }, el("label", { htmlFor: `f_${f.key}`, textContent: f.label }), control(f), el("p", { className: "help", textContent: f.help }))));
    }
    const values = s.pending?.settings ?? s.settings.values;
    for (const f of fields) {
      const c = $(`f_${f.key}`);
      if (c && document.activeElement !== c) c.value = String(values[f.key] ?? f.value);
    }
    $("stitle").textContent = s.pending ? "Proposed change" : "Document settings";
    $("ssummary").textContent = s.pending ? s.pending.summary : s.settings.summary;
  }
  $("draw").disabled = !s.pending || !!s.busy;
  $("draw").textContent = s.busy === "drawing" ? "Drawing…" : "Draw this";
  $("cancel").hidden = !s.pending;
  const msg = $("message");
  msg.textContent = s.message ?? "";
  msg.classList.toggle("err", !!s.messageIsError || /could not|cannot|must be|not valid/i.test(s.message ?? ""));
}

$("draw").onclick = () => act("draw");
$("cancel").onclick = () => act("cancel");

const events = new EventSource("/events");
events.onmessage = (m) => show(JSON.parse(m.data));
events.onerror = () => $("dot").classList.remove("on");
