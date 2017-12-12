extern crate getopts;
extern crate handlebars;
extern crate rand;
extern crate serde_json;

use getopts::Options;
use handlebars::{Helper, HelperDef, Handlebars, RenderContext, RenderError, to_json};
use serde_json::value::Value;
use rand::{Rng, SeedableRng, XorShiftRng};
use std::env;
use std::process::exit;
use std::io::stdout;
use std::sync::{Arc, Mutex};

struct RandomStr(Arc<Mutex<XorShiftRng>>);
impl HelperDef for RandomStr {
    fn call(
        &self,
        h: &Helper,
        _: &Handlebars,
        rc: &mut RenderContext,
    ) -> Result<(), RenderError> {
        let r: String = {
            self.0.lock().unwrap().gen_ascii_chars()
                .take(h.param(0).unwrap().value().as_u64().unwrap() as usize)
                .collect()
        };
        try!(rc.writer.write(r.into_bytes().as_ref()));
        Ok(())
    }
}

struct RandomInt(Arc<Mutex<XorShiftRng>>);
impl HelperDef for RandomInt {
    fn call(
        &self,
        h: &Helper,
        _: &Handlebars,
        rc: &mut RenderContext,
    ) -> Result<(), RenderError> {
        let digits = h.param(0).unwrap().value().as_u64().unwrap() as u32;
        let ten: u64 = 10;
        let r = {
            self.0.lock().unwrap().gen_range(ten.pow(digits - 1), ten.pow(digits) - 1)
        };
        try!(rc.writer.write(r.to_string().into_bytes().as_ref()));
        Ok(())
    }
}

struct Range;
impl HelperDef for Range {
    fn call_inner(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &mut RenderContext,
    ) -> Result<Option<Value>, RenderError> {
        let n = h.param(0).unwrap().value().as_u64().unwrap();
        let vec: Vec<u64> = (0..n).collect();
        Ok(Some(to_json(&vec)))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.reqopt("", "template", "", "[PATH]");
    opts.reqopt("", "seed", "", "[u32,u32,u32,u32]");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("v{}\n{}\n{}", env!("CARGO_PKG_VERSION"), opts.short_usage(args[0].as_str()), f);
            exit(1);
        }
    };
    let mut handlebars = Handlebars::new();
    let template = matches.opt_str("template");
    if let Err(e) = handlebars.register_template_file("", template.unwrap()) {
        panic!("{}", e);
    }

    let seed_str = matches
        .opt_str("seed")
        .unwrap();

    let mut seed_iter = seed_str
        .split(',')
        .map(|x| x.parse().unwrap());

    let mut seed = [0; 4];
    for i in 0..4 {
        seed[i] = seed_iter.next().unwrap();
    }

    let rng = Arc::new(Mutex::new(XorShiftRng::from_seed(seed)));
    let random_str = RandomStr(rng.clone());
    handlebars.register_helper("str", Box::new(random_str));
    let random_int = RandomInt(rng);
    handlebars.register_helper("int", Box::new(random_int));
    let range = Range;
    handlebars.register_helper("range", Box::new(range));

    handlebars
        .renderw("", &(), &mut stdout())
        .unwrap();
}
