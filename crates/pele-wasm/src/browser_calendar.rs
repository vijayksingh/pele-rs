//! Caught browser calendar imports. Every foreign throw returns through Rust;
//! owned invocation resources additionally return its exact original JS value.
use pele::compat::{CalendarFields, CivilDateTime, PeleError, RenderResources};
use wasm_bindgen::prelude::*;

/// Per-resource error capture: nested resources and nested ordinary renders
/// cannot overwrite this operation's original thrown value.
pub(super) struct Resources<'a> {
    pub(super) browser: &'a mut super::BrowserResources,
    pub(super) error: Option<JsValue>,
}
impl Resources<'_> {
    fn capture<T>(&mut self, value: Result<T, JsValue>) -> Result<T, PeleError> {
        value.map_err(|error| {
            if self.error.is_none() {
                self.error = Some(error.clone());
            }
            PeleError::new(
                "Browser calendar callback failed",
                pele::compat::ErrorCode::Semantic,
            )
        })
    }
}
impl RenderResources for Resources<'_> {
    fn javascript_failure(
        &mut self,
        operation: pele::compat::JavaScriptFailure,
    ) -> pele::compat::public::PublicError {
        self.browser.javascript_failure(operation)
    }
    fn text_width(&mut self, request: pele::compat::TextRequest<'_>) -> Result<f64, PeleError> {
        self.browser.text_width(request)
    }
    fn icon(&mut self, name: &str) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        self.browser.icon(name)
    }
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, PeleError> {
        self.capture(parse_date(input))
    }
    fn calendar_fields(&mut self, instant: f64) -> Result<Option<CalendarFields>, PeleError> {
        self.capture(calendar_fields(instant))
    }
    fn calendar_instant(&mut self, civil: CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.capture(calendar_instant(civil))
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        self.capture(now_millis())
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Date)]
    type CaughtDate;
    #[wasm_bindgen(catch, constructor, js_class = Date)]
    fn new(value: f64) -> Result<CaughtDate, JsValue>;
    #[wasm_bindgen(catch, static_method_of = CaughtDate, js_class = Date, js_name = now)]
    fn now() -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, static_method_of = CaughtDate, js_class = Date, js_name = parse)]
    fn parse(value: &str) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getTime)]
    fn time(this: &CaughtDate) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getFullYear)]
    fn year(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getMonth)]
    fn month(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getDate)]
    fn day(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getHours)]
    fn hour(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getMinutes)]
    fn minute(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getSeconds)]
    fn second(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getMilliseconds)]
    fn millisecond(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getDay)]
    fn weekday(this: &CaughtDate) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, method, js_name = getTimezoneOffset)]
    fn offset(this: &CaughtDate) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, method, js_name = setFullYear)]
    fn set_year(this: &CaughtDate, year: u32) -> Result<f64, JsValue>;
}

pub(super) fn parse_date(input: &str) -> Result<Option<f64>, JsValue> {
    let instant = CaughtDate::parse(input)?;
    Ok(instant.is_finite().then_some(instant))
}
pub(super) fn now_millis() -> Result<f64, JsValue> {
    CaughtDate::now()
}
pub(super) fn calendar_fields(instant: f64) -> Result<Option<CalendarFields>, JsValue> {
    let date = CaughtDate::new(instant)?;
    if !date.time()?.is_finite() {
        return Ok(None);
    }
    Ok(Some(CalendarFields {
        civil: CivilDateTime {
            year: date.year()? as i32,
            month: (date.month()? + 1) as u8,
            day: date.day()? as u8,
            hour: date.hour()? as u8,
            minute: date.minute()? as u8,
            second: date.second()? as u8,
            millisecond: date.millisecond()? as u16,
        },
        weekday: date.weekday()? as u8,
        utc_offset_minutes: -date.offset()? as i32,
    }))
}
pub(super) fn calendar_instant(c: CivilDateTime) -> Result<Option<f64>, JsValue> {
    let year = if (0..100).contains(&c.year) {
        c.year + 400
    } else {
        c.year
    };
    let args = js_sys::Array::new();
    for value in [
        year,
        i32::from(c.month) - 1,
        i32::from(c.day),
        i32::from(c.hour),
        i32::from(c.minute),
        i32::from(c.second),
        i32::from(c.millisecond),
    ] {
        args.push(&JsValue::from_f64(f64::from(value)));
    }
    let constructor = js_sys::Reflect::get(&js_sys::global(), &"Date".into())?
        .unchecked_into::<js_sys::Function>();
    let date = js_sys::Reflect::construct(&constructor, &args)?.unchecked_into::<CaughtDate>();
    if (0..100).contains(&c.year) {
        date.set_year(c.year as u32)?;
    }
    let instant = date.time()?;
    Ok(instant.is_finite().then_some(instant))
}
