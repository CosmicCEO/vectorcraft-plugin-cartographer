// usage: fuel <plugin.wasm> <input.json> <params.json>  -> prints the fuel (about one per instruction) the plug-in used.
use wasmi::*;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let wasm = std::fs::read(&a[1]).unwrap();
    let input = std::fs::read(&a[2]).unwrap();
    let params = std::fs::read(&a[3]).unwrap();
    let mut cfg = Config::default();
    cfg.consume_fuel(true);
    let engine = Engine::new(&cfg);
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let mut store = Store::new(&engine, ());
    store.set_fuel(u64::MAX).unwrap();
    let linker = <Linker<()>>::new(&engine);
    let inst = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
    let mem = inst.get_memory(&store, "memory").unwrap();
    let alloc = inst.get_typed_func::<u32, u32>(&store, "vc_alloc").unwrap();
    let run = inst.get_typed_func::<(u32, u32, u32, u32), i64>(&store, "vc_run").unwrap();
    let ip = alloc.call(&mut store, input.len() as u32).unwrap();
    mem.write(&mut store, ip as usize, &input).unwrap();
    let pp = alloc.call(&mut store, params.len() as u32).unwrap();
    mem.write(&mut store, pp as usize, &params).unwrap();
    let before = store.get_fuel().unwrap();
    let r = run.call(&mut store, (ip, input.len() as u32, pp, params.len() as u32)).unwrap();
    let used = before - store.get_fuel().unwrap();
    let (ptr, len) = ((r as u64 & 0xffff_ffff) as usize, (r as u64 >> 32) as usize);
    let mut out = vec![0u8; len];
    mem.read(&store, ptr, &mut out).unwrap();
    println!("fuel {} output_bytes {} starts {}", used, len, String::from_utf8_lossy(&out[..out.len().min(60)]));
}
