//! The ssp command-line tool.
//!
//! Reads an epoch record (.qf or .json), derives liquidity regimes and renders
//! either a risk report or a digest-stamped stability attestation.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use ssp::adjudicate::Policy;
use ssp::json::{self, Json};
use ssp::model::Deliberation;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("ssp: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        print_help();
        return Ok(ExitCode::from(2));
    }

    let command = args[0].clone();
    if command == "help" || command == "-h" || command == "--help" {
        print_help();
        return Ok(ExitCode::SUCCESS);
    }

    let mut format = String::from("text");
    let mut force_json = false;
    let mut output: Option<String> = None;
    let mut policy = Policy::default();
    let mut input: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                format = args.get(i).cloned().unwrap_or_else(|| "text".into());
            }
            "--consensus" => {
                i += 1;
                policy.consensus_threshold = parse_number(args.get(i))?;
            }
            "--dissent" => {
                i += 1;
                policy.dissent_ceiling = parse_number(args.get(i))?;
            }
            "--min-mass" => {
                i += 1;
                policy.minimum_mass = parse_number(args.get(i))?;
            }
            "--json" => force_json = true,
            "-o" | "--output" => {
                i += 1;
                output = args.get(i).cloned();
                if output.is_none() {
                    return Err("--output requires a path".into());
                }
            }
            other if other.starts_with("--") => return Err(format!("unknown option '{other}'")),
            other => input = Some(other.to_string()),
        }
        i += 1;
    }

    let path = input.ok_or_else(|| "no record file given (use '-' for stdin)".to_string())?;
    let source = read_source(&path)?;
    // parse_auto picks the encoding from the extension, so --json renames the
    // input for the parser's eyes only.
    let parse_path = if force_json {
        "input.json".to_string()
    } else {
        path.clone()
    };

    match command.as_str() {
        "adjudicate" => {
            let (delib, adj) =
                ssp::run(&parse_path, &source, &policy).map_err(|e| e.to_string())?;
            let rendered = if format == "json" {
                ssp::report::render_json(&delib, &adj)
            } else {
                ssp::report::render_text(&delib, &adj)
            };
            emit(output, &rendered)
        }
        "inspect" => {
            let mut delib =
                ssp::parse::parse_auto(&parse_path, &source).map_err(|e| e.to_string())?;
            ssp::normalize::normalize_deliberation(&mut delib);
            emit(output, &inspect_json(&delib))
        }
        "bundle" => {
            let (delib, adj) =
                ssp::run(&parse_path, &source, &policy).map_err(|e| e.to_string())?;
            let bundle = ssp::bundle::build(&delib, &adj);
            emit(output, &ssp::bundle::to_json(&bundle))
        }
        "verify" => {
            let (delib, adj) =
                ssp::run(&parse_path, &source, &policy).map_err(|e| e.to_string())?;
            let bundle = ssp::bundle::build(&delib, &adj);
            let body = ssp::bundle::to_json(&bundle);
            if ssp::bundle::verify(&bundle) {
                println!("digest OK: {}", ssp::bundle::fnv1a_hex(body.as_bytes()));
                Ok(ExitCode::SUCCESS)
            } else {
                eprintln!("digest mismatch");
                Ok(ExitCode::from(4))
            }
        }
        other => Err(format!("unknown command '{other}'")),
    }
}

fn read_source(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| e.to_string())?;
        Ok(buffer)
    } else {
        fs::read_to_string(path).map_err(|e| format!("cannot read '{path}': {e}"))
    }
}

fn emit(output: Option<String>, text: &str) -> Result<ExitCode, String> {
    match output {
        Some(path) => {
            fs::write(&path, text).map_err(|e| format!("cannot write '{path}': {e}"))?;
            Ok(ExitCode::SUCCESS)
        }
        None => {
            print!("{text}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn parse_number(value: Option<&String>) -> Result<f64, String> {
    let raw = value.ok_or_else(|| "missing value".to_string())?;
    raw.parse::<f64>()
        .map_err(|_| format!("expected a number, got '{raw}'"))
}

fn inspect_json(delib: &Deliberation) -> String {
    let agents: Vec<Json> = delib
        .agents
        .values()
        .map(|a| {
            json::obj(vec![
                ("id", json::s(&a.id)),
                ("name", json::s(&a.name)),
                ("weight", json::num(a.weight)),
                ("role", json::s(&a.role)),
            ])
        })
        .collect();

    let claims: Vec<Json> = delib
        .claims
        .values()
        .map(|c| {
            json::obj(vec![
                ("id", json::s(&c.id)),
                ("topic", json::s(&c.topic)),
                ("text", json::s(&c.text)),
                ("normalized", json::s(&c.normalized)),
            ])
        })
        .collect();

    let positions: Vec<Json> = delib
        .positions
        .iter()
        .map(|p| {
            let citations: Vec<Json> = p
                .citations
                .iter()
                .map(|c| {
                    json::obj(vec![
                        ("source", json::s(&c.source)),
                        ("locator", json::s(&c.locator)),
                    ])
                })
                .collect();
            json::obj(vec![
                ("agent", json::s(&p.agent_id)),
                ("claim", json::s(&p.claim_id)),
                ("stance", json::s(p.stance.as_token())),
                ("confidence", json::num(p.confidence)),
                ("note", json::s(&p.note)),
                ("citations", Json::Arr(citations)),
            ])
        })
        .collect();

    let document = json::obj(vec![
        ("id", json::s(&delib.id)),
        ("question", json::s(&delib.question)),
        ("agents", Json::Arr(agents)),
        ("claims", Json::Arr(claims)),
        ("positions", Json::Arr(positions)),
    ]);

    json::to_string_pretty(&document)
}

fn print_help() {
    print!(
        "ssp <command> [options] <record-file | ->\n\n\
COMMANDS\n\
  adjudicate   Parse, normalize and print a risk report (text or json).\n\
  bundle       Emit a digest-stamped stability attestation.\n\
  verify       Re-derive an attestation digest and confirm it.\n\
  inspect      Print the parsed, normalized epoch record as JSON.\n\
  help         Print usage.\n\n\
OPTIONS\n\
  --format <text|json>   Report format for adjudicate. Default: text.\n\
  --consensus <0..1>     Expansion threshold. Default: 0.66.\n\
  --dissent <0..1>       Stress ceiling. Default: 0.34.\n\
  --min-mass <n>         Minimum decisive depth to escape BACKSTOP.\n\
  -o, --output <path>    Write to a file instead of stdout.\n\
  --json                 Treat stdin or extensionless input as JSON.\n"
    );
}
