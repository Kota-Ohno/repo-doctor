#[test]
fn readme_commands_work() {
    trycmd::TestCases::new().case("tests/cmd/*.toml");
}
