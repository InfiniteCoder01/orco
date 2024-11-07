fn main() {
    let ast: orco_c::Unit = orco_c::parsel::parse_str(
        &std::fs::read_to_string("frontends/orco-c/samples/simple.c").unwrap(),
    )
    .unwrap();

    // use orco_c::parsel::ToTokens;
    // println!("{}", ast.into_token_stream());
    println!("{}", &ast as &dyn orco::Unit);
}
