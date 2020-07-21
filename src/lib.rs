mod ls;
mod open;
mod random_dice;
mod sys;
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;

use nu_cli::{create_default_context, parse_and_eval, whole_stream_command, EnvironmentSyncer};
use nu_errors::ShellError;

use serde::Serialize;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Serialize)]
enum OkError {
    Ok(String),
    Error(ShellError),
    InternalError(String),
}

#[wasm_bindgen]
pub async fn run_nu(line: String) -> String {
    utils::set_panic_hook();

    let mut syncer = EnvironmentSyncer::new();
    let context = create_default_context(&mut syncer, true);
    match context {
        Ok(mut ctx) => {
            log!("processing line");
            ctx.add_commands(vec![
                whole_stream_command(random_dice::SubCommand),
                whole_stream_command(ls::Ls),
                whole_stream_command(open::Open),
                whole_stream_command(sys::Sys),
            ]);
            match parse_and_eval(&line, &mut ctx).await {
                Ok(val) => match serde_json::to_string(&OkError::Ok(val)) {
                    Ok(output) => output,
                    Err(e) => format!("Error converting to json: {:?}", e),
                },
                Err(e) => match serde_json::to_string(&OkError::Error(e)) {
                    Ok(output) => output,
                    Err(e) => format!("Error converting to json: {:?}", e),
                },
            }
        }
        Err(e) => match serde_json::to_string(&OkError::InternalError(format!("{}", e))) {
            Ok(output) => output,
            Err(e) => format!("Error converting to json: {:?}", e),
        },
    }
}