use std::fs::File;
use std::io;
use std::process::ExitCode;

use wasmtime::Engine;
use wasmtime::Instance;
use wasmtime::Linker;
use wasmtime::Memory;
use wasmtime::Module;
use wasmtime::Store;
use wasmtime::TypedFunc;

use rs_fd2locked2mmap2lc4wasm::FlockMode;
use rs_fd2locked2mmap2lc4wasm::LockedFd;
use rs_fd2locked2mmap2lc4wasm::ReadonlyMmap;
use rs_fd2locked2mmap2lc4wasm::UnlockedFd;

fn get_filename() -> Result<String, io::Error> {
    std::env::var("ENV_FILENAME").map_err(io::Error::other)
}

fn get_wat_filename() -> Result<String, io::Error> {
    std::env::var("ENV_WAT_FILENAME").map_err(io::Error::other)
}

enum CountMode {
    Native,
    Wasm,
}

fn get_count_mode() -> CountMode {
    match std::env::var("ENV_COUNT_MODE").unwrap_or_default().as_str() {
        "wasm" => CountMode::Wasm,
        "native" => CountMode::Native,
        _ => CountMode::Native,
    }
}

fn get_wat_bytes() -> Result<Vec<u8>, io::Error> {
    let wat_name: String = get_wat_filename()?;
    std::fs::read(wat_name)
}

fn sub() -> Result<(), io::Error> {
    let wat: Vec<u8> = get_wat_bytes()?;
    let we: Engine = Engine::default();
    let wm: Module = Module::new(&we, &wat).map_err(io::Error::other)?;
    let wl: Linker<()> = Linker::new(&we);
    let mut ws: Store<()> = Store::new(&we, ());
    let wi: Instance = wl.instantiate(&mut ws, &wm).map_err(io::Error::other)?;
    let wm: Memory = wi
        .get_memory(&mut ws, "memory")
        .ok_or(io::Error::other("unable to get memory"))?;
    let reset_mem: TypedFunc<(), ()> = wi
        .get_typed_func::<(), ()>(&mut ws, "reset_mem")
        .map_err(io::Error::other)?;
    let count_line: TypedFunc<(), i64> = wi
        .get_typed_func::<(), i64>(&mut ws, "count_line")
        .map_err(io::Error::other)?;

    let fname: String = get_filename()?;
    let f: File = File::open(fname)?;
    let unlocked: UnlockedFd = (&f).into();
    let fmode: FlockMode = FlockMode::ExclusiveNonBlocking;
    let locked: LockedFd = unlocked.into_locked(fmode)?;
    let rmap: ReadonlyMmap = locked.into_map()?;
    let cmod: CountMode = get_count_mode();
    let lcnt: usize = match cmod {
        CountMode::Wasm => rmap.count_lines(|dat: &[u8]| {
            let sz: usize = dat.len();
            match sz {
                0 => Ok(0),
                65536 => {
                    wm.write(&mut ws, 0, dat).map_err(io::Error::other)?;
                    let subtot: i64 = count_line.call(&mut ws, ()).map_err(io::Error::other)?;
                    let st: usize = subtot as usize;
                    Ok(st)
                }
                _ => {
                    reset_mem.call(&mut ws, ()).map_err(io::Error::other)?;
                    wm.write(&mut ws, 0, dat).map_err(io::Error::other)?;
                    let subtot: i64 = count_line.call(&mut ws, ()).map_err(io::Error::other)?;
                    let st: usize = subtot as usize;
                    Ok(st)
                }
            }
        })?,
        CountMode::Native => rmap.count_lines_native()?,
    };

    println!("line count: {lcnt}");
    Ok(())
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
