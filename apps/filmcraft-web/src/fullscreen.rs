//! Fullscreen belongs to the browser host, not eframe's native viewport command handling.
use std::cell::Cell;

use filmcraft_ui_egui::FilmcraftApp;
use serde_json::json;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};

thread_local! {
    static PENDING: Cell<bool> = const { Cell::new(false) };
}

fn active() -> bool {
    web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.fullscreen_element().is_some())
}

fn report(error: Option<&str>) {
    let supported = web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.fullscreen_enabled());
    crate::set_info("fullscreen", json!({"supported": supported, "active": active(), "pending": PENDING.with(Cell::get), "error": error}));
    crate::repaint();
}

fn message(error: JsValue) -> String {
    js_sys::Reflect::get(&error, &"message".into()).ok().and_then(|v| v.as_string()).or_else(|| error.as_string()).unwrap_or_else(|| format!("{error:?}"))
}

fn request() -> Result<(), String> {
    let result = begin_request();
    if let Err(error) = &result {
        report(Some(error));
    }
    result
}

fn begin_request() -> Result<(), String> {
    if PENDING.with(Cell::get) {
        return Err("A fullscreen transition is already pending".into());
    }
    let document = web_sys::window().and_then(|w| w.document()).ok_or("Fullscreen is unavailable: no document")?;
    if !document.fullscreen_enabled() && !active() {
        return Err("Fullscreen is unavailable in this browser or embedding host".into());
    }
    let (target, method): (JsValue, &str) = if active() {
        (document.into(), "exitFullscreen")
    } else {
        (document.document_element().ok_or("Fullscreen is unavailable: no editor document")?.into(), "requestFullscreen")
    };
    // web-sys's legacy void bindings discard the returned Promise. Preserve it so asynchronous
    // browser denial is reported instead of showing a false active state or an unhandled error.
    let function = js_sys::Reflect::get(&target, &method.into()).map_err(message)?.dyn_into::<js_sys::Function>().map_err(message)?;
    let result = function.call0(&target).map_err(message)?;
    PENDING.with(|pending| pending.set(true));
    report(None);
    spawn_local(async move {
        let result = JsFuture::from(js_sys::Promise::resolve(&result)).await;
        PENDING.with(|pending| pending.set(false));
        match result {
            Ok(_) => report(None),
            Err(error) => {
                let error = format!("Fullscreen: {}", message(error));
                report(Some(&error));
                crate::post(move |web, _| web.app.ui.status = error);
            }
        }
    });
    Ok(())
}

pub fn install(app: &mut FilmcraftApp) -> Result<(), String> {
    app.hooks.toggle_fullscreen = Some(Box::new(request));
    app.hooks.fullscreen_state = Some(Box::new(active));
    let document = web_sys::window().and_then(|w| w.document()).ok_or("Fullscreen is unavailable: no document")?;
    let changed = Closure::<dyn FnMut(web_sys::Event)>::new(|_| report(None));
    document.add_event_listener_with_callback("fullscreenchange", changed.as_ref().unchecked_ref()).map_err(message)?;
    // One listener for this page's lifetime, as for the host's other DOM integrations.
    changed.forget();
    report(None);
    Ok(())
}
