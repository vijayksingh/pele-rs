export const SOURCE = 'xychart\ntitle "Initial title"\nx-axis Month [Jan, Feb, Mar]\ny-axis Revenue -10 --> 20\nline Forecast [5 "PointA", 10 "PointB", 15 "PointC"]\nbar Sales [-5, 10, 15]';
const bits = number => {
  const view = new DataView(new ArrayBuffer(8));
  view.setFloat64(0, number);
  return view.getBigUint64(0).toString(16).padStart(16, '0');
};
export function runNaNCategory(core, entry, dispose, raw) {
  const model = raw ? entry.parse(SOURCE, {}, undefined, 0, {}) : core.parse(SOURCE);
  const axis = model.xAxis, categories = axis.categories, plots = model.plots;
  const config = {xyChart: {showDataLabel: true}};
  const draw = padding => {
    const trace = [];
    const settings = {padding, measurer: {width(text, size, style) {
      trace.push(['width', text, bits(size), style ?? 0]);
      return text.length * size * .6;
    }}};
    try {
      const output = raw ? entry.render(model, config, settings) : core.render(model, {config, ...settings});
      return {svg: output.svg, width: bits(output.width), height: bits(output.height), trace};
    } catch (error) {
      return {error: {name: error?.name, message: error?.message}, trace};
    }
  };
  try {
    const first = draw(NaN), recovery = draw(8);
    return {raw, first, recovery, identity: {
      axis: model.xAxis === axis, categories: model.xAxis.categories === categories, plots: model.plots === plots,
    }};
  } finally { dispose(model); }
}
