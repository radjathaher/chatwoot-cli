mod command_tree;
mod http;
mod openapi;

use anyhow::{Context, Result, anyhow};
use clap::{Arg, ArgAction, Command};
use command_tree::{CommandTree, InputField, Operation, ParamDef, RequestBody};
use http::HttpClient;
use openapi::{parse_input_value, value_to_form_field};
use serde_json::{Map, Value, json};
use std::env;
use std::io::Write;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let tree = command_tree::load_command_tree();
    let cli = build_cli(&tree);
    let matches = cli.get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        return handle_list(&tree, matches);
    }
    if let Some(matches) = matches.subcommand_matches("describe") {
        return handle_describe(&tree, matches);
    }
    if let Some(matches) = matches.subcommand_matches("tree") {
        return handle_tree(&tree, matches);
    }

    let token = env::var("CHATWOOT_API_TOKEN")
        .ok()
        .or_else(|| env::var("CHATWOOT_API_ACCESS_TOKEN").ok());
    let base_url = env::var("CHATWOOT_BASE_URL").unwrap_or_else(|_| tree.base_url.clone());

    let pretty = matches.get_flag("pretty");
    let raw = matches.get_flag("raw");

    let (res_name, res_matches) = matches
        .subcommand()
        .ok_or_else(|| anyhow!("resource required"))?;
    let (op_name, op_matches) = res_matches
        .subcommand()
        .ok_or_else(|| anyhow!("operation required"))?;

    let op = find_op(&tree, res_name, op_name)
        .ok_or_else(|| anyhow!("unknown command {res_name} {op_name}"))?;

    if !op.security.is_empty() && token.is_none() {
        return Err(anyhow!(
            "CHATWOOT_API_TOKEN missing (required for this endpoint)"
        ));
    }

    let mut path = op.path.clone();
    for param in op.params.iter().filter(|p| p.location == "path") {
        let value = op_matches
            .get_one::<String>(&param.flag)
            .ok_or_else(|| anyhow!("missing --{}", param.flag))?;
        path = path.replace(&format!("{{{}}}", param.name), value);
    }

    let mut query = Vec::new();
    for param in op.params.iter().filter(|p| p.location == "query") {
        if param.is_array {
            if let Some(values) = op_matches.get_many::<String>(&param.flag) {
                for value in values {
                    query.push((param.name.clone(), value.clone()));
                }
            }
        } else if let Some(value) = op_matches.get_one::<String>(&param.flag) {
            query.push((param.name.clone(), value.clone()));
        }
    }

    let (body_json, form_body) = build_body(op.request_body.as_ref(), op_matches)?;

    let client = HttpClient::new(base_url, token)?;
    let response = if let Some(form) = form_body {
        client.execute_form(&op.method, &path, &query, form)?
    } else {
        client.execute_json(&op.method, &path, &query, body_json)?
    };

    let output = render_output(&response, raw)?;
    if pretty {
        write_stdout_line(&serde_json::to_string_pretty(&output)?)?;
    } else if output.is_string() {
        write_stdout_line(output.as_str().unwrap_or(""))?;
    } else {
        write_stdout_line(&serde_json::to_string(&output)?)?;
    }

    if response.status >= 400 {
        return Err(anyhow!("http {}", response.status));
    }

    Ok(())
}

fn build_cli(tree: &CommandTree) -> Command {
    let mut cmd = Command::new("chatwoot")
        .about("Chatwoot CLI (auto-generated from OpenAPI)")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("pretty")
                .long("pretty")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Pretty-print JSON output"),
        )
        .arg(
            Arg::new("raw")
                .long("raw")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Return status, headers, and raw body"),
        );

    cmd = cmd.subcommand(
        Command::new("list")
            .about("List resources and operations")
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .help("Emit machine-readable JSON"),
            ),
    );

    cmd = cmd.subcommand(
        Command::new("describe")
            .about("Describe a specific operation")
            .arg(Arg::new("resource").required(true))
            .arg(Arg::new("op").required(true))
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .help("Emit machine-readable JSON"),
            ),
    );

    cmd = cmd.subcommand(
        Command::new("tree").about("Show full command tree").arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("Emit machine-readable JSON"),
        ),
    );

    for resource in &tree.resources {
        let mut res_cmd = Command::new(resource.name.clone())
            .about(resource.name.clone())
            .subcommand_required(true)
            .arg_required_else_help(true);
        for op in &resource.ops {
            let mut op_cmd = Command::new(op.name.clone()).about(op.summary.clone().unwrap_or_default());
            for param in &op.params {
                op_cmd = op_cmd.arg(build_param_arg(param));
            }
            if let Some(request_body) = &op.request_body {
                op_cmd = op_cmd.arg(
                    Arg::new("body")
                        .long("body")
                        .value_name("JSON")
                        .help("Raw JSON request body"),
                );
                for field in &request_body.input_fields {
                    op_cmd = op_cmd.arg(build_input_field_arg(field));
                }
            }
            res_cmd = res_cmd.subcommand(op_cmd);
        }
        cmd = cmd.subcommand(res_cmd);
    }

    cmd
}

fn build_param_arg(param: &ParamDef) -> Arg {
    let mut arg = Arg::new(&param.flag)
        .long(&param.flag)
        .value_name(&param.name)
        .required(param.required)
        .help(param.description.clone().unwrap_or_default());

    if param.is_array {
        arg = arg.action(ArgAction::Append).num_args(1..);
    }

    arg
}

fn build_input_field_arg(field: &InputField) -> Arg {
    Arg::new(&field.flag)
        .long(&field.flag)
        .value_name(&field.name)
        .help(field.description.clone().unwrap_or_default())
}

fn handle_list(tree: &CommandTree, matches: &clap::ArgMatches) -> Result<()> {
    if matches.get_flag("json") {
        let mut out = Vec::new();
        for res in &tree.resources {
            let ops: Vec<String> = res.ops.iter().map(|op| op.name.clone()).collect();
            out.push(json!({"resource": res.name, "ops": ops}));
        }
        write_stdout_line(&serde_json::to_string_pretty(&out)?)?;
        return Ok(());
    }

    for res in &tree.resources {
        write_stdout_line(&res.name)?;
        for op in &res.ops {
            write_stdout_line(&format!("  {}", op.name))?;
        }
    }
    Ok(())
}

fn handle_describe(tree: &CommandTree, matches: &clap::ArgMatches) -> Result<()> {
    let resource = matches
        .get_one::<String>("resource")
        .ok_or_else(|| anyhow!("resource required"))?;
    let op_name = matches
        .get_one::<String>("op")
        .ok_or_else(|| anyhow!("operation required"))?;

    let op = find_op(tree, resource, op_name)
        .ok_or_else(|| anyhow!("unknown command {resource} {op_name}"))?;

    if matches.get_flag("json") {
        write_stdout_line(&serde_json::to_string_pretty(op)?)?;
        return Ok(());
    }

    write_stdout_line(&format!("{resource} {op_name}"))?;
    write_stdout_line(&format!("  method: {}", op.method))?;
    write_stdout_line(&format!("  path: {}", op.path))?;
    if let Some(summary) = &op.summary {
        write_stdout_line(&format!("  summary: {summary}"))?;
    }
    if !op.params.is_empty() {
        write_stdout_line("  params:")?;
        for param in &op.params {
            let req = if param.required { "required" } else { "optional" };
            write_stdout_line(&format!(
                "    --{} ({}, {})",
                param.flag, param.location, req
            ))?;
        }
    }
    if let Some(request_body) = &op.request_body {
        write_stdout_line("  body:")?;
        write_stdout_line(&format!("    required: {}", request_body.required))?;
        write_stdout_line(&format!(
            "    content-types: {}",
            request_body.content_types.join(", ")
        ))?;
    }

    Ok(())
}

fn handle_tree(tree: &CommandTree, matches: &clap::ArgMatches) -> Result<()> {
    if matches.get_flag("json") {
        write_stdout_line(&serde_json::to_string_pretty(tree)?)?;
        return Ok(());
    }

    for res in &tree.resources {
        write_stdout_line(&res.name)?;
        for op in &res.ops {
            write_stdout_line(&format!("  {} {}", op.method.to_uppercase(), op.name))?;
        }
    }
    Ok(())
}

fn find_op<'a>(tree: &'a CommandTree, res_name: &str, op_name: &str) -> Option<&'a Operation> {
    tree.resources
        .iter()
        .find(|res| res.name == res_name)
        .and_then(|res| res.ops.iter().find(|op| op.name == op_name))
}

fn build_body(request_body: Option<&RequestBody>, matches: &clap::ArgMatches) -> Result<(Option<Value>, Option<Vec<(String, String)>>)> {
    let Some(request_body) = request_body else {
        return Ok((None, None));
    };

    let raw_body = matches.get_one::<String>("body");
    let mut input_map: Map<String, Value> = Map::new();

    for field in &request_body.input_fields {
        if let Some(value) = matches.get_one::<String>(&field.flag) {
            input_map.insert(field.name.clone(), parse_input_value(value));
        }
    }

    if raw_body.is_some() && !input_map.is_empty() {
        return Err(anyhow!("use either --body or --input-* flags"));
    }

    let body_json = if let Some(raw) = raw_body {
        Some(
            serde_json::from_str(raw)
                .with_context(|| format!("invalid JSON for --body"))?,
        )
    } else if !input_map.is_empty() {
        Some(Value::Object(input_map))
    } else {
        None
    };

    if request_body.required && body_json.is_none() {
        return Err(anyhow!("request body required"));
    }

    if let Some(Value::Object(map)) = &body_json {
        let missing: Vec<String> = request_body
            .required_fields
            .iter()
            .filter(|name| !map.contains_key(*name))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(anyhow!("missing required fields: {}", missing.join(", ")));
        }
    }

    let prefers_form = request_body
        .content_types
        .iter()
        .any(|t| t == "application/x-www-form-urlencoded")
        && !request_body
            .content_types
            .iter()
            .any(|t| t == "application/json");

    if prefers_form {
        let Some(Value::Object(map)) = body_json else {
            if body_json.is_some() {
                return Err(anyhow!("form body must be a JSON object"));
            }
            return Ok((None, Some(Vec::new())));
        };
        let mut form = Vec::new();
        for (key, value) in map {
            let Some(field) = value_to_form_field(&value) else {
                return Err(anyhow!("form body only supports string/number/bool"));
            };
            form.push((key.clone(), field));
        }
        return Ok((None, Some(form)));
    }

    Ok((body_json, None))
}

fn render_output(response: &http::ResponseData, raw: bool) -> Result<Value> {
    let parsed = serde_json::from_str::<Value>(&response.body).ok();
    if raw {
        let mut headers = Map::new();
        for (key, value) in &response.headers {
            headers.insert(key.clone(), Value::String(value.clone()));
        }
        let body = parsed.unwrap_or(Value::String(response.body.clone()));
        return Ok(json!({
            "status": response.status,
            "headers": headers,
            "body": body
        }));
    }

    Ok(parsed.unwrap_or(Value::String(response.body.clone())))
}

fn write_stdout_line(line: &str) -> Result<()> {
    let mut out = std::io::stdout().lock();
    writeln!(out, "{line}").context("write stdout")?;
    Ok(())
}
