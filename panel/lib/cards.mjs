// A card is a form built from a schema (the CLI's `schema <kind>`): no per-kind code in the panel.

/** `{fields: [{key, label, type, value, help, options?, min?, max?}]}` with each value taken from `values`, else the schema's default. */
export function cardModel(schema, values) {
  return {
    fields: schema.fields.map((f) => {
      const value = values?.[f.key] ?? f.default;
      // A true/false setting is a choice between on and off.
      const field = { key: f.key, label: f.label, type: f.type, value: f.options?.[0] === "on" && typeof value === "boolean" ? (value ? "on" : "off") : value, help: f.help };
      if (f.options) field.options = f.options;
      if (f.min !== undefined) field.min = f.min;
      if (f.max !== undefined) field.max = f.max;
      return field;
    }),
  };
}
