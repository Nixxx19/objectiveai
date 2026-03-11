const { z, toJSONSchema } = require('zod');
const Inner = z.string().meta({ title: 'inner.Title' });
const Outer = z.object({ x: Inner }).meta({ title: 'outer.Title' });
const allTitles = new Set(['inner.Title', 'outer.Title']);

const result = toJSONSchema(Outer, {
  reused: 'inline',
  override(ctx) {
    const js = ctx.jsonSchema;
    if (!js || typeof js !== 'object') return;
    if ('title' in js && typeof js.title === 'string' && allTitles.has(js.title) && js.title !== 'outer.Title') {
      const title = js.title;
      for (const key of Object.keys(js)) delete js[key];
      js.$ref = title;
    }
  }
});
console.log(JSON.stringify(result, null, 2));
