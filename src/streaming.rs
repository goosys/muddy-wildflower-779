//! Keep the isolated Tokio runtime alive until an SSE response finishes.
//!
//! The pinned SDK drops its fetch runtime when headers are returned, and its
//! wasm-streams adapter refers to an unexported JS class on Emscripten. Pump into
//! a native `TransformStream` from a separate isolated runtime to avoid both.

use axum::{body::Body, http::StatusCode, response::Response};
use futures_util::StreamExt;
use haikunator_worker::controllers::mcp::MAX_REQUEST_BYTES;
use tower::ServiceExt;
use worker::{
    js_sys::{Array, Function, Promise, Uint8Array},
    wasm_bindgen::{closure::Closure, JsCast, JsValue},
    wasm_bindgen_futures::{tokio::schedule_isolated, JsFuture},
    web_sys::{TransformStream, WritableStreamDefaultWriter},
    AbortSignal, HttpRequest,
};

#[worker::send]
pub async fn respond(req: HttpRequest) -> worker::Result<worker::Response> {
    // JS promises cross the host-loop boundary; cross-runtime Tokio oneshots lose
    // wakes in the pinned hosted scheduler when another runtime is being driven.
    let (head, resolve_head, reject_head) = deferred();
    // Start outside the SDK's outer runtime so the new event loop gets its first drive.
    worker::wasm_bindgen_futures::spawn_local(async move {
        schedule_isolated(
            async move {
                let (disconnected, _listener) = listen_for_disconnect(&req)?;
                let (response, pump) = tokio::select! {
                    biased;
                    _ = JsFuture::from(disconnected.clone()) => {
                        let response: worker::web_sys::Response =
                            worker::Response::empty()?.with_status(499).into();
                        resolve_head.call1(&JsValue::NULL, &response.into())?;
                        return Ok(());
                    },
                    prepared = async {
                        prepare(route_buffered_request(req).await?).await
                    } => prepared?,
                };
                // Return headers before writing: writes wait for a client reader.
                let response: worker::web_sys::Response = response.into();
                resolve_head.call1(&JsValue::NULL, &response.into())?;
                if let Some((body, writer)) = pump {
                    pump_body(body, writer, disconnected).await?;
                }
                Ok::<(), worker::Error>(())
            },
            move |result| {
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(err)) => Some(err.to_string()),
                    Err(err) => Some(err.to_string()),
                };
                if let Some(error) = error {
                    worker::console_error!("MCP response failed: {error}");
                    let _ = reject_head.call1(&JsValue::NULL, &JsValue::from_str(&error));
                }
            },
        );
    });
    let response = JsFuture::from(head)
        .await?
        .dyn_into::<worker::web_sys::Response>()?;
    Ok(worker::Response::from(response))
}

fn deferred() -> (Promise, Function, Function) {
    let mut callbacks = None;
    let promise = Promise::new(&mut |resolve, reject| callbacks = Some((resolve, reject)));
    let (resolve, reject) =
        callbacks.expect("Promise constructor calls its executor synchronously");
    (promise, resolve, reject)
}

async fn route_buffered_request(req: HttpRequest) -> worker::Result<Response> {
    let (parts, body) = req.into_parts();
    let mut incoming = Body::new(body).into_data_stream();
    let mut bytes = Vec::new();
    let mut oversized = false;
    // Worker Body cancels the incoming JS reader on drop. Drain to EOF so a
    // rejected body does not signal a disconnected client before we can send 413.
    // Excess bytes are discarded, keeping the retained input bounded at 16 KiB.
    while let Some(chunk) = incoming.next().await {
        let chunk = chunk.map_err(|err| worker::Error::RustError(err.to_string()))?;
        if !oversized && bytes.len().saturating_add(chunk.len()) <= MAX_REQUEST_BYTES {
            bytes.extend_from_slice(&chunk);
        } else {
            oversized = true;
            bytes.clear();
        }
    }
    let router = if oversized {
        super::decorate_router(axum::Router::new().fallback(|| async {
            (
                StatusCode::PAYLOAD_TOO_LARGE,
                "MCP request body is too large",
            )
        }))
    } else {
        super::router()
    };
    let request = axum::http::Request::from_parts(parts, Body::from(bytes));
    Ok(match router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    })
}

struct AbortListener {
    signal: Option<AbortSignal>,
    callback: Option<Closure<dyn FnMut()>>,
    resolve: Function,
}

impl Drop for AbortListener {
    fn drop(&mut self) {
        if let (Some(signal), Some(callback)) = (&self.signal, &self.callback) {
            let _ = signal
                .remove_event_listener_with_callback("abort", callback.as_ref().unchecked_ref());
        }
        // Settle any pending JS futures on normal completion as well as abort.
        let _ = self.resolve.call0(&JsValue::NULL);
    }
}

fn listen_for_disconnect(req: &HttpRequest) -> worker::Result<(Promise, AbortListener)> {
    let (promise, resolve, _) = deferred();
    let mut listener = AbortListener {
        signal: None,
        callback: None,
        resolve: resolve.clone(),
    };
    let Some(signal) = req.extensions().get::<AbortSignal>().cloned() else {
        return Ok((promise, listener));
    };
    if signal.aborted() {
        resolve.call0(&JsValue::NULL)?;
        return Ok((promise, listener));
    }
    let callback = Closure::new(move || {
        let _ = resolve.call0(&JsValue::NULL);
    });
    signal.add_event_listener_with_callback("abort", callback.as_ref().unchecked_ref())?;
    listener.signal = Some(signal);
    listener.callback = Some(callback);
    Ok((promise, listener))
}

type Pump = (Body, WritableStreamDefaultWriter);

async fn prepare(response: Response) -> worker::Result<(worker::Response, Option<Pump>)> {
    if !response
        .headers()
        .get("content-type")
        .is_some_and(|value| value.as_bytes().starts_with(b"text/event-stream"))
    {
        return Ok((super::buffered_response(response).await?, None));
    }
    let (parts, body) = response.into_parts();
    let stream = TransformStream::new()?;
    let writer = stream.writable().get_writer()?;
    let response = worker::ResponseBuilder::new()
        .with_status(parts.status.as_u16())
        .with_headers(parts.headers.into())
        .stream(stream.readable());
    Ok((response, Some((body, writer))))
}

#[expect(
    clippy::future_not_send,
    reason = "JS stream futures stay on their isolated single-threaded runtime"
)]
async fn pump_body(
    body: Body,
    writer: WritableStreamDefaultWriter,
    disconnected: Promise,
) -> worker::Result<()> {
    let mut stream = body.into_data_stream();
    // Race in JS as well: on disconnect these promises settle and release the
    // Rust callbacks even if the original native write remains backpressured.
    let until_disconnect =
        |promise: Promise| JsFuture::from(Promise::race(&Array::of2(&promise, &disconnected)));
    let closed = until_disconnect(writer.closed());
    tokio::pin!(closed);
    loop {
        let chunk = tokio::select! {
            biased;
            // This settles on writer closure or request abort, even between items.
            // Drop the body without erroring an already-disconnected response.
            _ = &mut closed => return Ok(()),
            chunk = stream.next() => chunk,
        };
        match chunk {
            Some(Ok(bytes)) => {
                let array = Uint8Array::from(bytes.as_ref());
                tokio::select! {
                    biased;
                    _ = &mut closed => return Ok(()),
                    written = until_disconnect(writer.write_with_chunk(&array)) => {
                        if written.is_err() {
                            return Ok(());
                        }
                    },
                }
            }
            Some(Err(err)) => {
                let reason = worker::js_sys::Error::new(&err.to_string());
                drop(until_disconnect(writer.abort_with_reason(&reason)));
                return Err(worker::Error::RustError(err.to_string()));
            }
            None => {
                tokio::select! {
                    biased;
                    _ = &mut closed => {},
                    _ = until_disconnect(writer.close()) => {},
                }
                return Ok(());
            }
        }
    }
}
