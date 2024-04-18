fn main() {
    cynic_codegen::register_schema("tjing")
        .from_sdl_file("schemas/tjing.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
