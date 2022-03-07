mod build_info;
mod parser;

rustler::init!("Elixir.VXLParser", [parser::parse, build_info::build_info]);
