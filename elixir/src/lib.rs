mod build_info;
mod parser;

rustler::init!("Elixir.Vektor.Runtime.VXL", [parser::parse, build_info::build_info]);
