use std::io;
use std::process::ExitCode;

use std::fs::File;

use std::sync::Arc;
use std::sync::RwLock;

use io::BufRead;
use io::Read;

use io::BufWriter;
use io::Write;

use serde_json::Map;
use serde_json::Value;

use wasmtime::Caller;
use wasmtime::Engine;
use wasmtime::Extern;
use wasmtime::Instance;
use wasmtime::Linker;
use wasmtime::Memory;
use wasmtime::Module;
use wasmtime::ModuleExport;
use wasmtime::Store;
use wasmtime::TypedFunc;

fn lines() -> impl Iterator<Item = Result<String, io::Error>> {
    io::stdin().lock().lines()
}

fn jsons() -> impl Iterator<Item = Result<Map<String, Value>, io::Error>> {
    lines().map(|r| r.and_then(|s: String| serde_json::from_str(&s).map_err(io::Error::other)))
}

fn wat_path() -> Result<String, io::Error> {
    std::env::var("ENV_WAT_PATH").map_err(io::Error::other)
}

const MAX_WAT_SIZE: u64 = 1048576;

fn wat_bytes() -> Result<Vec<u8>, io::Error> {
    let wpath: String = wat_path()?;
    let f: File = File::open(wpath)?;
    let mut taken = f.take(MAX_WAT_SIZE);
    let mut buf: Vec<u8> = vec![];
    taken.read_to_end(&mut buf)?;
    Ok(buf)
}

fn sub() -> Result<(), io::Error> {
    let we: Engine = Engine::default();

    let mut wl: Linker<()> = Linker::new(&we);

    let wat: Vec<u8> = wat_bytes()?;
    let wm: Module = Module::new(&we, &wat).map_err(io::Error::other)?;
    let mut ws: Store<()> = Store::new(&we, ());

    let memhint: ModuleExport = wm
        .get_export_index("memory")
        .ok_or(io::Error::other("memory not found"))?;

    let lmap: RwLock<Map<String, Value>> = RwLock::default();
    let amap: Arc<RwLock<_>> = Arc::new(lmap);
    let amap4wasm: Arc<_> = amap.clone();

    let amap4reset: Arc<_> = amap4wasm.clone();
    let amap4delete: Arc<_> = amap4wasm.clone();
    let amap4get_int: Arc<_> = amap4wasm.clone();
    let amap4overwrite_lng: Arc<_> = amap4wasm.clone();

    wl.func_wrap("host", "reset_omap", move || {
        let am: &Arc<_> = &amap4reset;
        let rm: &RwLock<_> = am;
        let rguard = rm.try_write();
        let Ok(mut guard) = rguard else {
            return -1;
        };
        let mm: &mut Map<_, _> = &mut guard;
        mm.clear();
        0
    })
    .map_err(io::Error::other)?;

    let caller2mem = move |caller: &mut Caller<'_, ()>| {
        let omemext: Option<Extern> = caller.get_module_export(&memhint);
        let memext: Extern = omemext?;
        match memext {
            Extern::Memory(m) => Some(m),
            _ => None,
        }
    };

    wl.func_wrap(
        "host",
        "delete_key",
        move |mut caller: Caller<'_, ()>, keyptr: i32, keylen: i32| {
            let omem: Option<Memory> = caller2mem(&mut caller);
            let Some(mem) = omem else {
                return -2;
            };

            let offset: usize = keyptr as usize;
            let mut buf: [u8; 65536] = [0; 65536];
            let size: usize = buf.len().min(keylen as usize);
            let read_ok: bool = mem.read(&caller, offset, &mut buf[..size]).is_ok();
            if !read_ok {
                return -3;
            }

            let skey: &[u8] = &buf[..size];
            let key: &str = std::str::from_utf8(skey).unwrap_or_default();

            let am: &Arc<_> = &amap4delete;
            let rm: &RwLock<_> = am;
            let rguard = rm.try_write();
            let Ok(mut guard) = rguard else {
                return -1;
            };
            let mm: &mut Map<_, _> = &mut guard;

            mm.remove(key);

            0
        },
    )
    .map_err(io::Error::other)?;

    wl.func_wrap(
        "host",
        "get_int_by_key",
        move |mut caller: Caller<'_, ()>, keyptr: i32, keylen: i32| {
            let omem: Option<Memory> = caller2mem(&mut caller);
            let Some(mem) = omem else {
                return (0, -2);
            };

            let offset: usize = keyptr as usize;
            let mut buf: [u8; 65536] = [0; 65536];
            let size: usize = buf.len().min(keylen as usize);
            let read_ok: bool = mem.read(&caller, offset, &mut buf[..size]).is_ok();
            if !read_ok {
                return (0, -3);
            }

            let skey: &[u8] = &buf[..size];
            let key: &str = std::str::from_utf8(skey).unwrap_or_default();

            let am: &Arc<_> = &amap4get_int;
            let rm: &RwLock<_> = am;
            let rguard = rm.try_read();
            let Ok(guard) = rguard else {
                return (0, -4);
            };
            let rm: &Map<_, _> = &guard;
            let oval: Option<&Value> = rm.get(key);
            let Some(val) = oval else {
                return (0, -5);
            };
            let rval = match val {
                Value::Number(n) => Ok(n),
                _ => Err("not a number"),
            };
            let Ok(num) = rval else {
                return (0, -6);
            };
            let olong: Option<i64> = num.as_i64();
            let Some(lng) = olong else {
                return (0, -7);
            };
            let rint: Result<i32, _> = lng.try_into();
            match rint {
                Ok(i) => (i, 0),
                Err(_) => (0, -8),
            }
        },
    )
    .map_err(io::Error::other)?;

    wl.func_wrap("host", "int2hex2u64", move |i: i32| {
        rs_i2hex4wasm::int2hex2string2u64le(i)
    })
    .map_err(io::Error::other)?;

    wl.func_wrap(
        "host",
        "overwrite_long_as_hex",
        move |mut caller: Caller<'_, ()>, keyptr: i32, keylen: i32, val: i64| {
            let omem: Option<Memory> = caller2mem(&mut caller);
            let Some(mem) = omem else {
                return -2;
            };

            let offset: usize = keyptr as usize;
            let mut buf: [u8; 65536] = [0; 65536];
            let size: usize = buf.len().min(keylen as usize);
            let read_ok: bool = mem.read(&caller, offset, &mut buf[..size]).is_ok();
            if !read_ok {
                return -3;
            }

            let skey: &[u8] = &buf[..size];
            let key: &str = std::str::from_utf8(skey).unwrap_or_default();

            let am: &Arc<_> = &amap4overwrite_lng;
            let rm: &RwLock<_> = am;
            let rguard = rm.try_write();
            let Ok(mut guard) = rguard else {
                return -1;
            };
            let mm: &mut Map<_, _> = &mut guard;

            let leb: [u8; 8] = val.to_le_bytes();
            let les: &str = std::str::from_utf8(&leb).unwrap_or_default();

            let jval: Value = Value::from(les);
            mm.insert(key.into(), jval);

            0
        },
    )
    .map_err(io::Error::other)?;

    let wi: Instance = wl.instantiate(&mut ws, &wm).map_err(io::Error::other)?;

    let wconv: TypedFunc<(), ()> = wi
        .get_typed_func(&mut ws, "converter")
        .map_err(io::Error::other)?;

    let mut ol = io::stdout().lock();
    let mut bw = BufWriter::new(&mut ol);

    let irmap = jsons();
    for rmap in irmap {
        let jmap: Map<String, Value> = rmap?;

        {
            let rmap: &RwLock<_> = &amap;
            let mut guard = rmap
                .try_write()
                .map_err(|_| "unable to write lock")
                .map_err(io::Error::other)?;
            let mm: &mut Map<_, _> = &mut guard;
            *mm = jmap;
        }

        wconv.call(&mut ws, ()).map_err(io::Error::other)?;

        {
            let rmap: &RwLock<_> = &amap;
            let guard = rmap
                .try_read()
                .map_err(|_| "unable to read lock")
                .map_err(io::Error::other)?;
            let mm: &Map<_, _> = &guard;
            serde_json::to_writer(&mut bw, mm)?;
        }

        writeln!(&mut bw)?;
    }

    bw.flush()?;
    drop(bw);
    ol.flush()?;

    Ok(())
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
