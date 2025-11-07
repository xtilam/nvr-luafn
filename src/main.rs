use serde_json::{Value as JSONValue, json};
use std::process::Command;
use std::{env, process::exit};

fn main() {
    let nvim_address = env::var("NVIM").unwrap_or_else(|_| {
        println!("NVIM not set, exiting.");
        exit(0);
    });
    let nvim_fn_call = env::var("NVIM_FN_CALL").unwrap_or_else(|_| {
        println!("NVIM_FN_CALL not set, exiting.");
        exit(0);
    });

    #[cfg(debug_assertions)]
    println!("NVIM_LISTEN_ADDRESS: {}", nvim_address);

    let args: Vec<String> = env::args().skip(1).collect();
    let lua_code = format!(
        "({{ pcall(({{pcall(loadstring('return ' .. {}))}})[2], {}) }})[2]",
        json!(nvim_fn_call),
        args.iter()
            .map(|s| {
                match serde_json::from_str(s) {
                    Ok(JSONValue::String(val)) => val.to_string(),
                    Ok(JSONValue::Number(val)) => val.to_string(),
                    Ok(JSONValue::Bool(val)) => val.to_string(),
                    Ok(JSONValue::Null) => "nil".to_string(),
                    Ok(JSONValue::Array(_)) | Ok(JSONValue::Object(_)) => {
                        format!("vim.fn.json_decode({})", json!(s).to_string())
                    }
                    Err(_) => json!(s).to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(", ")
    );

    let full_command = format!("luaeval({}) ? 0 : ''", json!(lua_code));

    #[cfg(debug_assertions)]
    println!("Lua Code: {}", lua_code);
    spawn_detached(
        Command::new("nvim")
            .arg("--server")
            .arg(nvim_address)
            .arg("--remote-expr")
            .arg(full_command),
    );
}

// macro_rules! die {
//     ($($arg:tt)*) => {{
//         use std::process;
//         eprintln!($($arg)*);
//         process::exit(0);
//     }};
// }

fn spawn_detached(com: &mut Command) {
    #[cfg(unix)]
    {
        use std::{fs::File, os::unix::process::CommandExt};
        let devnull = File::open("/dev/null")?;
        com.stdin(devnull.try_clone()?)
            .stdout(devnull.try_clone()?)
            .stderr(devnull)
            .before_exec(|| {
                unsafe {
                    libc::setsid();
                }
                Ok(())
            })
            .spawn()
            .ok();
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        com.creation_flags(DETACHED_PROCESS).spawn().ok();
    }
}
