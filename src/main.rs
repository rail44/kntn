extern crate getopts;
extern crate handlebars;
extern crate rand;
#[macro_use]
extern crate serde_json;

use handlebars::{Helper, HelperDef, Handlebars, RenderContext, RenderError, to_json};
use serde_json::value::Value;
use rand::{Rng, SeedableRng, XorShiftRng};
use std::env;
use std::process::exit;
use std::io::{BufWriter, stdout};
use std::sync::{Arc, Mutex};

const BUFSIZE: usize = 8196;

struct RandomStr(Arc<Mutex<XorShiftRng>>);
impl HelperDef for RandomStr {
    fn call(&self, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        let l = h.param(0).unwrap().value().as_u64().unwrap() as usize;
        let r: String = {
            self.0
                .lock()
                .unwrap()
                .gen_ascii_chars()
                .take(l)
                .collect()
        };
        try!(rc.writer.write(r.into_bytes().as_ref()));
        Ok(())
    }
}

struct RandomInt(Arc<Mutex<XorShiftRng>>);
impl HelperDef for RandomInt {
    fn call(&self, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        let digits = h.param(0).unwrap().value().as_u64().unwrap() as u32;
        let ten: u64 = 10;
        let r = {
            self.0.lock().unwrap().gen_range(
                ten.pow(digits - 1),
                ten.pow(digits) - 1,
            )
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

struct Config {
    pub template: String,
    pub seed: Option<[u32; 4]>,
    pub data: serde_json::Value,
}

impl<'a> From<&'a getopts::Matches> for Config {
    fn from(matches: &'a getopts::Matches) -> Config {
        Config {
            template: matches.opt_str("template").unwrap(),
            seed: matches.opt_str("seed").map(|s| {
                let mut seed_iter = s.split(',').map(|x| x.parse().unwrap());
                let mut seed = [0; 4];
                for i in 0..4 {
                    seed[i] = seed_iter.next().unwrap();
                }
                seed
            }),
            data: matches.opt_str("data").map_or_else(
                || json!({}),
                |s| {
                    serde_json::from_str(s.as_str()).unwrap()
                },
            ),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("", "template", "", "PATH");
    opts.optopt("", "seed", "", "u32,u32,u32,u32");
    opts.optopt("", "data", "", "JSON");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!(
                "v{}\n{}\n{}",
                env!("CARGO_PKG_VERSION"),
                opts.short_usage(args[0].as_str()),
                f
            );
            exit(1);
        }
    };

    let config = Config::from(&matches);

    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("", &config.template).unwrap();

    let rng = Arc::new(Mutex::new(config.seed.map_or_else(
        || XorShiftRng::new_unseeded(),
        |seed| XorShiftRng::from_seed(seed)
    )));

    let random_str = RandomStr(rng.clone());
    handlebars.register_helper("str", Box::new(random_str));
    let random_int = RandomInt(rng);
    handlebars.register_helper("int", Box::new(random_int));
    let range = Range;
    handlebars.register_helper("range", Box::new(range));

    let stdout = stdout();
    handlebars.renderw(
        "",
        &config.data,
        &mut BufWriter::with_capacity(BUFSIZE, stdout.lock()),
    ).unwrap();
}
